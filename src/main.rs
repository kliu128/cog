mod config;
mod server;
mod service_manager;

use anyhow::Result;
use futures::{select, FutureExt, StreamExt};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use tokio_util::sync::CancellationToken;

/// Handle SIG* shutdown signals and gracefully stop
async fn handle_signals(service_canceller: CancellationToken) -> Result<()> {
    let signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
    let handle = signals.handle();

    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT | SIGQUIT => {
                // Shutdown the system
                service_canceller.cancel();
                handle.close();
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("cogd initializing");

    let service_canceller = CancellationToken::new();
    tokio::spawn(handle_signals(service_canceller.clone()));

    select! {
        r = service_manager::start_service_manager(service_canceller).fuse() => r,
        r = server::start_status_server().fuse() => r
    }
}
