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

use usbd_serial::SerialPort;

// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

use embedded_hal::digital::v2::OutputPin;
use hal::{clocks::*, entry, pac, *};
use rp2040_hal as hal;

#[entry]
fn main() -> ! {
    info!("Program start");
    // get object singleton
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    // configure a watchdog
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = hal::Sio::new(pac.SIO);

    let xosc_crystal_freq = 12_000_000;

    let clocks = init_clocks_and_plls(
        xosc_crystal_freq,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    //Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);

    //Set up the USB PicoTool Class Device driver
    let mut picotool: PicoToolReset<_> = PicoToolReset::new(&usb_bus);

    // Create a USB device RPI Vendor ID and on of these Product ID:
    // https://github.com/raspberrypi/picotool/blob/master/picoboot_connection/picoboot_connection.c#L23-L27
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2e8a, 0x000a))
        .manufacturer("In the loop")
        .product("picotool reset port")
        .serial_number("00001")
        .device_class(0) // from: https://www.usb.org/defined-class-codes
        .build();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.gpio21.into_push_pull_output();

    let mut count = 0;
    let mut state = false;
    loop {
        usb_dev.poll(&mut [&mut picotool, &mut serial]);
        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Err(_e) => {
                // nothing
            }
            Ok(0) => {
                // nothing
            }
            Ok(count) => {
                // Convert to upper case
                buf.iter_mut().take(count).for_each(|b| {
                    b.make_ascii_uppercase();
                });
                // Send back to the host
                let mut wr_ptr = &buf[..count];
                while !wr_ptr.is_empty() {
                    match serial.write(wr_ptr) {
                        Ok(len) => wr_ptr = &wr_ptr[len..],
                        // On error, just drop unwritten data.
                        // One possible error is Err(WouldBlock), meaning the USB
                        // write buffer is full.
                        Err(_) => break,
                    };
                }
            }
        }
        if count > 500 {
            count = 0;
            if state {
                led_pin.set_low().unwrap();
            } else {
                led_pin.set_high().unwrap();
            }
            state = !state;
        }
        count += 1;
        delay.delay_ms(1);
    }
}
