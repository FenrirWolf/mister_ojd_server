# mister_ojd_server
This a server that allows [Open Joystick Display](http://2.233.121.97/openjoystick.htm) to receive USB controller input data from a [MiSTer FPGA](https://github.com/MiSTer-devel/Wiki_MiSTer/wiki) via local network connection.

# Installing
Download the release file and unzip it. You'll find two folders, one containing the server binary and another containing a `user-startup.sh` script. Copy both folders to the root of your MiSTer's sd card. Next time you power on your MiSTer, the server will automatically start and wait for a connection from Open Joystic Display.

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

Additionally, if the `/media/fat/linux` directory on your MiSTer's SD card contains an `iptables.up.rules` file, then you might have to allow-list port `56709` there too. See the `iptables.up.rules` file included in this repository for an example of how you can do that.

Once you've done that, run Open Joystick Display on your computer and choose `Network (OJD Server Beta)` under `Profile Input Driver`. Enter `MiSTer` as the hostname under `Server IP or Hostname`, then click `Reconnect`.

Open Joystick Display can be kinda finicky about connecting over the network, so if you notice that nothing is happening then try restarting the program.

## FAQ

Q: I keep getting errors when trying to connect to the server!

A: Double-check the steps outlined in the `Connecting to the server` section above. If you still encounter problems after following both those steps, feel free to open an issue.

Note: An easy way to verify that the server is working is to run `nc MiSTer 56709` in your terminal then press Enter a few times. If you get a JSON payload in response then the server is successfully sending your gamepad's state.

Q: I can connect Open Joystick Display to the server, but the button mappings are all wrong!

A: That happens because the server returns raw button data exactly as received from the linux kernel without any further processing or remapping. I suggest cloning the profile mapping for your controller in Open Joystick Display, then manually changing the button map until your inputs are properly displayed.

Q: Will you release an updater script for the server?

A: Maybe, but probably not. The program is pretty simple and will rarely need to be updated once installed, if ever at all.

## License
This software is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.