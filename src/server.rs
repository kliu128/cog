use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::{io::AsyncWriteExt, net::UnixListener};

use crate::service_statuses::ServiceStatuses;

pub fn get_socket_path() -> PathBuf {
    let runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR not set; cannot create socket");
    return PathBuf::from(runtime_dir).join("cogd-ctl");
}

#[allow(dead_code)]
pub async fn start_status_server(service_statuses: Arc<Mutex<ServiceStatuses>>) -> Result<()> {
    // Unlink the socket in case it already exists
    let socket_path = get_socket_path();
    // Ignore error since we can't really do anything about it, and we expect
    // an error if the file doesn't exist (which is fine)
    let _ = fs::remove_file(&socket_path).await;
    let listener = UnixListener::bind(&socket_path)?;

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
