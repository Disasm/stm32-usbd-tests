#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use stm32_usbd::UsbBus;
use stm32l4xx_hal::{prelude::*, stm32};
use usb_device::prelude::*;
use usb_device::test_class::TestClass;

fn enable_crs() {
    let rcc = unsafe { &(*stm32::RCC::ptr()) };
    rcc.apb1enr1.modify(|_, w| w.crsen().set_bit());
    let crs = unsafe { &(*stm32::CRS::ptr()) };
    // Initialize clock recovery
    // Set autotrim enabled.
    crs.cr.modify(|_, w| w.autotrimen().set_bit());
    // Enable CR
    crs.cr.modify(|_, w| w.cen().set_bit());
}

/// Enables VddUSB power supply
fn enable_usb_pwr() {
    // Enable PWR peripheral
    let rcc = unsafe { &(*stm32::RCC::ptr()) };
    rcc.apb1enr1.modify(|_, w| w.pwren().set_bit());

    // Enable VddUSB
    let pwr = unsafe { &*stm32::PWR::ptr() };
    pwr.cr2.modify(|_, w| w.usv().set_bit());
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let _clocks = rcc
        .cfgr
        .hsi48(true)
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(&mut flash.acr);

    enable_crs();

    // disable Vddusb power isolation
    enable_usb_pwr();

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);

    let usb_dm = gpioa.pa11.into_af10(&mut gpioa.moder, &mut gpioa.afrh);
    let usb_dp = gpioa.pa12.into_af10(&mut gpioa.moder, &mut gpioa.afrh);

    let usb_bus = UsbBus::new(dp.USB, (usb_dm, usb_dp));

    let mut test = TestClass::new(&usb_bus);

    let mut usb_dev = { test.make_device(&usb_bus) };

    loop {
        if usb_dev.poll(&mut [&mut test]) {
            test.poll();
        }
    }
}
