mod config;
mod service_manager;

use anyhow::Result;
use futures::future;
use futures::future::FusedFuture;
use futures::select;
use futures::FutureExt;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    println!("wumpusd initializing");

    let mut services = HashMap::new();

    loop {
        let config = config::read_config_file()?;

        // Add new services
        for command in &config.run {
            if !services.contains_key(command) {
                let cancellation_token = CancellationToken::new();
                services.insert(
                    command.clone(),
                    (
                        service_manager::start_service(command.clone(), cancellation_token.clone())
                            .boxed()
                            .fuse(),
                        cancellation_token,
                    ),
                );
            }
        }

        // Cancel services that are no longger wanted
        for (cmd, (_, cancellation_token)) in &mut services {
            if !config.run.contains(cmd) {
                cancellation_token.cancel();
            }
        }

        // Clear out dead futures from the services list
        services.retain(|_k, v| !v.0.is_terminated());

        select! {
            _ = future::join_all(services.values_mut().map(|(future, _)| future)).fuse() => {
                println!("All services have stopped / no services listed; exiting");
                return Ok(());
            }
            _ = config::wait_for_config_change().fuse() => {
                println!("Config file changed; reloading");
            }
        }
    }
}
