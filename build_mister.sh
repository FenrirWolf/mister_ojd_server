#!/bin/sh

SYSROOT=/build/root

export PKG_CONFIG_DIR=
export PKG_CONFIG_LIBDIR=${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig
export PKG_CONFIG_SYSROOT_DIR=${SYSROOT}
export PKG_CONFIG_ALLOW_CROSS=1
# tell pkg-config where to find libudev.pc
export PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig
# tell cargo to link with an armhf compatible linker
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc-10

cargo build --release --target armv7-unknown-linux-gnueabihf
