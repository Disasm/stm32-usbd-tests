#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m::asm::delay;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use stm32f3xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use stm32f3xx_hal::{hal::digital::v2::OutputPin, pac, prelude::*};
use usb_device::{class_prelude::*, prelude::*, test_class::TestClass};

static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut USB_TEST_CLASS: Option<TestClass<UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

fn configure_usb_clock() {
    let rcc = unsafe { &*pac::RCC::ptr() };
    rcc.cfgr.modify(|_, w| w.usbpre().set_bit());
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .pclk2(24.MHz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    // F3 Discovery board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa
        .pa12
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    usb_dp.set_low().ok();
    delay(clocks.sysclk().0 / 100);

    let usb_dm = gpioa
        .pa11
        .into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);
    let usb_dp = usb_dp.into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);

    configure_usb_clock();

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: usb_dm,
        pin_dp: usb_dp,
    };

    // Unsafe to allow access to static variables
    unsafe {
        USB_BUS = Some(UsbBus::new(usb));
        USB_TEST_CLASS = Some(TestClass::new(USB_BUS.as_ref().unwrap()));
        USB_DEVICE = Some(
            USB_TEST_CLASS
                .as_ref()
                .unwrap()
                .make_device(USB_BUS.as_ref().unwrap()),
        );
    }

    unsafe {
        NVIC::unmask(pac::Interrupt::USB_HP_CAN_TX);
        NVIC::unmask(pac::Interrupt::USB_LP_CAN_RX0);
    }

    loop {
        cortex_m::asm::wfi();
    }
}

use pac::interrupt;
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
