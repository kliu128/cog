mod config;

use anyhow::{Error, Result};
use futures::future;
use futures::select;
use futures::FutureExt;
use futures::StreamExt;
use inotify::{Inotify, WatchMask};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::{fs, io::ErrorKind};
use tokio_util::sync::CancellationToken;

mod service_manager;

fn get_config_file() -> Result<String> {
    let xdg_dirs = xdg::BaseDirectories::new()?;
    let config_path = xdg_dirs.get_config_home().join("wumpusd.yml");
    Ok(String::from(config_path.to_string_lossy()))
}

fn read_config_file() -> Result<config::Config> {
    let config_path = get_config_file()?;
    println!("Loading config from {}", &config_path);

    let mut config_contents = fs::read_to_string(&config_path);
    match config_contents {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let mut file = File::create(&config_path)?;
            file.write_all(include_bytes!("config.yml"))?;
            config_contents = fs::read_to_string(&config_path);
        }
        Err(e) => return Err(Error::new(e)),
        Ok(ref _contents) => {}
    }
    let config: config::Config = serde_yaml::from_str(&config_contents?)?;

    Ok(config)
}

async fn wait_for_config_change() -> Result<()> {
    let mut inotify = Inotify::init().expect("Error while initializing inotify instance");

    // Watch for modify and close events.
    inotify
        .add_watch(get_config_file()?, WatchMask::MODIFY | WatchMask::CLOSE)
        .expect("Failed to add file watch");

    // Read events that were added with `add_watch` above.
    let buffer = [0; 1024];
    let mut events = inotify.event_stream(buffer)?;

    events.next().await;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("wumpusd initializing");

    let mut services = HashMap::new();

    loop {
        let config = read_config_file()?;
        for command in &config.run {
            if !services.contains_key(command) {
                let cancellation_token = CancellationToken::new();
                services.insert(
                    command.clone(),
                    (
                        service_manager::start_service(command.clone(), cancellation_token.clone())
                            .boxed(),
                        cancellation_token,
                    ),
                );
            }
        }

        for (cmd, (_, cancellation_token)) in &mut services {
            if !config.run.contains(cmd) {
                cancellation_token.cancel();
            }
        }

        select! {
            _ = future::join_all(services.values_mut().map(|(future, _)| future)).fuse() => {
                panic!();
            }
            _ = wait_for_config_change().fuse() => {
                println!("Config file changed; reloading");
            }
        }
    }
}
