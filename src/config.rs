use anyhow::Error;
use anyhow::Result;
use futures::StreamExt;
use inotify::{Inotify, WatchMask};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::{fs, io::ErrorKind};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub run: Vec<String>,
}

pub fn get_config_file() -> Result<String> {
    // Allow environment variable to override xdg config location
    if let Ok(path) = env::var("COGD_CONFIG_FILE") {
        return Ok(path);
    }

    let xdg_dirs = xdg::BaseDirectories::new()?;
    let config_path = xdg_dirs.get_config_home().join("cogd.yml");
    Ok(String::from(config_path.to_string_lossy()))
}

pub fn read_config_file() -> Result<Config> {
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
    let config = serde_yaml::from_str(&config_contents?)?;

    Ok(config)
}

pub async fn wait_for_config_change() -> Result<()> {
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
