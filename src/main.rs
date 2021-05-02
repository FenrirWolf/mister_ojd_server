use std::{
    ffi::CStr,
    fs::File,
    os::unix::io::AsRawFd,
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

use anyhow::{Context, Result};
use serde_json::{json, Map, Number, Value};

struct Gamepad {
    name: String,
    buttons: Vec<bool>,
    axes: Vec<f64>,
}

#[repr(C)]
struct js_event {
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
nix::ioctl_read_buf!(get_gamepad_name, JSIOCG_MAGIC, JSIOCGNAME, u8);

fn main() -> Result<()> {
    let mut stream = connect_to_ojd()?;
    let mut gamepad = get_gamepad_info()?;
    let mut recv_buf = [0u8; 26];

    loop {
        if stream.read(&mut recv_buf)? > 0 {
            read_gamepad_events(&mut gamepad)?;
            let payload = build_json_payload(&mut gamepad);
            write!(&mut stream, "{}#{}", payload.len(), payload)?;
        } else {
            println!("No bytes from the server! Quitting...");
            break;
        }
    }

    Ok(())
}

fn connect_to_ojd() -> Result<TcpStream> {
    let mut ip_addr = local_ipaddress::get().context("Unable to retrieve local IP address")?;
    ip_addr.push_str(":56709");
    
    println!("binding to {}...", ip_addr);
    let listener = TcpListener::bind(ip_addr).context("Unable to bind to local IP address")?;

    println!("accepting...");
    let (stream, _) = listener.accept().context("Unable to accept TCP connection")?;

    println!("Connected!");
    Ok(stream)
}

fn get_gamepad_info() -> Result<Gamepad> {
    let mut num_axes: u8 = 0;
    let mut num_buttons: u8 = 0;
    let mut gamepad_name = [0u8; 128];

    let file = File::open("/dev/input/js0").context("No gamepad detected")?;
    let fd = file.as_raw_fd();

    unsafe {
        get_num_axes(fd, &mut num_axes).context("Couldn't get gamepad axes")?;
        get_num_buttons(fd, &mut num_buttons).context("Couldn't get gamepad buttons")?;
        get_gamepad_name(fd, &mut gamepad_name).context("Couldn't get gamepad name")?;
    }

    let name_len = gamepad_name.iter()
        .position(|&ch| ch == b'\0')
        .context("Gamepad name isn't nul-terminated somehow???")? + 1;

    let parsed_name: String = CStr::from_bytes_with_nul(&gamepad_name[..name_len])
        .context("Failed to parse gamepad name")?
        .to_string_lossy()
        .into();
    
    Ok(
        Gamepad {
        name: parsed_name,
        buttons: vec![false; num_buttons as usize],
        axes: vec![0f64; num_axes as usize],
    })
}

fn read_gamepad_events(gamepad: &mut Gamepad) -> Result<()> {
    let mut raw = [0u8; 8];

    let mut file = File::open("/dev/input/js0").context("No gamepad detected")?;

    for button in &mut gamepad.buttons {
        file.read(&mut raw).context("Couldn't read js_event")?;
        let event: js_event = unsafe { std::mem::transmute(raw) };
        *button = event.value != 0;
    }

    for axis in &mut gamepad.axes {
        file.read(&mut raw).context("Couldn't read js_event")?;
        let event: js_event = unsafe { std::mem::transmute(raw) };
        *axis = event.value as f64 / i16::MAX as f64;
    }

    Ok(())
}

fn build_json_payload(gamepad: &Gamepad) -> String {
    let axes_json: Vec<Value> = gamepad.axes.iter()
        .map(|&val| Value::Number(Number::from_f64(val).unwrap_or(0.into())))
        .collect();

    let buttons_json: Vec<Map<String, Value>> = gamepad.buttons.iter()
        .map(|&val| {
            let mut map = Map::new();
            map.insert(String::from("pressed"), val.into());
            map.insert(String::from("value"), val.into());
            map
        })
        .collect();

    let payload = json!(
        {
            "axes": axes_json,
            "buttons": buttons_json,
            "connected": true,
            "id": gamepad.name,
            "index": 0,
            "mapping": "",
            "timestamp": 0,
        }
    );

    payload.to_string()
}
