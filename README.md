# mister_ojd_server
This is a server that allows [Open Joystick Display](http://kernelzechs.com/open-joystick-display/) to receive USB controller inputs from a [MiSTer FPGA](https://github.com/MiSTer-devel/Main_MiSTer/wiki) via network connection.

The server is currently functioning, but is not yet in a fully completed state.

## Building
This program uses the `inotify` API to await controller hotswap events. As of this writing, the MiSTer linux stack still uses `glibc v2.23`. This means that you would specifically need a `armv7-linux-gnueabihf-gcc-6` cross compiler to use the `gnueabihf` target, as newer GCC versions will result in the binary producing a glibc incompatibility error at runtime.

Instead of tracking down an old GCC cross compiler, it's recommended to use a `musleabihf` target to bypass glibc compatibility issues entirely. Normally using a musl toolchain would also allow you to link with `rust-lld`, but the `lld` linker is currently unable to link this program due to a bug involving unknown relocations. So you'll have to procure an `armv7-linux-gnueabihf-gcc` toolchain anyway to use its linker, but any GCC version should suffice.

Once you have done this, change the `linker` filed under `.cargo/config` to the name of the specific ARM GCC binary on your system.

You will then be able to build and link the project via Cargo:

`cargo build --release --target armv7-unknown-linux-musleabihf`

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