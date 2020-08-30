//! Count how many times the device has been reset

#![no_std]
#![no_main]

use xtensa_lx106_rt::entry;

use core::fmt::Write;
use core::panic::PanicInfo;
use esp8266_flash::ESPFlash;
use esp8266_hal::prelude::*;
use esp8266_hal::target::Peripherals;
use spi_memory::prelude::*;

#[entry]
fn main() -> ! {
    let dp = unsafe { Peripherals::steal() };
    let pins = dp.GPIO.split();
    let mut led = pins.gpio2.into_push_pull_output();
    let (mut timer1, _) = dp.TIMER.timers();

    let mut serial = dp
        .UART0
        .serial(pins.gpio1.into_uart(), pins.gpio3.into_uart());
    let mut flash = ESPFlash::new(dp.SPI0);

    const ADDR: u32 = 0x7E000;

    timer1.delay_ms(1000);

    led.set_high().unwrap();

    let mut buff = [0u8; 8];

    flash.read(ADDR, &mut buff).unwrap();

    buff[0] += 1;

    flash.erase_sectors(ADDR, 1).unwrap();

    flash.write_bytes(ADDR, &mut buff).unwrap();

    let _ = write!(&mut serial, "counter {}:\r\n", buff[0]);

    loop {}
}

/// Basic panic handler - just loops
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
