#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use stm32_usbd::{UsbBus, UsbBusType};
use stm32f1xx_hal::{prelude::*, stm32};
use usb_device::{test_class::TestClass, prelude::*, class_prelude::*};

static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut USB_TEST_CLASS: Option<TestClass<UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    delay(clocks.sysclk().0 / 100);

    let usb_dm = gpioa.pa11;
    let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

    // Unsafe to allow access to static variables
    unsafe {
        USB_BUS = Some(UsbBus::new(dp.USB, (usb_dm, usb_dp)));
        USB_TEST_CLASS = Some(TestClass::new(USB_BUS.as_ref().unwrap()));
        USB_DEVICE = Some(USB_TEST_CLASS.as_ref().unwrap().make_device(USB_BUS.as_ref().unwrap()));
    }

    let p = cortex_m::Peripherals::take().unwrap();
    let mut nvic = p.NVIC;
    nvic.enable(stm32::Interrupt::USB_HP_CAN_TX);
    nvic.enable(stm32::Interrupt::USB_LP_CAN_RX0);

    loop {
        cortex_m::asm::wfi();
    }
}

use stm32::interrupt;
#[interrupt]
fn USB_HP_CAN_TX() {
    usb_interrupt();
}

#[interrupt]
fn USB_LP_CAN_RX0() {
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
