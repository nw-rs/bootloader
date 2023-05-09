#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate cortex_m_rt as rt;

use cortex_m_semihosting::hprintln;
use rt::entry;

use core::panic::PanicInfo;

use nw_board_support::hal;
use nw_board_support::*;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn main() -> ! {
    hprintln!("bootloader entry");
    let dp = unsafe { hal::pac::Peripherals::steal() };

    init_mpu();

    let _clocks = init_clocks(dp.RCC);

    cortex_m::asm::dmb();

    hprintln!("init external flash");
    external_flash::init();
    external_flash::set_memory_mapped();
    hprintln!("memory mapped mode enabled");

    bootloader_mpu_init();
    hprintln!("mpu configured for boot");

    hprintln!("booting");
    unsafe { cortex_m::asm::bootload(0x90000000u32 as *const u32) }
}

fn bootloader_mpu_init() {
    cortex_m::asm::dmb();

    unsafe {
        let mpu = &*cortex_m::peripheral::MPU::PTR;

        mpu.ctrl.write(0);

        mpu.rnr.write(7);
        mpu.rbar.write(0x9000_0000);
        mpu.rasr.write(1);

        mpu.ctrl.write(1);
    }

    cortex_m::asm::dsb();
    cortex_m::asm::isb();
}
