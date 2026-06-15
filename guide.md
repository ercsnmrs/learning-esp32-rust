How to start

cargo install esp-generate --locked

cargo install espup

espup install

cargo build --release

cargo espflash flash --port /dev/ttyACM0 --baud 115200 --release --monitor