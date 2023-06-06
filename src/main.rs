#![no_std]
#![no_main]
#![allow(dead_code)]

use nw_board_support::{external_flash, pac};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    cortex_m_semihosting::hprintln!("{:?}", info);
    loop {}
}

const EXT_FLASH_START: u32 = 0x90000000;

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Enable GPIOs A, B and C (B for the LED, A & C for the keypad)
    dp.RCC.ahb1enr.write(|w| {
        w.gpioaen()
            .set_bit()
            .gpioben()
            .set_bit()
            .gpiocen()
            .set_bit()
    });

    // Configure pin for keypad row G (output driven low)
    dp.GPIOA.moder.write(|w| w.moder6().output());
    dp.GPIOA.odr.write(|w| w.odr6().low());

    // Configure pin for keypad column 1 (input with pull up)
    dp.GPIOC.moder.write(|w| w.moder0().input());
    dp.GPIOC.pupdr.write(|w| w.pupdr0().pull_up());

    external_flash::init();

    // Check if "4" is pressed on the keypad, if so don't boot.
    if dp.GPIOC.idr.read().idr0().is_low() {
        dp.GPIOB
            .moder
            .write(|w| w.moder5().output().moder4().output().moder0().output());

        dp.GPIOB
            .odr
            .write(|w| w.odr5().high().odr4().low().odr0().low());

        loop {}
    } else {
        external_flash::set_memory_mapped();

        unsafe {
            let p = cortex_m::Peripherals::steal();

            let mut delay = cortex_m::delay::Delay::new(p.SYST, 25_000_000);

            delay.delay_ms(500);

            cortex_m::asm::bootload(EXT_FLASH_START as *const u32)
        }
    }
}
