use anyhow::Result;
use tokio::fs;
use tokio::{io::AsyncWriteExt, net::UnixListener};

const SOCKET_PATH: &str = "./socket";

pub async fn start_status_server() -> Result<()> {
    // Unlink the socket in case it already exists
    let _ = fs::remove_file(SOCKET_PATH).await;
    let listener = UnixListener::bind(SOCKET_PATH)?;

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                println!("New client detected");

                stream.write(b"nice").await?;
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}
