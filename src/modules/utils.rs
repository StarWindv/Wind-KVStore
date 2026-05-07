use std::env;
use std::net::{SocketAddr, TcpListener, IpAddr};
use std::sync::OnceLock;
use actix_web::HttpRequest;
use chrono::Local;
use if_addrs::get_if_addrs;

pub fn output_title(is_server: Option<bool>) {
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


fn get_formatted_time() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn server_info(
    ip: &str,
    method: &str,
    route: &str,
)
{
    let time: String = get_formatted_time();
    let output: String = format!("[{}] [{}] [{}] [{}]", ip, time, method, route);
    println!("{}", output);
}

fn get_header_value<'a>(
    req: &'a HttpRequest,
    header_name: &str) -> Option<&'a str>
{
    req.headers().get(header_name)?.to_str().ok()
}

pub fn get_client_ip(req: &HttpRequest) -> String {
    if let Some(ip) = get_header_value(&req, "CF-Connecting-IP") {
        return ip.to_string();
    }

    if let Some(ip) = get_header_value(&req, "X-Forwarded-For") {
        let first_ip = ip.split(',').next().unwrap_or(ip).trim();
        return first_ip.to_string();
    }

    let conn_info = req.connection_info();
    match conn_info.peer_addr() {
        Some(ip_str) => {
            match ip_str.parse::<SocketAddr>() {
                Ok(addr) => addr.ip().to_string(),
                Err(_) =>  ip_str.to_string(),
            }
        }
        None => "Unknown".to_string(),
    }
}

pub fn format_header(req: &HttpRequest, output: OnceLock<bool>) {
    if let Some (flag) = output.get() {
        if *flag {
            println!(" * Receive Headers: ");
            for (name, value) in req.headers() {
                println!("   - {}: {}", name, value.to_str().unwrap_or("Unknown"));
            }
        }
    }
}

pub fn get_session_from_header(http_req: &HttpRequest) -> String{
    http_req.headers()
        .iter()
        .find(|(name, _)| name.as_str().eq_ignore_ascii_case("x-session-id"))
        .map(|(_, value)| value.to_str().unwrap_or("No-session-id-content"))
        .unwrap_or("No-session-id-field")
        .to_string()
}

pub fn get_lan_ip() -> Option<String> {
    get_if_addrs().ok().and_then(|addrs| {
        addrs.into_iter()
            .filter(|iface| !iface.is_loopback() && iface.ip().is_ipv4())
            .map(|iface| iface.ip().to_string())
            .next()
    })
}

pub fn format_session_id(session_id: &String) {
    println!(" * Session-ID: {} \n ", session_id);
}

pub fn is_local_port_available(host: String, port: u16) -> bool {
    let ip: IpAddr = match host.parse() {
        Ok(ip) => ip,
        Err(_) => return false,
    };
    match TcpListener::bind(SocketAddr::new(ip, port)) {
        Ok(_) => true,
        Err(_) => false
    }
}
