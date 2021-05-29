use std::time::Duration;

use anyhow::Result;
use futures::future;
use futures::future::FusedFuture;
use futures::future::FutureExt;
use futures::select;
use std::collections::HashMap;
use tokio::process::Command;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::config;

/// Starts a process and will automatically auto-restart it until the
/// cancellation token is activated, at which point it will kill the chidl.
pub async fn start_service(cmd: String, cancellation_token: CancellationToken) {
    loop {
        println!("Starting shell command: {}", cmd);
        let mut process = Command::new("sh").args(&["-c", &cmd]).spawn().unwrap();

        select! {
            _ = cancellation_token.cancelled().fuse() => {
                println!("Shutting down command {}", cmd);
                process.kill().await.unwrap();
                break;
            }
            status = process.wait().fuse() => {
                let status = status.unwrap();
                println!("Process exited with code {}; restarting in 1 s", status);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub async fn start_service_manager(cancellation_token: CancellationToken) -> Result<()> {
    let mut services = HashMap::new();
    let mut config = config::read_config_file()?;

    let mut cancelled_future = cancellation_token.cancelled().boxed().fuse();

    loop {
        // Add new services
        for command in &config.run {
            if !services.contains_key(command) {
                let cancellation_token = CancellationToken::new();
                services.insert(
                    command.clone(),
                    (
                        start_service(command.clone(), cancellation_token.clone())
                            .boxed()
                            .fuse(),
                        cancellation_token,
                    ),
                );
            }
        }

        // Cancel services that are no longer wanted
        for (cmd, (_, cancellation_token)) in services.iter_mut() {
            if !config.run.contains(cmd) {
                cancellation_token.cancel();
            }
        }

        // Clear out dead futures from the services list
        services.retain(|_k, v| !v.0.is_terminated());

        let values = services.values_mut();
        let mut futures = future::join_all(values.map(|(future, _)| future)).fuse();

        select! {
            _ = futures => {
                println!("All services have stopped / no services listed; exiting");
                return Ok(());
            }
            _ = config::wait_for_config_change().fuse() => {
                println!("Config file changed; reloading");

                config = config::read_config_file()?;
            }
            _ = cancelled_future => {
                println!("Cancelled; shutting down");

                config.run.clear();
                // The next loop will shut down all services
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
        let future =
            crate::service_manager::start_service("sleep infinity".to_string(), token.clone());

        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            token.cancel();
        });

        // This will hang infinitely if cancellation fails
        future.await;

        Ok(())
    }
}
