use anyhow::Context;
use daemonize::Daemonize;
use inotify::{EventMask, Inotify, WatchMask};
use nix::{ioctl_read, ioctl_read_buf};
use serde_json::{json, Value};

use std::{
    ffi::CStr,
    fs::File,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    os::unix::io::AsRawFd,
};

#[derive(Default)]
struct Gamepad {
    name: String,
    buttons: Vec<f64>,
    axes: Vec<f64>,
    connected: bool,
}

// Linux joystick event struct defined in `/usr/include/linux/joystick.h`
#[allow(non_camel_case_types)]
#[repr(C)]
struct js_event {
    time: u32,
    value: i16,
    kind: u8,
    number: u8,
}

// Joystick IOCTL command values, also from `/usr/include/linux/joystick.h`
const JSIOCG_MAGIC: u8 = b'j';
const JSIOCGAXES: u8 = 0x11;
const JSIOCGBUTTONS: u8 = 0x12;
const JSIOCGNAME: u8 = 0x13;

// Generate rust-friendly wrappers for the linux joystick driver commands
ioctl_read!(get_num_axes, JSIOCG_MAGIC, JSIOCGAXES, u8);
ioctl_read!(get_num_buttons, JSIOCG_MAGIC, JSIOCGBUTTONS, u8);
ioctl_read_buf!(get_gamepad_name, JSIOCG_MAGIC, JSIOCGNAME, u8);

fn main() {
    if let Err(e) = daemonize_me() {
        panic!("Unable to daemonize application: {}", e);
    }

    loop {
        println!("Waiting for Open Joystick Display");

        let mut socket = match connect_to_ojd() {
            Ok(stream) => {
                println!("Connected to Open Joystick Display");
                stream
            }
            Err(e) => {
                println!("Unable to connect to Open Joystick Display: {}", e);
                continue;
            }
        };

        let mut gamepad = Gamepad::new();

        let mut recv_buf = [0; 26];

        while socket.read(&mut recv_buf).is_ok() {
            if !gamepad.connected {
                if let Err(e) = gamepad.connect() {
                    println!("Unable to detect gamepad status: {}", e);
                } else {
                    println!("Gamepad connected");
                }
            }

            if gamepad.update_state().is_err() {
                println!("Gamepad disconnected");
            }

            let payload = gamepad.build_json_payload();

            if let Err(e) = socket.write(&payload) {
                println!("Unable to send payload to Open Joystick Display: {}", e);
                break;
            }
        }

        println!("Disconnected from Open Joystick Display");
    }
}

fn daemonize_me() -> anyhow::Result<()> {
    let stdout = File::create("/tmp/ojd_server.out")?;
    let stderr = File::create("/tmp/ojd_server.err")?;

    let daemon = Daemonize::new().stdout(stdout).stderr(stderr);

    Ok(daemon.start()?)
}

fn connect_to_ojd() -> anyhow::Result<TcpStream> {
    let listener =
        TcpListener::bind("0.0.0.0:56709").context("Unable to bind to local IP address")?;

    let (stream, _) = listener
        .accept()
        .context("Unable to accept TCP connection")?;

    Ok(stream)
}

impl Gamepad {
    fn new() -> Self {
        Self::default()
    }

    fn connect(&mut self) -> anyhow::Result<()> {
        self.wait_for_connection()?;

        let mut num_axes: u8 = 0;
        let mut num_buttons: u8 = 0;
        let mut gamepad_name = [0; 128];

        let device = File::open("/dev/input/js0").context("No gamepad detected")?;

        let fd = device.as_raw_fd();

        unsafe {
            get_num_axes(fd, &mut num_axes).context("Couldn't get gamepad axes")?;
            get_num_buttons(fd, &mut num_buttons).context("Couldn't get gamepad buttons")?;
            get_gamepad_name(fd, &mut gamepad_name).context("Couldn't get gamepad name")?;
        }

        let name_len = gamepad_name
            .iter()
            .position(|&ch| ch == b'\0')
            .context("Gamepad name isn't nul-terminated somehow???")?
            + 1;

        let parsed_name: String = CStr::from_bytes_with_nul(&gamepad_name[..name_len])
            .context("Failed to parse gamepad name")?
            .to_string_lossy()
            .into();

        self.name = parsed_name;
        self.buttons.resize(num_buttons as usize, 0.);
        self.axes.resize(num_axes as usize, 0.);
        self.connected = true;

        Ok(())
    }

    fn wait_for_connection(&mut self) -> anyhow::Result<()> {
        // skip the wait if a controller is already connected
        if File::open("/dev/input/js0").is_ok() {
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

    fn update_state(&mut self) -> anyhow::Result<()> {
        let mut device = match File::open("/dev/input/js0") {
            Ok(device) => device,
            Err(e) => {
                self.connected = false;
                return Err(e.into());
            }
        };

        let mut raw = [0; 8];

        for button in &mut self.buttons {
            if let Err(e) = device.read(&mut raw) {
                self.connected = false;
                return Err(e.into());
            }

            let event: js_event = unsafe { std::mem::transmute(raw) };

            *button = event.value as f64 / i16::MAX as f64;
        }

        for axis in &mut self.axes {
            if let Err(e) = device.read(&mut raw) {
                self.connected = false;
                return Err(e.into());
            }

            let event: js_event = unsafe { std::mem::transmute(raw) };

            *axis = event.value as f64 / i16::MAX as f64;
        }

        Ok(())
    }

    fn build_json_payload(&mut self) -> Vec<u8> {
        let buttons_json: Vec<Value> = self
            .buttons
            .iter()
            .map(|&value| json!({"pressed": value !=0f64, "value": value}))
            .collect();

        let json = json!(
            {
                "axes": self.axes,
                "buttons": buttons_json,
                "connected": self.connected,
                "id": self.name,
                "index": 0,
                "mapping": "",
                "timestamp": 0,
            }
        )
        .to_string();

        let mut payload = Vec::new();

        let _ = write!(&mut payload, "{}#{}", json.len(), json);

        payload
    }
}
