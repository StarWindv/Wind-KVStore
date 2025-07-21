// src/main.rs
mod kvstore;
mod shell;

use anyhow::Result;
use shell::Shell;

fn main() -> Result<()> {
    let mut shell = Shell::new();
    shell.run()
}
