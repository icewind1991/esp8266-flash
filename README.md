# ESP8266-flash

A driver for the esp8266 onboard flash

## Example

```rust
use esp8266_hal::target::Peripherals;
use esp8266_flash::ESPFlash;
use spi_memory::prelude::*;

let dp = unsafe { Peripherals::steal() };
let pins = dp.GPIO.split();
let mut flash = ESPFlash::new(dp.SPI0);

let mut buff = [0u8; 8];

flash.read(ADDR, &mut buff).unwrap();

buff[0] += 1;

flash.erase_sectors(ADDR, 1).unwrap();

flash.write_bytes(ADDR, &mut buff).unwrap();
```

## Flashing the example

In order to flash the example program

- Edit `setenv` to point to `RUSTC`
- Connect the esp8266 over usb
- run `flash_example`
- check the output using `picocom --baud 115200 /dev/ttyUSB0`

See the [xtensa-rust-quickstart](https://github.com/icewind1991/xtensa-rust-quickstart/tree/esp8266) for more information.