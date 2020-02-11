#![no_std]
#![no_main]


use core::panic::PanicInfo;

use cortex_m_rt::entry;
use stm32l0xx_hal::{
    prelude::*,
    pac::{
        self,
        Interrupt,
        interrupt,
        GPIOB,
    },
    rcc,
    syscfg::SYSCFG,
    usb::{UsbBus, UsbBusType, USB},
};
use usb_device::{
    prelude::*,
    class_prelude::*,
    test_class::TestClass,
};


static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>>  = None;
static mut USB_TEST_CLASS: Option<TestClass<UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>>     = None;


#[entry]
fn main() -> ! {
    let cp = pac::CorePeripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut nvic   = cp.NVIC;
    let mut rcc    = dp.RCC.freeze(rcc::Config::hsi16());
    let mut syscfg = SYSCFG::new(dp.SYSCFG, &mut rcc);
    let     hsi48  = rcc.enable_hsi48(&mut syscfg, dp.CRS);
    let     gpioa  = dp.GPIOA.split(&mut rcc);

    let usb = USB::new(dp.USB, gpioa.pa11, gpioa.pa12, hsi48);

    // Safe, as the interrupt handler that accesses these statics is not enabled
    // yet, which means we still have exclusive access.
    unsafe {
        USB_BUS = Some(UsbBus::new(usb));
        USB_TEST_CLASS = Some(TestClass::new(USB_BUS.as_ref().unwrap()));
        USB_DEVICE = Some(
            USB_TEST_CLASS
                .as_ref()
                .unwrap()
                .make_device(USB_BUS.as_ref().unwrap())
        );
    }

    nvic.enable(Interrupt::USB);

    loop {
        cortex_m::asm::wfi();
    }
}


#[interrupt]
fn USB() {
    static mut CNTR: u32 = 0;

    *CNTR += 1;
    if *CNTR > 10 {
        panic!("too many interrupt calls")
    }

    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    let test    = unsafe { USB_TEST_CLASS.as_mut().unwrap() };

    if usb_dev.poll(&mut [test]) {
        test.poll();
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
