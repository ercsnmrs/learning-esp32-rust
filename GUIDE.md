# Setup & Flashing Guide (Linux / Ubuntu)

This walks through everything needed to build and flash this `no_std` Rust firmware to an
**ESP32-S3** board, from a clean Ubuntu install. Commands assume Ubuntu with `bash`.

---

## 1. System prerequisites

`espup`, `espflash`, and friends are compiled from source and need a few system packages
(`libudev` is what lets the flasher talk to the serial port):

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libudev-dev git curl
```

If you don't already have Rust, install `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# then restart the shell, or:  source "$HOME/.cargo/env"
```

---

## 2. Install the ESP tooling

```bash
cargo install espup --locked                       # manages the Xtensa Rust toolchain
cargo install esp-generate --locked                # project template generator
cargo install ldproxy --locked                     # linker shim used by some ESP builds
cargo install esp-config --features=tui --locked   # TUI for esp-hal config options
cargo install espflash --locked                    # flasher + serial monitor
cargo install cargo-espflash --locked              # cargo subcommand wrapper
```

---

## 3. Install the Xtensa Rust toolchain

The ESP32-S3 uses the Xtensa architecture, which mainline Rust doesn't support. `espup`
installs a Rust compiler fork (the `esp` toolchain) plus the `xtensa-esp32s3-elf-gcc` linker:

```bash
espup install
```

---

## 4. Load the ESP environment (the step everyone forgets)

`espup install` writes an export script but **does not** modify your current shell. Until you
source it, `xtensa-esp32s3-elf-gcc` is not on your `PATH` — which is exactly why a fresh
terminal fails at the **link** step with the linker "not found". Source it:

```bash
. $HOME/export-esp.sh
```

Make it permanent so every new shell has it:

```bash
echo '. $HOME/export-esp.sh' >> ~/.bashrc
```

Confirm the linker is reachable:

```bash
which xtensa-esp32s3-elf-gcc
```

> **Note:** You do **not** need `rustup component add rust-src --toolchain nightly` for the
> ESP32-S3. That's only for RISC-V ESP chips that build with nightly + `build-std`. The `esp`
> toolchain already includes everything, and the generated `rust-toolchain.toml` selects it
> automatically.

---

## 5. Generate a project (if starting fresh)

```bash
esp-generate <name_of_your_project>
cd <name_of_your_project>
```

(Skip this if you're building an existing project — just `cd` into it.)

---

## 6. Build

```bash
cargo build --release
```

---

## 7. Serial port permissions

Find your board's port (native USB shows up as `ttyACM*`, a UART bridge as `ttyUSB*`):

```bash
ls /dev/ttyACM* /dev/ttyUSB*
```

If `espflash` reports a permission error opening the port, your user isn't in the `dialout`
group. Add yourself, then **log out and back in (or reboot)** for it to take effect — after
that you won't need `sudo` to flash:

```bash
sudo usermod -aG dialout $USER
```

---

## 8. Flash & monitor

Sanity-check the connection first:

```bash
espflash board-info
```

Then flash the release build and open the serial monitor:

```bash
cargo espflash flash --port /dev/ttyACM0 --baud 115200 --release --monitor
```

Adjust `--port` to whatever step 7 reported. Press `Ctrl+C` to exit the monitor.