# mister_ojd_server
This is a server that allows [Open Joystick Display](http://kernelzechs.com/open-joystick-display/) to receive USB controller inputs from a [MiSTer FPGA](https://github.com/MiSTer-devel/Main_MiSTer/wiki) via network connection.

The server is currently functioning, but is not yet in a fully completed state.

## Building
The easiest way to build this program is by using [cargo-cross](https://github.com/rust-embedded/cross). Install it with `cargo` by running

```
cargo install cross
```

You need to have `docker` available and running on your system for `cross` to work. follow the instructions on the linked page for more information about that. Once everything is working, you can build the program with `cross`:

```
cross build --release --target armv7-unknown-linux-gnueabihf
```

## Preparation
The server communicates with Open Joystick Display via TCP port `56709`. To allow your MiSTer to receive connections over this port, add the following rule to `/media/fat/linux/iptables.up.rules`:

`-A INPUT -p tcp -m state --state NEW --dport 56709 -j ACCEPT`

You might also have to forward port `56709` to your MiSTer's local IP address in your router.

## Running the program
First, copy the binary (via FTP or Samba or however you want to do it) to any location in the `/media/fat/` directory on the MiSTer. Next, invoke the binary over ssh.

Finally, run Open Joystick Display on your computer and choose `Network (OJD Server Beta)` under `Profile Input Driver`. Enter your MiSTer's local IP under `Server IP or Hostname` then click `Reconnect`.

Open Joystick Display can be kinda finicky about connecting over the network, so if you notice that nothing is happening then try restarting the server and then reopening OJD as well.

## Todo
* Daemonize the server application so that users won't have to manually start it up each time they want to use it.
* Create a binary release and supply a `update_ojd_server.sh` script to eliminate the need for manually compiling and moving the server binary over to the MiSTer.