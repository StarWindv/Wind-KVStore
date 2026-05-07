use anyhow::{anyhow, Result};
use winnow::{
    ascii::multispace0,
    combinator::delimited,
    token::take_while,
    Parser,
};
use std::env;
use std::net::{SocketAddr, TcpListener, IpAddr};
use std::sync::OnceLock;
use actix_web::HttpRequest;
use chrono::Local;
use if_addrs::get_if_addrs;


#[derive(Debug)]
pub enum ParsedGetCommand {
    All,
    Key(String),
}


fn keyword_ic<'a>(kw: &'static str) -> impl Parser<&'a str, &'a str, winnow::error::ContextError> {
    take_while(1.., |c: char| c.is_alphabetic())
        .verify(move |s: &str| s.eq_ignore_ascii_case(kw))
}

fn quoted_str<'a>() -> impl Parser<&'a str, &'a str, winnow::error::ContextError> {
    delimited('"', take_while(0.., |c| c != '"'), '"')
}

fn swallow_ws(input: &mut &str) {
    let _ = multispace0::<_, winnow::error::ContextError>.parse_next(input);
}


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


pub fn parse_put_command(command: &str) -> Result<Vec<(String, String)>> {
    let mut input = command.trim();

    keyword_ic("PUT")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid PUT command: expected PUT keyword"))?;

    swallow_ws(&mut input);

    let mut pairs = Vec::new();
    loop {
        swallow_ws(&mut input);
        let key = quoted_str()
            .parse_next(&mut input)
            .map_err(|_| anyhow!("Invalid PUT command: expected quoted key"))?;

        swallow_ws(&mut input);
        if !input.starts_with(':') {
            return Err(anyhow!("Invalid PUT command: expected ':' separator"));
        }
        input = &input[1..];

        swallow_ws(&mut input);
        let value = quoted_str()
            .parse_next(&mut input)
            .map_err(|_| anyhow!("Invalid PUT command: expected quoted value"))?;

        pairs.push((key.to_string(), value.to_string()));

        swallow_ws(&mut input);
        if input.starts_with(',') {
            input = &input[1..];
            continue;
        }
        break;
    }

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid PUT command: unexpected trailing input"));
    }
    if pairs.is_empty() {
        return Err(anyhow!("Invalid PUT command: no key-value pairs found"));
    }

    Ok(pairs)
}


pub fn parse_get_command(command: &str) -> Result<ParsedGetCommand> {
    let mut input = command.trim();

    keyword_ic("GET")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid GET command"))?;
    swallow_ws(&mut input);
    keyword_ic("WHERE")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid GET command: expected WHERE"))?;
    swallow_ws(&mut input);
    keyword_ic("KEY")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid GET command: expected KEY"))?;
    swallow_ws(&mut input);

    if !input.starts_with('=') {
        return Err(anyhow!("Invalid GET command: expected '='"));
    }
    input = &input[1..];
    swallow_ws(&mut input);

    if input.starts_with('*') {
        input = &input[1..];
        swallow_ws(&mut input);
        if !input.is_empty() {
            return Err(anyhow!("Invalid GET command: unexpected trailing input after '*'"));
        }
        return Ok(ParsedGetCommand::All);
    }

    let key = quoted_str()
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid GET command: expected quoted key or '*'"))?;

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid GET command: unexpected trailing input"));
    }

    Ok(ParsedGetCommand::Key(key.to_string()))
}


pub fn parse_delete_command(command: &str) -> Result<String> {
    let mut input = command.trim();

    keyword_ic("DEL")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid DELETE command: expected DEL"))?;
    swallow_ws(&mut input);
    keyword_ic("WHERE")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid DELETE command: expected WHERE"))?;
    swallow_ws(&mut input);
    keyword_ic("KEY")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid DELETE command: expected KEY"))?;
    swallow_ws(&mut input);

    if !input.starts_with('=') {
        return Err(anyhow!("Invalid DELETE command: expected '='"));
    }
    input = &input[1..];
    swallow_ws(&mut input);

    let key = quoted_str()
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid DELETE command: expected quoted key"))?;

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid DELETE command: unexpected trailing input"));
    }

    Ok(key.to_string())
}


pub fn parse_identifier_get(command: &str) -> Result<()> {
    let mut input = command.trim();

    keyword_ic("IDENTIFIER")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid IDENTIFIER command"))?;
    swallow_ws(&mut input);
    keyword_ic("GET")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid IDENTIFIER command: expected GET"))?;

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid IDENTIFIER GET command: unexpected trailing input"));
    }

    Ok(())
}


pub fn parse_identifier_set(command: &str) -> Result<String> {
    let mut input = command.trim();

    keyword_ic("IDENTIFIER")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid IDENTIFIER command"))?;
    swallow_ws(&mut input);
    keyword_ic("SET")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid IDENTIFIER command: expected SET"))?;
    swallow_ws(&mut input);

    let id = quoted_str()
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid IDENTIFIER SET command: expected quoted identifier"))?;

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid IDENTIFIER SET command: unexpected trailing input"));
    }

    Ok(id.to_string())
}


pub fn parse_compact(command: &str) -> Result<()> {
    let mut input = command.trim();

    keyword_ic("COMPACT")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid COMPACT command"))?;

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid COMPACT command: unexpected trailing input"));
    }

    Ok(())
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


#[allow(unused)]
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
        Ok(_) => true,
        Err(_) => false
    }
}
