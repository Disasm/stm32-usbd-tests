# Testing process

## Stage 1: testing disconnected behavior

Disconnect USB cable from the target board. Connect ST-LINK programmer and power.
```bash
cargo run --release --example interrupt
```
Observe that there is no panic message.

Connect USB cable to the target board.
Observe that there is a panic message "too many interrupt calls".

Repeat for all the boards available.

## Stage 2: serial tests

Build and run the `usb_serial` example.

```bash
cargo run --release --example usb_serial
```

Run the [serial-threadmark](https://github.com/mvirkkunen/serial-threadmark) benchmark to stress-test the USB driver.

```bash
cd /path/to/serial-threadmark
cargo run --release /dev/serial/by-id/usb-Fake_company_Serial_port_TEST-if00 1000000
```

Observe that the test passes without errors and firmware does not panic.

Repeat for all the boards available.

## Stage 3: general tests

Connect USB cable to the target board.
 
```bash
cargo run --release
cd /path/to/usb-device
cargo test
```

Repeat for all the boards available.

## Stage 4: compliance tests
 
Use [USB 2.0 Test Tool](https://www.usb.org/usb2tools) to run the `Chapter 9 Tests [USB 2 devices]` test suite. 

Repeat for all the boards available.
