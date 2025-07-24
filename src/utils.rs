use regex::Regex;
use anyhow::anyhow;
use std::env;


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