use anyhow::Result;
use wind_kvstore::utils;
use wind_kvstore::types::wind_server::WindServer;


#[actix_web::main]
async fn main() -> Result<()> {
    utils::output_title(Some(true));
    let server = match WindServer::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!(" * Server has exited.");
            std::process::exit(1);
        }
    };
    if let Err(e) = server.run().await {
        eprintln!("{}", e);
        eprintln!(" * Server has exited.");
        std::process::exit(1);
    }
    println!(" * Server has stopped.");
    Ok(())
}
