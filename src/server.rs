use crate::config::load_config;
use crate::kvstore::KVStore;
use crate::utils::{
    format_header, format_session_id, get_client_ip, get_lan_ip, get_session_from_header,
    is_local_port_available, parse_compact, parse_delete_command, parse_get_command,
    parse_identifier_get, parse_identifier_set, parse_put_command, server_info,
};
use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer, Responder, get, post,
    web::{self, Data, Json},
};
use anyhow::{Result, anyhow};
use clap::Parser;
use dashmap::DashMap;
use futures::executor::block_on;
use log::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time;
use uuid::Uuid;

static PRINT_HEADER: OnceLock<bool> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, help = "Output requests headers")]
    header: bool,
}

// 会话结构体
struct Session {
    store: Mutex<Option<KVStore>>,
    last_active: Mutex<Instant>,
    current_path: Mutex<Option<String>>,
}

impl Session {
    fn new() -> Self {
        Session {
            store: Mutex::new(None),
            last_active: Mutex::new(Instant::now()),
            current_path: Mutex::new(None),
        }
    }
}

// 会话管理器
type SessionManager = Arc<DashMap<String, Arc<Session>>>;

// API 数据结构
#[derive(Deserialize)]
struct KeyValueRequest {
    key: String,
    value: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct PathRequest {
    path: String,
}

#[derive(Deserialize)]
struct IdentifierRequest {
    identifier: String,
}

#[derive(Serialize)]
struct KeyValueResponse {
    key: String,
    value: Option<String>,
}

#[derive(Serialize)]
struct IdentifierResponse {
    identifier: String,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
}

// 初始化会话管理器
fn init_session_manager() -> SessionManager {
    let manager: SessionManager = Arc::new(DashMap::new());
    start_session_cleanup(manager.clone());
    manager
}

// 会话清理任务
fn start_session_cleanup(manager: SessionManager) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60 * 5)); // 每5分钟检查一次
        loop {
            interval.tick().await;
            let now = Instant::now();
            // 使用 retain 的同步版本，在闭包内获取锁
            manager.retain(|_, session| {
                // 同步获取锁（注意：这里需要确保锁不会长时间持有）
                let last_active = block_on(session.last_active.lock());
                now.duration_since(*last_active) < Duration::from_secs(60 * 30) // 30分钟超时
            });
        }
    });
}

// 获取或创建会话
async fn get_or_create_session(
    sessions: Data<SessionManager>,
    session_id: Option<String>,
) -> (String, Arc<Session>) {
    if let Some(id) = session_id {
        if let Some(session) = sessions.get(&id) {
            *session.last_active.lock().await = Instant::now();
            return (id, session.clone());
        }
    }

    let new_id = Uuid::new_v4().to_string();
    let new_session = Arc::new(Session::new());
    sessions.insert(new_id.clone(), new_session.clone());
    (new_id, new_session)
}

// API 处理函数
#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    server_info(
        get_client_ip(&req).as_str(),
        req.method().as_str(),
        req.path(),
    );
    HttpResponse::Ok().body("Wind-KVStore Server is Running!")
}

#[post("/api/open")]
async fn open_db(
    sessions: Data<SessionManager>,
    req: Json<PathRequest>,
    session_id: Option<String>,
    http_req: HttpRequest,
) -> impl Responder {
    server_info(
        get_client_ip(&http_req).as_str(),
        http_req.method().as_str(),
        http_req.path(),
    );

    let (session_id, session) = get_or_create_session(sessions, session_id).await;
    println!(" * Create Session-ID: {}\n", session_id.clone());
    format_header(&http_req, PRINT_HEADER.clone());

    let mut store = session.store.lock().await;

    match KVStore::open(&req.path, None) {
        Ok(kv_store) => {
            *store = Some(kv_store);
            *session.current_path.lock().await = Some(req.path.clone());
            *session.last_active.lock().await = Instant::now();

            HttpResponse::Ok()
                .insert_header(("X-Session-ID", session_id))
                .json(StatusResponse {
                    status: "Database opened".to_string(),
                })
        }
        Err(e) => {
            error!("Failed to open database: {}", e);
            HttpResponse::InternalServerError().json(StatusResponse {
                status: format!("Error: {}", e),
            })
        }
    }
}

#[get("/api/close")]
async fn close_db(sessions: Data<SessionManager>, http_req: HttpRequest) -> impl Responder {
    server_info(
        get_client_ip(&http_req).as_str(),
        http_req.method().as_str(),
        http_req.path(),
    );
    let session_id = get_session_from_header(&http_req);
    format_session_id(&session_id);
    format_header(&http_req, PRINT_HEADER.clone());

    let (session_id, session) = get_or_create_session(sessions, Option::from(session_id)).await;
    let mut store = session.store.lock().await;

    match store.as_mut() {
        Some(store) => store,
        None => {
            return HttpResponse::BadRequest().json(StatusResponse {
                status: "No database open".to_string(),
            });
        }
    };

    if let Some(kv_store) = store.take() {
        if let Err(e) = kv_store.close() {
            error!("Failed to close database: {}", e);
            return HttpResponse::InternalServerError().json(StatusResponse {
                status: format!("Error: {}", e),
            });
        }
    }

    *session.current_path.lock().await = None;
    *session.last_active.lock().await = Instant::now();

    HttpResponse::Ok()
        .insert_header(("X-Session-ID", session_id))
        .json(StatusResponse {
            status: "Database closed".to_string(),
        })
}

#[post("/api/put")]
async fn put_value(
    sessions: Data<SessionManager>,
    req: Json<Vec<KeyValueRequest>>,
    http_req: HttpRequest,
) -> impl Responder {
    server_info(
        get_client_ip(&http_req).as_str(),
        http_req.method().as_str(),
        http_req.path(),
    );

    let session_id = get_session_from_header(&http_req);
    format_session_id(&session_id);
    format_header(&http_req, PRINT_HEADER.clone());

    let (session_id, session) = get_or_create_session(sessions, Option::from(session_id)).await;
    let mut store = session.store.lock().await;

    let kv_store = match store.as_mut() {
        Some(store) => store,
        None => {
            return HttpResponse::BadRequest().json(StatusResponse {
                status: "No database open".to_string(),
            });
        }
    };

    let mut success = 0;
    let mut errors = Vec::new();

    for kv in req.iter() {
        if let Some(value) = &kv.value {
            if let Err(e) = kv_store.put(kv.key.as_bytes(), value.as_bytes()) {
                errors.push(format!("{}: {}", kv.key, e));
            } else {
                success += 1;
            }
        }
    }

    *session.last_active.lock().await = Instant::now();

    if errors.is_empty() {
        HttpResponse::Ok()
            .insert_header(("X-Session-ID", session_id))
            .json(StatusResponse {
                status: format!("Inserted {} key-value pairs", success),
            })
    } else {
        HttpResponse::PartialContent()
            .insert_header(("X-Session-ID", session_id))
            .json(StatusResponse {
                status: format!("Inserted {}, errors: {:?}", success, errors),
            })
    }
}


#[get("/api/get")]
async fn get_value(
    sessions: Data<SessionManager>,
    query: web::Query<KeyValueRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    server_info(
        get_client_ip(&http_req).as_str(),
        http_req.method().as_str(),
        http_req.path(),
    );
    let session_id = get_session_from_header(&http_req);
    format_session_id(&session_id);
    format_header(&http_req, PRINT_HEADER.clone());

    let (session_id, session) = get_or_create_session(sessions, Option::from(session_id)).await;
    let mut store = session.store.lock().await;

    let kv_store = match store.as_mut() {
        Some(store) => store,
        None => {
            return HttpResponse::BadRequest().json(StatusResponse {
                status: "No database open".to_string(),
            });
        }
    };

    match kv_store.get(query.key.as_bytes()) {
        Ok(Some(value)) => {
            let value_str = String::from_utf8(value).unwrap_or_else(|_| "<BINARY>".to_string());
            *session.last_active.lock().await = Instant::now();

            HttpResponse::Ok()
                .insert_header(("X-Session-ID", session_id))
                .json(KeyValueResponse {
                    key: query.key.clone(),
                    value: Some(value_str),
                })
        }
        Ok(None) => {
            *session.last_active.lock().await = Instant::now();
            HttpResponse::Ok()
                .insert_header(("X-Session-ID", session_id))
                .json(KeyValueResponse {
                    key: query.key.clone(),
                    value: None,
                })
        }
        Err(e) => {
            error!("Get error: {}", e);
            HttpResponse::InternalServerError().json(StatusResponse {
                status: format!("Error: {}", e),
            })
        }
    }
}


#[post("/api/del")]
async fn delete_value(
    sessions: Data<SessionManager>,
    req: Json<KeyValueRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    server_info(
        get_client_ip(&http_req).as_str(),
        http_req.method().as_str(),
        http_req.path(),
    );

    let session_id = get_session_from_header(&http_req);
    format_session_id(&session_id);
    format_header(&http_req, PRINT_HEADER.clone());

    let (session_id, session) = get_or_create_session(sessions, Option::from(session_id)).await;

    let mut store = session.store.lock().await;

    let kv_store = match store.as_mut() {
        Some(store) => store,
        None => {
            return HttpResponse::BadRequest().json(StatusResponse {
                status: "No database open".to_string(),
            });
        }
    };

    match kv_store.delete(req.key.as_bytes()) {
        Ok(_) => {
            *session.last_active.lock().await = Instant::now();
            HttpResponse::Ok()
                .insert_header(("X-Session-ID", session_id))
                .json(StatusResponse {
                    status: "Key deleted".to_string(),
                })
        }
        Err(e) => {
            error!("Delete error: {}", e);
            HttpResponse::InternalServerError().json(StatusResponse {
                status: format!("Error: {}", e),
            })
        }
    }
}


#[get("/api/id/get")]
async fn get_identifier(sessions: Data<SessionManager>, http_req: HttpRequest) -> impl Responder {
    server_info(
        get_client_ip(&http_req).as_str(),
        http_req.method().as_str(),
        http_req.path(),
    );
    let session_id = get_session_from_header(&http_req);
    format_session_id(&session_id);
    format_header(&http_req, PRINT_HEADER.clone());

    let (session_id, session) = get_or_create_session(sessions, Option::from(session_id)).await;

    let store = session.store.lock().await;

    let kv_store = match store.as_ref() {
        Some(store) => store,
        None => {
            return HttpResponse::BadRequest().json(StatusResponse {
                status: "No database open".to_string(),
            });
        }
    };

    *session.last_active.lock().await = Instant::now();

    HttpResponse::Ok()
        .insert_header(("X-Session-ID", session_id))
        .json(IdentifierResponse {
            identifier: kv_store.get_identifier().to_string(),
        })
}


#[post("/api/id/set")]
async fn set_identifier(
    sessions: Data<SessionManager>,
    req: Json<IdentifierRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    server_info(
        get_client_ip(&http_req).as_str(),
        http_req.method().as_str(),
        http_req.path(),
    );
    let session_id = get_session_from_header(&http_req);
    format_session_id(&session_id);
    format_header(&http_req, PRINT_HEADER.clone());

    let (session_id, session) = get_or_create_session(sessions, Option::from(session_id)).await;

    let mut store = session.store.lock().await;

    let kv_store = match store.as_mut() {
        Some(store) => store,
        None => {
            return HttpResponse::BadRequest().json(StatusResponse {
                status: "No database open".to_string(),
            });
        }
    };

    match kv_store.set_identifier(&req.identifier) {
        Ok(_) => {
            *session.last_active.lock().await = Instant::now();
            HttpResponse::Ok()
                .insert_header(("X-Session-ID", session_id))
                .json(StatusResponse {
                    status: "Identifier updated".to_string(),
                })
        }
        Err(e) => {
            error!("Set identifier error: {}", e);
            HttpResponse::InternalServerError().json(StatusResponse {
                status: format!("Error: {}", e),
            })
        }
    }
}


#[get("/api/current")]
async fn get_current(sessions: Data<SessionManager>, http_req: HttpRequest) -> impl Responder {
    server_info(
        get_client_ip(&http_req).as_str(),
        http_req.method().as_str(),
        http_req.path(),
    );
    let session_id = get_session_from_header(&http_req);
    format_session_id(&session_id);
    format_header(&http_req, PRINT_HEADER.clone());

    let (session_id, session) = get_or_create_session(sessions, Option::from(session_id)).await;

    *session.last_active.lock().await = Instant::now();

    let path = session
        .current_path
        .lock()
        .await
        .clone()
        .unwrap_or_default(); // 修改这里

    HttpResponse::Ok()
        .insert_header(("X-Session-ID", session_id))
        .json(PathRequest { path })
}


#[get("/api/compact")]
async fn compact_db(sessions: Data<SessionManager>, http_req: HttpRequest) -> impl Responder {
    server_info(
        get_client_ip(&http_req).as_str(),
        http_req.method().as_str(),
        http_req.path(),
    );

    let session_id = get_session_from_header(&http_req);
    format_session_id(&session_id);
    format_header(&http_req, PRINT_HEADER.clone());

    let (session_id, session) = get_or_create_session(sessions, Option::from(session_id)).await;

    let mut store = session.store.lock().await;

    let kv_store = match store.as_mut() {
        Some(store) => store,
        None => {
            return HttpResponse::BadRequest().json(StatusResponse {
                status: "No database open".to_string(),
            });
        }
    };

    match kv_store.compact() {
        Ok(_) => {
            *session.last_active.lock().await = Instant::now();
            HttpResponse::Ok()
                .insert_header(("X-Session-ID", session_id))
                .json(StatusResponse {
                    status: "Database compacted".to_string(),
                })
        }
        Err(e) => {
            error!("Compact error: {}", e);
            HttpResponse::InternalServerError().json(StatusResponse {
                status: format!("Error: {}", e),
            })
        }
    }
}


#[post("/api/execute")]
async fn execute_command(
    sessions: Data<SessionManager>,
    command: String,
    http_req: HttpRequest,
) -> impl Responder {
    server_info(
        get_client_ip(&http_req).as_str(),
        http_req.method().as_str(),
        http_req.path(),
    );

    let session_id = get_session_from_header(&http_req);
    format_session_id(&session_id);
    format_header(&http_req, PRINT_HEADER.clone());

    let (session_id, session) = get_or_create_session(sessions, Option::from(session_id)).await;

    let mut store = session.store.lock().await;

    let kv_store = match store.as_mut() {
        Some(store) => store,
        None => {
            return HttpResponse::BadRequest().json(StatusResponse {
                status: "No database open".to_string(),
            });
        }
    };

    // 复用shell中的命令解析逻辑
    let mut arr: Vec<String> = Vec::new();
    for cmd in command.split(";") {
        if cmd == "" || cmd == " " {
            continue;
        }
        // println!("cmd: {}", cmd);
        arr.push(format!(
            "\"{}: {}{}\"",
            cmd,
            parse_and_execute(cmd, kv_store)
                .await
                .unwrap_or_else(|e| format!("Error: {}", e)),
            ";"
        ));
    }
    *session.last_active.lock().await = Instant::now();

    let result = arr.join(" ");
    // println!("RESULT: {}", result);

    HttpResponse::Ok()
        .insert_header(("X-Session-ID", session_id))
        .json(StatusResponse { status: result })
}

// 命令解析和执行
async fn parse_and_execute(command: &str, store: &mut KVStore) -> Result<String> {
    let command = command.trim();

    // 解析PUT命令
    if let Ok(kvs) = parse_put_command(command) {
        let mut success = 0;
        for (key, value) in kvs {
            store.put(key.as_bytes(), value.as_bytes())?;
            success += 1;
        }
        return Ok(format!("Inserted {} key-value pairs", success));
    }

    // 解析GET命令
    if let Ok(key) = parse_get_command(command) {
        if let Some(value) = store.get(key.as_bytes())? {
            return Ok(String::from_utf8(value).unwrap_or_else(|_| "<BINARY>".to_string()));
        }
        return Ok("Key not found".to_string());
    }

    // 解析DELETE命令
    if let Ok(key) = parse_delete_command(command) {
        store.delete(key.as_bytes())?;
        return Ok("Key deleted".to_string());
    }

    // 解析COMPACT命令
    if parse_compact(command).is_ok() {
        store.compact()?;
        return Ok("Database compacted".to_string());
    }

    // 解析IDENTIFIER GET命令
    if parse_identifier_get(command).is_ok() {
        return Ok(store.get_identifier().to_string());
    }

    // 解析IDENTIFIER SET命令
    if let Ok(id) = parse_identifier_set(command) {
        store.set_identifier(&id)?;
        return Ok(format!("Identifier set to '{}'", id));
    }

    Err(anyhow!("Unknown command"))
}

// 启动服务器
pub async fn run_server() -> Result<()> {
    let args = Args::parse();
    PRINT_HEADER
        .set(args.header)
        .expect("Global flag init succeed.");
    // println!("{:?}", PRINT_HEADER);

    let config = load_config()?;

    let sessions = init_session_manager();

    if !is_local_port_available(config.host.clone(), config.port.clone()) {
        return Err(anyhow::anyhow!(
            " * Port `{}` on Host `{}` is already in use.",
            config.port,
            config.host
        ));
    }

    println!(" * Starting Wind-KVStore Server...");
    if config.host == "0.0.0.0" {
        println!(" * Server start on: http://{}:{}", "127.0.0.1", config.port);
        println!(
            " * Server start on: http://{}:{}",
            get_lan_ip().unwrap().to_string(),
            config.port
        );
    } else {
        println!(" * Server start on: http://{}:{}", config.host, config.port);
    }
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(sessions.clone()))
            .service(index)
            .service(open_db)
            .service(close_db)
            .service(put_value)
            .service(get_value)
            .service(delete_value)
            .service(get_identifier)
            .service(set_identifier)
            .service(get_current)
            .service(compact_db)
            .service(execute_command)
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await?;

    Ok(())
}
