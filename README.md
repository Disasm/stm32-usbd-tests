# Testing process

## Stage 1: testing disconnected behavior

Disconnect USB cable from the target board. Connect ST-LINK programmer and power.
```bash
cargo run --release --example interrupt
```
Observe that there is no panic message.
Repeat for all the boards available.

## Stage 2: general tests

Connect USB cable to the target board.
 
```bash
cargo run --release
cd /path/to/usb-device
cargo test
```

Repeat for all the boards available.

## Stage 3: compliance tests
 
Use [USB 2.0 Test Tool](https://www.usb.org/usb2tools) to run the `Chapter 9 Tests [USB 2 devices]` test suite. 

Repeat for all the boards available.
