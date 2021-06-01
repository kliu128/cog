use std::{io::Read, os::unix::net::UnixStream};

use anyhow::Result;
use clap::App;
use console::style;

mod server;
mod service_statuses;

use crate::service_statuses::ServiceStatuses;

fn main() -> Result<()> {
    let _matches = App::new("cog")
        .version("0.1.0")
        .author("Kevin Liu <kevin@kliu.io>")
        .about("cogd controller")
        .get_matches();

    let mut stream = UnixStream::connect(server::get_socket_path())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    let status: ServiceStatuses = serde_json::from_str(&response)?;

    println!("{}", style("Services under management by cogd:").bold());

    for command in status.names {
        println!("- [ {} ] {}", style("RUNNING").green().bold(), command);
    }

    Ok(())
}
