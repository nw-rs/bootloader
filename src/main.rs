#![no_std]
#![no_main]
#![allow(dead_code)]

use nw_board_support as _;
use panic_probe as _;

use nw_board_support::{external_flash, pac};

mod dfu;
mod usb;

const EXT_FLASH_START: u32 = 0x90000000;

// use core::mem::MaybeUninit;

// static mut USB_BUS: MaybeUninit<usb::usb_device::class_prelude::UsbBusAllocator<usb::UsbBusType>> =
//     MaybeUninit::uninit();
// static mut USB_DEVICE: MaybeUninit<usb::usb_device::device::UsbDevice<usb::UsbBusType>> =
//     MaybeUninit::uninit();
// static mut USB_DFU: MaybeUninit<usbd_dfu::DFUClass<usb::UsbBusType, dfu::QspiDfu>> =
//     MaybeUninit::uninit();

fn init_clocks() {
    let rcc = unsafe { &(*pac::RCC::ptr()) };
    let pwr = unsafe { &(*pac::PWR::ptr()) };
    let flash = unsafe { &(*pac::FLASH::ptr()) };

    // Turn on HSI
    rcc.cr.modify(|_, w| w.hsion().on());
    while rcc.cr.read().hsirdy().is_not_ready() {}

    // Switch to HSI
    rcc.cfgr.modify(|_, w| w.sw().hsi());
    while !rcc.cfgr.read().sws().is_hsi() {}

    // Enable HSE using crystal oscillator
    rcc.cr.modify(|_, w| w.hsebyp().not_bypassed());
    rcc.cr.modify(|_, w| w.hseon().on());
    while rcc.cr.read().hserdy().is_not_ready() {}

    rcc.cr.modify(|_, w| w.pllon().off());

    // Configure PLL and set its source to the HSE
    rcc.pllcfgr.modify(|_, w| unsafe {
        w.pllm().bits(4);
        w.plln().bits(216);
        w.pllp().bits(2);
        w.pllq().bits(9);
        w.pllsrc().hse()
    });

    // Enable PWR domain and set correct voltage scaling
    rcc.apb1enr.modify(|_, w| w.pwren().enabled());
    pwr.cr1.modify(|_, w| w.vos().scale1());

    // Enable PLL
    rcc.cr.modify(|_, w| w.pllon().on());
    while rcc.cr.read().pllrdy().is_not_ready() {}

    // Enable power over-drive mode
    pwr.cr1.modify(|_, w| w.oden().set_bit());
    while !pwr.csr1.read().odrdy().bit_is_set() {}

    // Switch the voltage regulator from normal mode to over-drive mode
    pwr.cr1.modify(|_, w| w.odswen().set_bit());
    while !pwr.csr1.read().odswrdy().bit_is_set() {}

    // Enable PLL48CLK
    rcc.dckcfgr2.modify(|_, w| w.ck48msel().pll());

    flash.acr.write(|w| w.latency().bits(0b0111));

    rcc.cfgr
        .modify(|_, w| w.ppre1().div4().ppre2().div2().hpre().div1());

    rcc.cfgr.modify(|_, w| w.sw().pll());
    while !rcc.cfgr.read().sws().is_pll() {}

    cortex_m::asm::delay(16);
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let rcc = unsafe { &(*pac::RCC::ptr()) };
    let gpioa = unsafe { &(*pac::GPIOA::ptr()) };
    let gpiob = unsafe { &(*pac::GPIOB::ptr()) };
    let gpioc = unsafe { &(*pac::GPIOC::ptr()) };

    // Enable GPIOs A, B and C (B for the LED, A & C for the keypad)
    rcc.ahb1enr.modify(|_, w| {
        w.gpioaen()
            .set_bit()
            .gpioben()
            .set_bit()
            .gpiocen()
            .set_bit()
    });

    // Configure pin for keypad row G (output driven low)
    gpioa.moder.modify(|_, w| w.moder6().output());
    gpioa.odr.modify(|_, w| w.odr6().low());

    // Configure LED pins as outputs
    gpiob
        .moder
        .modify(|_, w| w.moder0().output().moder4().output().moder5().output());

    // Configure pin for keypad column 1 (input with pull up)
    gpioc.moder.modify(|_, w| w.moder0().input());
    gpioc.pupdr.modify(|_, w| w.pupdr0().pull_up());

    external_flash::init();

    // Check if "4" is pressed on the keypad, if so don't boot.
    if gpioc.idr.read().idr0().is_low() {
        // Turn off the LED
        gpiob
            .odr
            .modify(|_, w| w.odr0().low().odr4().low().odr5().low());

        static mut EP_MEMORY: [u32; 1024] = [0; 1024];

        // let bus = unsafe {
        //     USB_BUS
        //         .as_mut_ptr()
        //         .write(usb::get_usb_bus_allocator(216_000_000, &mut EP_MEMORY));
        //     &*USB_BUS.as_ptr()
        // };

        let bus = usb::get_usb_bus_allocator(216_000_000, unsafe { &mut EP_MEMORY });

        init_clocks();

        let dfu_mem = dfu::QspiDfu::new();

        let mut dfu = usbd_dfu::DFUClass::new(&bus, dfu_mem);

        // unsafe {
        //     USB_DFU
        //         .as_mut_ptr()
        //         .write(usbd_dfu::DFUClass::new(bus, dfu_mem));
        // }

        let usb_id = usb_device::device::UsbVidPid(0x0483, 0xdf11);

        let mut usb_dev = usb_device::device::UsbDeviceBuilder::new(&bus, usb_id)
            .manufacturer("Numworks")
            .product("RustWorks Bootloader")
            .serial_number("TEST")
            .device_class(0xFE)
            .device_sub_class(0x01)
            .build();

        // unsafe {
        //     USB_DEVICE.as_mut_ptr().write(usb_dev);
        // }

        dp.GPIOB.odr.modify(|_, w| w.odr5().high());

        loop {
            // let usb_dev = unsafe { &mut *USB_DEVICE.as_mut_ptr() };
            // let dfu = unsafe { &mut *USB_DFU.as_mut_ptr() };

            usb_dev.poll(&mut [&mut dfu]);
        }
    } else {
        external_flash::set_memory_mapped();

        let mut delay = cortex_m::delay::Delay::new(cp.SYST, 16_000_000);

        delay.delay_ms(500);

        unsafe { cortex_m::asm::bootload(EXT_FLASH_START as *const u32) }
    }
}
