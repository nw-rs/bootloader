use synopsys_usb_otg;
use usb_device;

pub struct USB {
    hclk: u32,
}

pub fn get_usb_bus_allocator(
    hclk: u32,
    ep_memory: &'static mut [u32],
) -> usb_device::class_prelude::UsbBusAllocator<UsbBusType> {
    let rcc = unsafe { &*crate::pac::RCC::ptr() };
    let gpioa = unsafe { &*crate::pac::GPIOA::ptr() };

    rcc.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
    gpioa.afrh.modify(|_, w| w.afrh11().af10().afrh12().af10());
    gpioa
        .moder
        .modify(|_, w| w.moder11().alternate().moder12().alternate());
    gpioa.ospeedr.modify(|_, w| {
        w.ospeedr11()
            .very_high_speed()
            .ospeedr12()
            .very_high_speed()
    });

    let usb = USB { hclk };

    UsbBusType::new(usb, ep_memory)
}

impl USB {
    pub fn new(hclk: u32) -> Self {
        Self { hclk }
    }
}

unsafe impl Sync for USB {}

unsafe impl synopsys_usb_otg::UsbPeripheral for USB {
    const REGISTERS: *const () = crate::pac::OTG_FS_GLOBAL::ptr() as *const ();

    const HIGH_SPEED: bool = false;
    const FIFO_DEPTH_WORDS: usize = 320;
    const ENDPOINT_COUNT: usize = 6;

    fn enable() {
        let rcc = unsafe { &*crate::pac::RCC::ptr() };

        cortex_m::interrupt::free(|_| {
            // Enable USB peripheral
            rcc.ahb2enr.modify(|_, w| w.otgfsen().set_bit());

            // Reset USB peripheral
            rcc.ahb2rstr.modify(|_, w| w.otgfsrst().set_bit());
            rcc.ahb2rstr.modify(|_, w| w.otgfsrst().clear_bit());
        });
    }

    fn ahb_frequency_hz(&self) -> u32 {
        self.hclk
    }
}

pub type UsbBusType = synopsys_usb_otg::UsbBus<USB>;
