#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use stm32f0xx_hal::{prelude::*, stm32};

use stm32_usbd::{UsbBus, UsbBusType};
use usb_device::{test_class::TestClass, prelude::*, class_prelude::*};

static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut USB_TEST_CLASS: Option<TestClass<UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

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

    let usb_dm = gpioa.pa11;
    let usb_dp = gpioa.pa12;

    // Unsafe to allow access to static variables
    unsafe {
        USB_BUS = Some(UsbBus::new(dp.USB, (usb_dm, usb_dp)));
        USB_TEST_CLASS = Some(TestClass::new(USB_BUS.as_ref().unwrap()));
        USB_DEVICE = Some(USB_TEST_CLASS.as_ref().unwrap().make_device(USB_BUS.as_ref().unwrap()));
    }

    let p = cortex_m::Peripherals::take().unwrap();
    let mut nvic = p.NVIC;
    nvic.enable(stm32::Interrupt::USB);

    loop {
        cortex_m::asm::wfi();
    }
}

use stm32::interrupt;
#[interrupt]
fn USB() {
    usb_interrupt();
}

fn usb_interrupt() {
    static mut CNTR: u32 = 0;
    unsafe {
        CNTR += 1;
        if CNTR > 10 {
            panic!("too many interrupt calls")
        }
    }

    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    let test = unsafe { USB_TEST_CLASS.as_mut().unwrap() };

    if usb_dev.poll(&mut [test]) {
        test.poll();
    }
}
