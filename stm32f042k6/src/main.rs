#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use stm32f0xx_hal::{prelude::*, stm32};
use stm32f0xx_hal::usb::{UsbBus, Peripheral};
use usb_device::test_class::TestClass;

#[entry]
fn main() -> ! {
    let mut dp = stm32::Peripherals::take().unwrap();

    let mut rcc = dp
        .RCC
        .configure()
        .hsi48()
        .enable_crs(dp.CRS)
        .sysclk(48.mhz())
        .pclk(24.mhz())
        .freeze(&mut dp.FLASH);

    let gpioa = dp.GPIOA.split(&mut rcc);

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: gpioa.pa12,
    };
    let usb_bus = UsbBus::new(usb);

    let mut test = TestClass::new(&usb_bus);

    let mut usb_dev = { test.make_device(&usb_bus) };

    loop {
        if usb_dev.poll(&mut [&mut test]) {
            test.poll();
        }
    }
}
