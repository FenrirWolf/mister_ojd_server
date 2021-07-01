use anyhow::{anyhow, Context, Result};
use inotify::{Inotify, EventMask, WatchMask};
use nix::{ioctl_read, ioctl_read_buf};
use serde_json::{json, Value};

use std::{
    ffi::CStr,
    fs::File,
    os::unix::io::AsRawFd,
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

struct Gamepad {
    name: String,
    buttons: Vec<bool>,
    axes: Vec<f64>,
    connected: bool,
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
ioctl_read!(get_num_axes, JSIOCG_MAGIC, JSIOCGAXES, u8);
ioctl_read!(get_num_buttons, JSIOCG_MAGIC, JSIOCGBUTTONS, u8);
ioctl_read_buf!(get_gamepad_name, JSIOCG_MAGIC, JSIOCGNAME, u8);

fn main() {
    loop {
        println!("Waiting for Open Joystick Display");

        let mut socket = match connect_to_ojd() {
            Ok(stream) => {
                println!("Connected to Open Joystick Display");
                stream
            },
            Err(e) => {
                println!("Unable to connect to Open Joystick Display: {}", e);
                continue
            }
        };

        let mut gamepad = Gamepad::new();
        let mut recv_buf = [0u8; 26];

        while let Ok(_) = socket.read(&mut recv_buf) {
            if !gamepad.connected {
                if let Err(e) = gamepad.connect() {
                    println!("Unable to detect gamepad status: {}", e);
                } else {
                    println!("Gamepad connected");
                }
            }

            if let Err(_) = gamepad.update_state() {
                println!("Gamepad disconnected");
            }

            let payload = gamepad.build_json_payload();
            if let Err(e) = write!(&mut socket, "{}#{}", payload.len(), payload) {
                println!("Unable to send payload to Open Joystick Display: {}", e);
            }
        }

        println!("Disconnected from Open Joystick Display");
    }
}

fn connect_to_ojd() -> Result<TcpStream> {
    let mut ip_addr = local_ipaddress::get().context("Unable to retrieve local IP address")?;
    ip_addr.push_str(":56709");
    
    let listener = TcpListener::bind(ip_addr).context("Unable to bind to local IP address")?;

    let (stream, _) = listener.accept().context("Unable to accept TCP connection")?;

    Ok(stream)
}

impl Gamepad {
    fn new() -> Self {
        Self {
            name: String::from(""),
            buttons: vec![],
            axes: vec![],
            connected: false,
        }
    }

    fn connect(&mut self) -> Result<()> {
        self.wait_for_connection()?;

        let mut num_axes: u8 = 0;
        let mut num_buttons: u8 = 0;
        let mut gamepad_name = [0u8; 128];

        let device = File::open("/dev/input/js0").context("No gamepad detected")?;
        let fd = device.as_raw_fd();

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
        
        self.name = parsed_name;
        self.buttons.resize(num_buttons as usize, false);
        self.axes.resize(num_axes as usize, 0 as f64);
        self.connected = true;

        Ok(())
    }

    fn wait_for_connection(&mut self) -> Result<()> {
        // skip the wait if `js0` is already connected
        if let Ok(_) = File::open("/dev/input/js0") {
            return Ok(());
        }

        let mut inotify = Inotify::init()?;
        inotify.add_watch("/dev/input", WatchMask::CREATE)?;

        let mut buf = [0; 1024];
        loop {
            let events = inotify.read_events_blocking(&mut buf)?;

            for event in events {
                if let Some(name) = event.name {
                    if name == "js0" && event.mask == EventMask::CREATE {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn update_state(&mut self) -> Result<()> {
        let mut device = match File::open("/dev/input/js0") {
            Ok(device) => device,
            Err(e) => {
                self.connected = false;
                return Err(anyhow!(e));
            }
        };

        let mut raw = [0u8; 8];

        for button in &mut self.buttons {
            if let Err(e) = device.read(&mut raw) {
                self.connected = false;
                return Err(anyhow!(e))

            }
            let event: js_event = unsafe { std::mem::transmute(raw) };
            *button = event.value != 0;
        }

        for axis in &mut self.axes {
            if let Err(e) = device.read(&mut raw) {
                self.connected = false;
                return Err(anyhow!(e))

            }
            let event: js_event = unsafe { std::mem::transmute(raw) };
            *axis = event.value as f64 / i16::MAX as f64;
        }

        Ok(())
    }

    fn build_json_payload(&mut self) -> String {
        let buttons_json: Vec<Value> = self.buttons.iter()
            .map(|&val| {
                json!({"pressed": val, "value": if val {1} else {0}})
            })
            .collect();

        let payload = json!(
            {
                "axes": self.axes,
                "buttons": buttons_json,
                "connected": self.connected,
                "id": self.name,
                "index": 0,
                "mapping": "",
                "timestamp": 0,
            }
        );

        payload.to_string()
    }
}
