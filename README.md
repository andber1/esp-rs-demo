# Demo Application for ESP32-C3-DevKit-RUST

A simple STD binary for the development board [ESP32-C3-DevKit-RUST](https://github.com/esp-rs/esp-rust-board).

## Getting Started

```
cargo espflash flash --release --baud 921600 --monitor 
```

## Prerequisites

```
sudo apt install -y pkg-config libudev-dev clang
rustup install nightly
rustup component add rust-src --toolchain nightly
cargo install ldproxy
cargo install cargo-espflash
```

# Further Reading
- https://github.com/ivmarkov/rust-esp32-std-demo
- https://github.com/esp-rs/espressif-trainings
- https://esp-rs.github.io/espressif-trainings/01_intro.html