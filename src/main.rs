mod config;

use anyhow::{Error, Result};
// use inotify::{Inotify, WatchMask};
use std::io::prelude::*;
use std::{collections::HashMap, fs::File};
use std::{fs, io::ErrorKind};

fn read_config_file() -> Result<config::Config> {
    let xdg_dirs = xdg::BaseDirectories::new()?;
    let config_path = xdg_dirs.get_config_home().join("wumpusd.yml");
    println!("Loading config from {}", config_path.to_str().unwrap());

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

fn main() -> Result<()> {
    println!("wumpusd initializing");

    let config = read_config_file()?;
    let mut processes: HashMap<&str, i32> = HashMap::new();

    for command in &config.run {
        processes.insert(command, 0);
    }

    println!("{:?}", read_config_file()?);
    // let mut inotify = Inotify::init().expect("Error while initializing inotify system");

    // // Watch config file
    // inotify.add_watch(config_path, WatchMask::MODIFY | WatchMask::CLOSE)?;

    Ok(())
}
