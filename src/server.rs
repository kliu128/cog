use std::sync::Arc;

use anyhow::Result;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::{io::AsyncWriteExt, net::UnixListener};

use crate::service_statuses::ServiceStatuses;

const SOCKET_PATH: &str = "./socket";

pub async fn start_status_server(service_statuses: Arc<Mutex<ServiceStatuses>>) -> Result<()> {
    // Unlink the socket in case it already exists
    let _ = fs::remove_file(SOCKET_PATH).await;
    let listener = UnixListener::bind(SOCKET_PATH)?;

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                println!("New client detected");
                let service_statuses = service_statuses.lock().await;
                let service_statuses = serde_json::to_string(&*service_statuses)?;
                stream.write(service_statuses.as_bytes()).await?;
                stream.shutdown().await?;
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}
