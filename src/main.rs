use std::{
    ffi::CStr,
    fs::File,
    os::unix::io::AsRawFd,
    io::prelude::*,
    net::TcpListener
};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Map, Number, Value};

struct GamepadInfo {
    axes: u8,
    buttons: u8,
    name: String,
}

#[repr(C)]
struct JsEvent {
    time: u32,
    value: i16,
    kind: u8,
    number: u8,
}

// values taken from `/usr/include/linux/joystick.h`
const JSIOCG_MAGIC: u8 = b'j';
const JSIOCGAXES: u8 = 0x11;
const JSIOCGBUTTONS: u8 = 0x12;
const JSIOCGNAME: u8 = 0x13;

// create the ioctl calls that we need to get gamepad metadata
nix::ioctl_read!(get_num_axes, JSIOCG_MAGIC, JSIOCGAXES, u8);
nix::ioctl_read!(get_num_buttons, JSIOCG_MAGIC, JSIOCGBUTTONS, u8);
nix::ioctl_read_buf!(get_controller_name, JSIOCG_MAGIC, JSIOCGNAME, u8);

fn main() -> Result<()> {
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
            // todo

        } else {
            println!("No bytes from the server! Quitting...");
            break;
        }
    }

    Ok(())
}

fn get_gamepad_info() -> Result<GamepadInfo> {
    let mut num_axes: u8 = 0;
    let mut num_buttons: u8 = 0;
    let mut gamepad_name = [0u8; 128];

    let file = File::open("/dev/input/js0").context("No controller detected")?;
    let fd = file.as_raw_fd();

    unsafe {
        get_num_axes(fd, &mut num_axes).context("Couldn't get gamepad axes")?;
        get_num_buttons(fd, &mut num_buttons).context("Couldn't get gamepad buttons")?;
        get_controller_name(fd, &mut gamepad_name).context("Couldn't get gamepad name")?;
    }

    let name_len = gamepad_name.iter()
        .position(|&ch| ch == b'\0')
        .context("Gamepad name isn't nul-terminated somehow???")? + 1;

    let parsed_name: String = CStr::from_bytes_with_nul(&gamepad_name[..name_len])?
        .to_string_lossy()
        .into();
    
    Ok(
        GamepadInfo {
        axes: num_axes,
        buttons: num_buttons,
        name: parsed_name,
    })
}