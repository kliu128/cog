use std::time::Duration;

use anyhow::Result;
use futures::future::FutureExt;
use futures::select;
use tokio::process::{Child, Command};
use tokio_util::sync::CancellationToken;

/// Starts a process and will automatically auto-restart it until the
/// cancellation token is activated, at which point it will kill the chidl.
pub async fn start_service(cmd: &str, cancellation_token: CancellationToken) -> Result<Child> {
    loop {
        println!("Starting shell command: {}", cmd);
        let mut process = Command::new("sh").args(&["-c", cmd]).spawn()?;

        select! {
            _ = cancellation_token.cancelled().fuse() => {
                println!("Shutting down command {}", cmd);
                process.kill().await?;
                return Ok(process);
            }
            status = process.wait().fuse() => {
                let status = status?;
                println!("Process exited with code {}; restarting in 1 s", status);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn cancellation_works() -> Result<()> {
        let token = CancellationToken::new();
        let future = crate::service_manager::start_service("sleep infinity", token.clone());

        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            token.cancel();
        });

        // This will hang infinitely if cancellation fails
        future.await?;

        Ok(())
    }
}
