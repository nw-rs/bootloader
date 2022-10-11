#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate cortex_m_rt as rt;

use rt::entry;

use cortex_m_semihosting::hprintln;

use core::{panic::PanicInfo, slice};

use nw_board_support::*;

use nw_board_support::hal;
use nw_board_support::hal::interrupt;

use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_dfu::DFUClass;

use dfu::QspiDfu;

mod dfu;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hprintln!("{:?}", info);
    loop {}
}

#[interrupt]
fn QUADSPI() {
    get_led().blue();

    unsafe {
        let qspi = &(*hal::pac::QUADSPI::ptr());
        qspi.cr.modify(|_, w| w.ftie().clear_bit());
    }
}

#[entry]
fn main() -> ! {
    let dp = unsafe { hal::pac::Peripherals::steal() };

    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    let clocks = init_clocks(dp.RCC);

    let usb_bus = get_usb_bus_allocator(&clocks, unsafe { &mut EP_MEMORY });

    let mut display = get_display(&clocks);

    external_flash::init();
    external_flash::set_memory_mapped();

    let test = unsafe { slice::from_raw_parts(0x90000000u32 as *const u32, 32) };

    hprintln!("{:08x?}\n", test);

    let test2 = unsafe { slice::from_raw_parts(0x90000010u32 as *const u32, 16) };

    hprintln!("{:08x?}\n", test2);

    external_flash::unset_memory_mapped();

    let dfu_mem = QspiDfu::new();

    let mut dfu = DFUClass::new(&usb_bus, dfu_mem);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0483, 0xdf11))
        .manufacturer("Numworks")
        .product("RustWorks Bootloader")
        .serial_number("TEST")
        .device_class(0x02)
        .build();

    display.write_top("DFU interface enabled, write to 0x900000000 for external flash.");
    display.draw_all();

    loop {
        usb_dev.poll(&mut [&mut dfu]);
    }
}
