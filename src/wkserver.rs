use anyhow::{Result};
use wind_kvstore::modules::utils;
use wind_kvstore::modules::server;


#[actix_web::main]
async fn main() -> Result<()> {
    utils::output_title(Option::from(true));
    if let Err(e) = server::run_server().await {
        eprintln!("{}", e);
        eprintln!(" * Server has exited.");
        std::process::exit(1);
    }
    println!(" * Server has stopped.");
    Ok(())
}
