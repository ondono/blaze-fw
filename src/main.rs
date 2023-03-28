#![no_std]
#![no_main]

//use panic_halt as _;

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB PicoTool Class Device support
use usbd_picotool_reset::PicoToolReset;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico::{
    entry,
    hal::{self, pac},
    XOSC_CRYSTAL_FREQ,
};

use rp_pico::hal::{clocks::*, *};

//use bsp::hal::{
//    clocks::{init_clocks_and_plls, Clock},
//    pac,
//    sio::Sio,
//    watchdog::Watchdog,
//};

#[entry]
fn main() -> ! {
    info!("Program start");

    // get object singleton
    let mut pac = pac::Peripherals::take().unwrap();

    // configure a watchdog
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;

    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    #[cfg(feature = "rp2040-hal/rp2040-e5")]
    {
        let sio = hal::Sio::new(pac.SIO);
        let _pins = rp_pico::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
    }

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB PicoTool Class Device driver
    let mut picotool: PicoToolReset<_> = PicoToolReset::new(&usb_bus);

    // Create a USB device RPI Vendor ID and on of these Product ID:
    // https://github.com/raspberrypi/picotool/blob/master/picoboot_connection/picoboot_connection.c#L23-L27
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2e8a, 0x000a))
        .manufacturer("Fake company")
        .product("Picotool port")
        .serial_number("TEST")
        .device_class(0) // from: https://www.usb.org/defined-class-codes
        .build();

    //let mut led_pin = pins.gpio21.into_push_pull_output();

    loop {
        usb_dev.poll(&mut [&mut picotool]);
        // info!("on!");
        // led_pin.set_high().unwrap();
        // delay.delay_ms(500);
        // info!("off!");
        // led_pin.set_low().unwrap();
        // delay.delay_ms(500);
    }
}

// End of file
