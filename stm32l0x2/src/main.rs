#![no_std]
#![no_main]


use core::panic::PanicInfo;


use cortex_m_rt::entry;
use stm32l0xx_hal::{
    prelude::*,
    pac,
    rcc,
    syscfg::SYSCFG,
    usb::{UsbBus, USB},
};
use usb_device::test_class::TestClass;


#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc    = dp.RCC.freeze(rcc::Config::hsi16());
    let mut syscfg = SYSCFG::new(dp.SYSCFG, &mut rcc);
    let     hsi48  = rcc.enable_hsi48(&mut syscfg, dp.CRS);
    let     gpioa  = dp.GPIOA.split(&mut rcc);

    let usb = USB::new(dp.USB, gpioa.pa11, gpioa.pa12, hsi48);

    let     bus    = UsbBus::new(usb);
    let mut test   = TestClass::new(&bus);
    let mut device = { test.make_device(&bus) };

    loop {
        if device.poll(&mut [&mut test]) {
            test.poll();
        }
    }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Safe. We're already panicking, so our use of the peripherals is not going
    // to conflict with anything else.
    let dp = unsafe { pac::Peripherals::steal() };

    let mut rcc   = dp.RCC.freeze(rcc::Config::hsi16());
    let     gpiob = dp.GPIOB.split(&mut rcc);

    let mut led = gpiob.pb2.into_push_pull_output();

    loop {
        led.set_high().unwrap();
    }
}
