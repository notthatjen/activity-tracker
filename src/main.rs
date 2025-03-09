mod data;
mod db;
mod tracker;
mod cli;
mod reports;
mod monitor;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
