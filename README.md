# mister_ojd_server

This is a WIP server for [Open Joystick Display](http://kernelzechs.com/open-joystick-display/) for use with the [MiSTer FPGA platform](https://github.com/MiSTer-devel/Main_MiSTer/wiki). It is currently in a heavily incomplete state.

## roadmap
* Flesh out the input-handling logic. Input handling is currently based on [gilrs_core](https://crates.io/crates/gilrs-core), which should allow quick prototyping on desktop operating systems as well as on the MiSTer itself.
* Detail the cross-compilation process. It's a huge pain due to `gilrs_core`'s dependency on `libudev-sys`, and in the future I might rewrite the gamepad logic to directly interface with `/dev/js0` so as to spare users the hassle.
* Detail the prerequisites for getting MiSTer to accept the client connection. You have to open port 56709 on the MiSTer itself and most users will have to forward the port in their router too.
* Create a binary release and supply a `update_ojd_server.sh` script for use on the MiSTer itself.
* Daemonize the server application so that users won't have to manually start it up each time they want to use it.