// src/shell-main
pub mod kvstore;
mod utils;
pub mod shell;
use anyhow::Result;
use shell::Shell;


fn main() -> Result<()> {
    let mut shell = Shell::new();
    shell.run()
}

