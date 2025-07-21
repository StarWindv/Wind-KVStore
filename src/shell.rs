// src/shell.rs
use crate::kvstore::KVStore;
use anyhow::{anyhow, Result};
use linefeed::{Interface, ReadResult};
use regex::Regex;
use std::path::Path;
use std::process::Command;
use std::env::consts::OS;
use std::env;


fn output_tile() {
    let version: &str = env!("CARGO_PKG_VERSION");
    let compile_time: &str = env!("BUILD_TIME");
    let version_info = format!("v{} [compiled {}]\n", version, compile_time);
    let title =
                            "\n".to_owned()                                            +
                            "Welcome to Wind-KVStore!\n"                               +
                            "\n"                                                       +
                          &*"██╗    ██╗    ██╗    ███╗   ██╗    ██████╗ \n".to_owned() +
                            "██║    ██║    ██║    ████╗  ██║    ██╔══██╗\n"            +
                            "██║ █╗ ██║    ██║    ██╔██╗ ██║    ██║  ██║\n"            +
                            "██║███╗██║    ██║    ██║╚██╗██║    ██║  ██║\n"            +
                            "╚███╔███╔╝    ██║    ██║ ╚████║    ██████╔╝\n"            +
                            " ╚══╝╚══╝     ╚═╝    ╚═╝  ╚═══╝    ╚═════╝ \n"            +
                            "\n"                                                       +
                          &*version_info                                               +
                            "Type \".help;\" for usage hints.";
    println!("{}", title);
}


const HELP_MSG: &str = concat!(
                                "Here is KVStore's help:\n",
                                "Notes: All commands (including dot commands) must end with a semicolon.",
                                "\n",
                                "SHELL COMMAND:\n",
                                "    .help                    Show this message.\n",
                                "    .quit                    Exit KVStore shell.\n",
                                "    .open <path>             Open kvstore at the specified path.\n",
                                "    .close                   Close current kvstore.\n",
                                "    .clear                   Execute screen clear command.\n",
                                "    .title                   Show KVStore's startup information.\n",
                                "\n",
                                "KV OPERATOR:\n",
                                "    PUT \"KEY\":\"VALUE\"        Insert key-value pairs into an active database.\n",
                                "    GET WHERE KEY=\"MyKey\"    Retrieve the value associated with key 'MyKey.\n",
                                "    DEL WHERE KEY=\"MyKey\"    Remove the key-value pair 'MyKey'.\n",
                                "    COMPACT                  Compress the currently activity KV database.\n",
                                "\n",
                                "METADATA OPERATOR:\n",
                                "    IDENTIFIER:\n",
                                "        GET                  Output the current KV database's identifier.\n",
                                "        SET \"<new id>\"       Set a new id for the current KV database.\n",
                                "\n",
                                "    If you have any questions or encounter program errors,\n",
                                "    please contact our technical support at:\n",
                                "    \x1b[4mstarwindv.stv@gmail.com\x1b[0m"
);


fn clear_scene() {
    if OS == "windows" {
        Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
            .expect("Failed to execute command");
    } else {
        Command::new("clear")
            .status()
            .expect("Failed to execute command");
    }
}


pub struct Shell {
    store: Option<KVStore>,
    current_path: Option<String>,
}


impl Shell {
    pub fn new() -> Self {
        Shell {
            store: None,
            current_path: None,
        }
    }


    pub fn run(&mut self) -> Result<()> {
        output_tile();

        let reader = Interface::new("kvstore-shell")?;
        reader.set_prompt(self.get_prompt().as_str())?;

        let length = self.get_prompt().chars().count();
        let wait_prompt = ". ".repeat(length/2-1) + "> ";

        while let ReadResult::Input(input) = reader.read_line()? {
            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            // 处理多行输入直到分号
            let mut command = input.to_string();
            while !command.ends_with(';') {
                reader.set_prompt(wait_prompt.as_str())?;
                if let ReadResult::Input(cont) = reader.read_line()? {
                    command.push_str(cont.trim());
                } else {
                    break;
                }
            }

            // 移除结尾分号
            if command.ends_with(';') {
                command.pop();
            }

            match self.execute_command(&command) {
                Ok(output) => {
                    if !output.is_empty() {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }

            reader.set_prompt(self.get_prompt().as_str())?;
        }

        Ok(())
    }


    fn get_prompt(&self) -> String {
        match &self.current_path {
            Some(path) => {
                // 将String转换为Path
                let ppath = Path::new(path);
                // 提取文件名，如果没有则使用完整路径
                let file_name = ppath.file_name()
                    .and_then(|os_str| os_str.to_str())
                    .unwrap_or(path);
                
                format!("\n{} > ", file_name)
            
            }
            None => "\nKVStore > ".to_string(),
        }
    }


    fn execute_command(&mut self, command: &str) -> Result<String> {
        let command = command.trim();
        
        // 处理元命令
        if command.starts_with('.') {
            return self.handle_meta_command(command);
        }
        
        // 检查数据库是否打开
        if self.store.is_none() {
            return Err(anyhow!("No database open. Use .open first"));
        }
        
        // 解析命令
        if let Ok(cmd) = self.parse_put_command(command) {
            return self.handle_put_command(cmd);
        }
        
        if let Ok(key) = self.parse_get_command(command) {
            return self.handle_get_command(key);
        }
        
        if let Ok(key) = self.parse_delete_command(command) {
            return self.handle_delete_command(key);
        }
        
        if let Ok(()) = self.parse_identifier_get(command) {
            return self.handle_identifier_get();
        }
        
        if let Ok(new_id) = self.parse_identifier_set(command) {
            return self.handle_identifier_set(new_id);
        }
        
        if let Ok(()) = self.parse_compact(command) {
            return self.handle_compact();
        }
        
        Err(anyhow!("Unknown command: {}", command))
    }


    fn handle_meta_command(&mut self, command: &str) -> Result<String> {
        match command {
            ".quit" => std::process::exit(0),

            ".close" => {
                if let Some(store) = self.store.take() {
                    store.close()?;
                    self.current_path = None;
                    Ok("Database closed".to_string())
                } else {
                    Err(anyhow!("No database open"))
                }
            }

            ".help" => {
                println!("{}", HELP_MSG);
                Ok(String::new())
            }

            ".clear" => {
                clear_scene();
                Ok(String::new())
            }

            ".title" => {
                output_tile();
                Ok(String::new())
            }

            cmd if cmd.starts_with(".open") => {
                let path = cmd.trim_start_matches(".open").trim();
                if path.is_empty() {
                    return Err(anyhow!("Usage: .open <path>"));
                }
                self.open_database(path)
            }
            _ => Err(anyhow!("Unknown meta command: {}", command)),
        }
    }


    fn open_database(&mut self, path: &str) -> Result<String> {
        // 关闭当前数据库
        if let Some(store) = self.store.take() {
            store.close()?;
        }
        
        let db_path = Path::new(path);
        let exists = db_path.exists();
        
        if !exists {
            println!("Database does not exist. Create new database? (y/n)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                return Err(anyhow!("Database creation canceled"));
            }
        }
        

        let store = KVStore::open(path, None)?;
        
        self.store = Some(store);
        self.current_path = Some(path.to_string());
        
        Ok(format!(
            "Database {} opened successfully", 
            if exists { "existing" } else { "new" }
        ))
    }


    // PUT 命令解析
    fn parse_put_command(&self, command: &str) -> Result<Vec<(String, String)>> {
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


    fn handle_put_command(&mut self, kvs: Vec<(String, String)>) -> Result<String> {
        let store = self.store.as_mut().ok_or(anyhow!("No database open"))?;
        let length = kvs.len();
        
        for (key, value) in kvs {
            store.put(key.as_bytes(), value.as_bytes())?;
        }
        
        Ok(format!("Inserted {} key-value pairs", length))
    }


    // GET 命令解析
    fn parse_get_command(&self, command: &str) -> Result<String> {
        let re = Regex::new(r#"GET\s+WHERE\s+KEY\s*=\s*"([^"]+)"\s*$"#)?;
        
        if let Some(caps) = re.captures(command) {
            if let Some(key) = caps.get(1) {
                return Ok(key.as_str().to_string());
            }
        }
        
        Err(anyhow!("Invalid GET command format"))
    }


    fn handle_get_command(&mut self, key: String) -> Result<String> {
        let store = self.store.as_mut().ok_or(anyhow!("No database open"))?;
        
        if let Some(value) = store.get(key.as_bytes())? {
            // 尝试转换为字符串，失败则显示十六进制
            match String::from_utf8(value.clone()) {
                Ok(s) => Ok(s),
                Err(_) => {
                    let hex_str = value.iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    Ok(format!("<BINARY DATA: {}>", hex_str))
                }
            }
        } else {
            Ok("Key not found".to_string())
        }
    }


    // DELETE 命令解析
    fn parse_delete_command(&self, command: &str) -> Result<String> {
        let re = Regex::new(r#"DEL\s+WHERE\s+KEY\s*=\s*"([^"]+)"\s*$"#)?;
        
        if let Some(caps) = re.captures(command) {
            if let Some(key) = caps.get(1) {
                return Ok(key.as_str().to_string());
            }
        }
        
        Err(anyhow!("Invalid DELETE command format"))
    }


    fn handle_delete_command(&mut self, key: String) -> Result<String> {
        let store = self.store.as_mut().ok_or(anyhow!("No database open"))?;
        
        if store.get(key.as_bytes())?.is_some() {
            store.delete(key.as_bytes())?;
            Ok("Key deleted".to_string())
        } else {
            Ok("Key not found".to_string())
        }
    }


    // IDENTIFIER GET 命令解析
    fn parse_identifier_get(&self, command: &str) -> Result<()> {
        if command.trim().eq_ignore_ascii_case("IDENTIFIER GET") {
            Ok(())
        } else {
            Err(anyhow!("Invalid IDENTIFIER GET command"))
        }
    }


    fn handle_identifier_get(&self) -> Result<String> {
        let store = self.store.as_ref().ok_or(anyhow!("No database open"))?;
        Ok(store.get_identifier().to_string())
    }


    // IDENTIFIER SET 命令解析
    fn parse_identifier_set(&self, command: &str) -> Result<String> {
        let re = Regex::new(r#"IDENTIFIER\s+SET\s+"([^"]+)"\s*$"#)?;

        if let Some(caps) = re.captures(command) {
            if let Some(id) = caps.get(1) {
                return Ok(id.as_str().to_string());
            }
        }
        
        Err(anyhow!("Invalid IDENTIFIER SET command"))
    }


    fn handle_identifier_set(&mut self, new_id: String) -> Result<String> {
        let store = self.store.as_mut().ok_or(anyhow!("No database open"))?;
        store.set_identifier(&new_id)?;
        Ok(format!("Identifier set to '{}'", new_id))
    }


    // COMPACT 命令解析
    fn parse_compact(&self, command: &str) -> Result<()> {
        if command.trim().eq_ignore_ascii_case("COMPACT") {
            Ok(())
        } else {
            Err(anyhow!("Invalid COMPACT command"))
        }
    }


    fn handle_compact(&mut self) -> Result<String> {
        let store = self.store.as_mut().ok_or(anyhow!("No database open"))?;
        store.compact()?;
        Ok("Database compacted".to_string())
    }
}