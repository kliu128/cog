mod config;
mod server;
mod service_manager;

use anyhow::Result;
use futures::{select, FutureExt};

#[tokio::main]
async fn main() -> Result<()> {
    println!("cogd initializing");

    select! {
        r = service_manager::start_service_manager().fuse() => r,
        r = server::start_status_server().fuse() => r
    }
}
