#![no_std]
#![no_main]
#![allow(dead_code)]

use nw_board_support as _;
use panic_probe as _;

use nw_board_support::{clocks, dfu, external_flash, keypad, led, pac, usb};

const EXT_FLASH_START: u32 = 0x90000000;

use core::mem::MaybeUninit;

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

static mut USB_BUS: MaybeUninit<usb::UsbBusAllocator<usb::UsbBusType>> = MaybeUninit::uninit();
static mut USB_DEVICE: MaybeUninit<usb::UsbDevice<usb::UsbBusType>> = MaybeUninit::uninit();
static mut USB_DFU: MaybeUninit<dfu::DFUClass<usb::UsbBusType, dfu::QspiDfu>> =
    MaybeUninit::uninit();

#[cortex_m_rt::entry]
fn main() -> ! {
    keypad::KeyMatrix::init();

    external_flash::init();

    // Check if "4" is pressed on the keypad, if so don't boot.
    if keypad::KeyMatrix::scan(clocks::HSI)[6] & 1 == 1 {
        led::init();
        led::off();

        let bus = unsafe {
            USB_BUS
                .as_mut_ptr()
                .write(usb::get_usb_bus_allocator(clocks::HCLK, &mut EP_MEMORY));
            &*USB_BUS.as_ptr()
        };

        clocks::init_clocks();

        let dfu_mem = dfu::QspiDfu::new();

        unsafe {
            USB_DFU.as_mut_ptr().write(dfu::DFUClass::new(bus, dfu_mem));
        }

        let usb_id = usb::UsbVidPid(0x0483, 0xdf11);

        let usb_dev = usb::UsbDeviceBuilder::new(&bus, usb_id)
            .manufacturer("Numworks")
            .product("RustWorks Bootloader")
            .serial_number("TEST")
            .device_class(0xFE)
            .device_sub_class(0x01)
            .build();

        unsafe {
            USB_DEVICE.as_mut_ptr().write(usb_dev);
        }

        unsafe {
            cortex_m::peripheral::NVIC::unmask(pac::interrupt::OTG_FS);
        }

        led::green();

        loop {
            cortex_m::asm::wfi();
        }
    } else {
        external_flash::set_memory_mapped();

        let mut delay = cortex_m::delay::Delay::new(
            unsafe { cortex_m::Peripherals::steal().SYST },
            clocks::HSI,
        );

        delay.delay_ms(500);

        unsafe { cortex_m::asm::bootload(EXT_FLASH_START as *const u32) }
    }
}

use pac::interrupt;

static mut LED_STATE: MaybeUninit<bool> = MaybeUninit::new(false);

#[allow(non_snake_case)]
#[interrupt]
fn OTG_FS() {
    let usb_dev = unsafe { &mut *USB_DEVICE.as_mut_ptr() };
    let dfu = unsafe { &mut *USB_DFU.as_mut_ptr() };

    usb_dev.poll(&mut [dfu]);

    let led = unsafe { &mut *LED_STATE.as_mut_ptr() };

    led::set(false, *led, !*led);

    unsafe { LED_STATE.as_mut_ptr().write(!(*led)) };
}
