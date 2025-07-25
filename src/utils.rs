use regex::Regex;
use anyhow::anyhow;
use std::env;
use std::net::{SocketAddr, TcpListener, IpAddr};
use std::sync::OnceLock;
use actix_web::HttpRequest;
use chrono::Local;
use if_addrs::get_if_addrs;


pub fn output_tile(is_server: Option<bool>) {
    let is_server = is_server.unwrap_or(false);
    let version: &str = env!("CARGO_PKG_VERSION");
    let compile_time: &str = env!("BUILD_TIME");
    let version_info = format!("v{} [compiled {}]\n", version, compile_time);
    let mut title =
        "\n".to_owned()                                            +
            "Welcome to Wind-KVStore!\n"                               +
            "\n"                                                       +
            "\t       "                                                +
            &*"██╗    ██╗    ██╗    ███╗   ██╗    ██████╗ \n"            +
            "\t       "                                                +
            "██║    ██║    ██║    ████╗  ██║    ██╔══██╗\n"            +
            "\t       "                                                +
            "██║ █╗ ██║    ██║    ██╔██╗ ██║    ██║  ██║\n"            +
            "\t       "                                                +
            "██║███╗██║    ██║    ██║╚██╗██║    ██║  ██║\n"            +
            "\t       "                                                +
            "╚███╔███╔╝    ██║    ██║ ╚████║    ██████╔╝\n"            +
            "\t       "                                                +
            " ╚══╝╚══╝     ╚═╝    ╚═╝  ╚═══╝    ╚═════╝ \n"            +
            "\n"                                                       +
            &*version_info                                               ;
    if !is_server {
        title += "Type \".help\" for usage hints.";
    }

    println!("{}", title);
}


pub fn parse_put_command(command: &str) -> anyhow::Result<Vec<(String, String)>> {
    let re = Regex::new(r#"PUT\s+"([^"]+)":"([^"]+)"(?:\s*,\s*"([^"]+)":"([^"]+)")*\s*$"#)?;

    if let Some(caps) = re.captures(command) {
        let mut kvs = Vec::new();

        // 第一个键值对
        if let (Some(k), Some(v)) = (caps.get(1), caps.get(2)) {
            kvs.push((k.as_str().to_string(), v.as_str().to_string()));
        }

        // 后续键值对
        let mut i = 3;
        while i < caps.len() {
            if let (Some(k), Some(v)) = (caps.get(i), caps.get(i + 1)) {
                kvs.push((k.as_str().to_string(), v.as_str().to_string()));
                i += 2;
            } else {
                break;
            }
        }

        if kvs.is_empty() {
            return Err(anyhow!("Invalid PUT command format"));
        }

        Ok(kvs)
    } else {
        Err(anyhow!("Invalid PUT command format"))
    }
}


pub fn parse_get_command(command: &str) -> anyhow::Result<String> {
    let re = Regex::new(r#"GET\s+WHERE\s+KEY\s*=\s*"([^"]+)"\s*$"#)?;

    if let Some(caps) = re.captures(command) {
        if let Some(key) = caps.get(1) {
            return Ok(key.as_str().to_string());
        }
    }

    Err(anyhow!("Invalid GET command format"))
}


pub fn parse_delete_command(command: &str) -> anyhow::Result<String> {
    let re = Regex::new(r#"DEL\s+WHERE\s+KEY\s*=\s*"([^"]+)"\s*$"#)?;

    if let Some(caps) = re.captures(command) {
        if let Some(key) = caps.get(1) {
            return Ok(key.as_str().to_string());
        }
    }

    Err(anyhow!("Invalid DELETE command format"))
}


pub fn parse_identifier_get(command: &str) -> anyhow::Result<()> {
    if command.trim().eq_ignore_ascii_case("IDENTIFIER GET") {
        Ok(())
    } else {
        Err(anyhow!("Invalid IDENTIFIER GET command"))
    }
}


pub fn parse_identifier_set(command: &str) -> anyhow::Result<String> {
    let re = Regex::new(r#"IDENTIFIER\s+SET\s+"([^"]+)"\s*$"#)?;

    if let Some(caps) = re.captures(command) {
        if let Some(id) = caps.get(1) {
            return Ok(id.as_str().to_string());
        }
    }

    Err(anyhow!("Invalid IDENTIFIER SET command"))
}


pub fn parse_compact(command: &str) -> anyhow::Result<()> {
    if command.trim().eq_ignore_ascii_case("COMPACT") {
        Ok(())
    } else {
        Err(anyhow!("Invalid COMPACT command"))
    }
}


fn get_formatted_time() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}


#[allow(unused)]
pub fn server_info(
    ip: &str,
    method: &str,
    route: &str,
)
{
    /*
    # """
    # [IP] [TIME] [METHOD] [REQUESTS]
    # """
    */
    let time: String = get_formatted_time();
    let output: String = format!("[{}] [{}] [{}] [{}]", ip, time, method, route);
    println!("{}", output);
}


#[allow(unused)]
fn get_header_value<'a>(
    req: &'a HttpRequest,
    header_name: &str) -> Option<&'a str>
{
    req.headers().get(header_name)?.to_str().ok()
}


#[allow(unused)]
pub fn get_client_ip(req: &HttpRequest) -> String {
    // 优先检查 CF-Connecting-IP (Cloudflare 提供的真实 IP 头)
    if let Some(ip) = get_header_value(&req, "CF-Connecting-IP") {
        return ip.to_string();
    }

    // 其次检查 X-Forwarded-For 头
    if let Some(ip) = get_header_value(&req, "X-Forwarded-For") {
        // X-Forwarded-For 可能包含多个 IP，取第一个
        let first_ip = ip.split(',').next().unwrap_or(ip).trim();
        return first_ip.to_string();
    }

    // 如果没有上述头信息，使用远程地址
    let conn_info = req.connection_info();
    match conn_info.peer_addr() {
        Some(ip_str) => {
            // 尝试解析为 SocketAddr 以提取纯 IP 部分
            match ip_str.parse::<SocketAddr>() {
                Ok(addr) => addr.ip().to_string(),
                Err(_) =>  ip_str.to_string(),
            }
        }
        None => "Unknown".to_string(),
    }
}


#[allow(unused)]
pub fn format_header(req: &HttpRequest, output: OnceLock<bool>) {
    // println!("{:?}", output);
    if let Some (flag) = output.get() {
        // println!("flag: {}", flag);
        if *flag {
            println!(" * Receive Headers: ");
            for (name, value) in req.headers() {
                println!("   - {}: {}", name, value.to_str().unwrap_or("Unknown"));
            }
        }
    }
}


#[allow(unused)]
pub fn get_session_from_header(http_req: &HttpRequest) -> String{
    http_req.headers()
        .iter()
        .find(|(name, _)| name.as_str().eq_ignore_ascii_case("x-session-id"))
        .map(|(_, value)| value.to_str().unwrap_or("No-session-id-content"))
        .unwrap_or("No-session-id-field")
        .to_string()
}


#[allow(unused)]
pub fn get_lan_ip() -> Option<String> {
    get_if_addrs().ok().and_then(|addrs| {
        addrs.into_iter()
            // 过滤回环接口和非IPv4地址（按需调整）
            .filter(|iface| !iface.is_loopback() && iface.ip().is_ipv4())
            .map(|iface| iface.ip().to_string())
            .next()
    })
}


#[allow(unused)]
pub fn format_session_id(session_id: &String) {
    println!(" * Session-ID: {} \n ", session_id);
}


#[allow(unused)]
pub fn is_local_port_available(host: String, port: u16) -> bool {
    let ip: IpAddr = match host.parse() {
        Ok(ip) => ip,
        Err(_) => return false,
    };
    match TcpListener::bind(SocketAddr::new(ip, port)) {
        Ok(_) => true,  // 绑定成功说明端口可用
        Err(_) => false // 绑定失败说明端口被占用或无权访问
    }
}
