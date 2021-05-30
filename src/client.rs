use std::{io::Read, os::unix::net::UnixStream};

use anyhow::Result;

mod service_statuses;

use crate::service_statuses::ServiceStatuses;

fn main() -> Result<()> {
    let mut stream = UnixStream::connect("./socket")?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    let status: ServiceStatuses = serde_json::from_str(&response)?;

    for command in status.names {
        println!("RUNNING: {}", command);
    }

    Ok(())
}
