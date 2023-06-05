#![no_std]
#![no_main]
#![allow(dead_code)]

use core::panic::PanicInfo;
use cortex_m::Peripherals;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nw_board_support::{external_flash, pac};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hprintln!("{:?}", info);
    loop {}
}

const EXT_FLASH_START: u32 = 0x90000000;

#[entry]
fn main() -> ! {
    unsafe {
        let dp = pac::Peripherals::steal();

        dp.RCC.apb2enr.write(|w| w.syscfgen().set_bit());

        dp.RCC.ahb1enr.write(|w| w.gpioben().set_bit());

        dp.GPIOB
            .moder
            .write(|w| w.moder5().output().moder4().output().moder0().output());
        dp.GPIOB
            .odr
            .write(|w| w.odr5().high().odr4().low().odr0().low());
    }

    external_flash::init();
    external_flash::set_memory_mapped();

    unsafe {
        let p = Peripherals::steal();

        let mut delay = cortex_m::delay::Delay::new(p.SYST, 25_000_000);

        delay.delay_ms(500);

        cortex_m::asm::bootload(EXT_FLASH_START as *const u32)
    }
}
