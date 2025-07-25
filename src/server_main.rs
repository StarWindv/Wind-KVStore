mod config;
mod kvstore;
mod utils;
mod server;

use crate::server::run_server;
use crate::utils::output_tile;
use anyhow::{Result};

#[actix_web::main]
async fn main() -> Result<()> {
    output_tile(Option::from(true));
    if let Err(e) = run_server().await {
        eprintln!("{}", e);
        eprintln!(" * Server has exited.");
        std::process::exit(1);
    }
    println!(" * Server has stopped.");
    Ok(())
}
