# &lt;ESP32-S3-RUST-WROOM &gt;

Embedded Rust firmware for the **ESP32-S3-WROOM**, built on
[`esp-hal`](https://github.com/esp-rs/esp-hal) (`no_std`).

## About

The wiring, peripheral logic, and circuit diagrams in this project are based on the **Super
Starter Kit for ESP32-S3-WROOM** documentation:

- https://super-starter-kit-for-esp32-s3-wroom.readthedocs.io/en/latest/index.html

That kit and its tutorials are written for Arduino/C++ and MicroPython — there is **no official
Rust equivalent** — so this project re-implements the same circuits and behavior in embedded
Rust, using the kit's docs as the reference for how each component should be wired and behave.

For the Rust side (peripheral setup, API patterns, idioms), the official esp-hal examples were
the main source of inspiration:

- https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples

## Getting started

See **[GUIDE.md](./GUIDE.md)** for full toolchain setup, build, and flashing instructions
(written for Linux / Ubuntu).

Once the toolchain is installed, the short version is:

```bash
cargo build --release
cargo espflash flash --port /dev/ttyACM0 --baud 115200 --release --monitor
```

## Hardware

- ESP32-S3-WROOM dev board
- Components from the Super Starter Kit (see the kit docs linked above)

## Credits

- **Circuit logic & diagrams:** Super Starter Kit for ESP32-S3-WROOM documentation
- **Rust examples & API patterns:** [esp-rs / esp-hal](https://github.com/esp-rs/esp-hal)