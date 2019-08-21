#![no_std]
#![no_main]


use core::panic::PanicInfo;


use cortex_m_rt::entry;
use stm32l0xx_hal::{
    prelude::*,
    pac,
    rcc,
    syscfg::SYSCFG,
    usb,
};
use stm32_usbd::UsbBus;
use usb_device::test_class::TestClass;


#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc    = dp.RCC.freeze(rcc::Config::hsi16());
    let mut syscfg = SYSCFG::new(dp.SYSCFG_COMP, &mut rcc);
    let     gpioa  = dp.GPIOA.split(&mut rcc);

    usb::init(&mut rcc, &mut syscfg, dp.CRS);

    let usb_dm = gpioa.pa11;
    let usb_dp = gpioa.pa12;

    let     bus    = UsbBus::new(dp.USB, (usb_dm, usb_dp));
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
