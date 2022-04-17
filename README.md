# mister_ojd_server
This a server that allows [Open Joystick Display](https://proxy.vulpes.one/gemini/kernelzechs.com/ojd/downloads/) to receive usb controller inputs from a [MiSTer FPGA](https://github.com/mister-devel/main_mister/wiki) via network connection.

# Installing
Download the release file and unzip it. You'll find two folders, one containing the server binary and another containing a modified `iptables.up.rules` file along with a `user-startup.sh` script. Copy both folders to the root of your mister's sd card. Next time you power on your MiSTer, the server will automatically start and wait for a connection from Open Joystic Display.

If you would rather build the server yourself then you can follow the steps in the next section.

## Building
The easiest way to build the program is by using [cargo-cross](https://github.com/rust-embedded/cross). install it with `cargo` by running

```
cargo install cross
```

You need to have `docker` available and running on your system for `cross` to work. Follow the instructions on the linked page for more information about that. Once everything is working, you can build the program with `cross`:

```
cross build --release --target armv7-unknown-linux-gnueabihf
```

## Connecting to the server
The server communicates with Open Joystick Display over TCP port `56709`. You might have to set up a port forwarding rule in your router in order for the connection to be allowed.

once you've done that, run open joystick display on your computer and choose `Network (OJD Server Beta)` under `Profile Input Driver`. Enter `MiSTer` as the hostname under `Server IP or Hostname`, then click `Reconnect`.

Open joystick display can be kinda finicky about connecting over the network, so if you notice that nothing is happening then try restarting the program.

## Q&A

Q: I can connect open joystick display to the server, but the button mappings are all wrong!

A: That happens because the server returns raw button data exactly as received from the linux kernel. I suggest cloning the profile mapping for your controller in open joystick display, then manually changing the button map until your inputs are properly displayed.


Q: Will you release an updater script for the server?

A: Probably not. The program is pretty simple and will rarely need to be updated once installed, if ever at all.