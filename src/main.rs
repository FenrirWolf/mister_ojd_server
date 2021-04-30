use std::{io::prelude::*, net::TcpListener};

use anyhow::{anyhow, Context, Result};
use gilrs_core::{Error as GilrsError, Gilrs, Gamepad};
use serde_json::{json, Map, Number, Value};

fn main() -> Result<()> {
    let gilrs = Gilrs::new().map_err(|err| match err {
        GilrsError::NotImplemented(_) => {
            anyhow!("gilrs is not supported on this system. unable to open gamepad data.")
        }
        GilrsError::Other(e) => anyhow!(e),
    })?;

    let gamepad = find_connected_gamepad(&gilrs).context("No gamepad found")?;

    println!("Gamepad connected: {}", gamepad.name());

    let mut ip_addr = local_ipaddress::get().context("Unable to retrieve local IP address")?;
    ip_addr.push_str(":56709");
    
    println!("binding to {}...", ip_addr);

    let listener = TcpListener::bind(ip_addr).context("Unable to bind to local IP address")?;

    println!("accepting...");

    let (mut stream, _) = listener.accept().context("Unable to accept TCP connection")?;

    println!("Connected!");

    let mut recv_buf = [0u8; 26];

    loop {
        println!("Reading...");
        if stream.read(&mut recv_buf)? > 0 {
            println!("Writing...");
            let payload = build_json_payload(gamepad);
            write!(&mut stream, "{}#{}", payload.len(), payload)?;
        } else {
            println!("No bytes from the server! Quitting...");
            break;
        }
    }

    Ok(())
}

fn find_connected_gamepad(gilrs: &Gilrs) -> Option<&Gamepad> {
    for idx in 0..gilrs.last_gamepad_hint() {
        if let Some(gamepad) = gilrs.gamepad(idx) {
            if gamepad.is_connected() {
                return Some(gamepad)
            }
        }
    }
    None
}

fn build_json_payload(gamepad: &Gamepad) -> String {
    todo!()
}