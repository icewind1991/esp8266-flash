#![no_std]

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;
use esp8266::SPI0;
use spi_memory::{BlockDevice, Error, Read};
use void::Void;

pub struct FlashSpi(SPI0);

/// Dummy chip select, since the onboard flash_example spi uses a hardware chip select
pub struct DummyCS;

impl Transfer<u8> for FlashSpi {
    type Error = ESPFlashError;

    fn transfer<'w>(&mut self, _words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        unreachable!()
    }
}

impl OutputPin for DummyCS {
    type Error = Void;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unreachable!()
    }
}

/// Access for the ESP8266 builtin flash_example
pub struct ESPFlash {
    spi: FlashSpi,
}

#[derive(Debug)]
pub enum ESPFlashError {
    Err = 1,
    Timeout = 2,
}

impl From<ESPFlashError> for Error<FlashSpi, DummyCS> {
    fn from(err: ESPFlashError) -> Self {
        Error::Spi(err)
    }
}

impl ESPFlash {
    pub fn new(spi: SPI0) -> Self {


        // take ownership of SPI0 to ensure nobody else can mess with the spi
        ESPFlash { spi: FlashSpi(spi) }
    }

    pub fn decompose(self) -> SPI0 {
        self.spi.0
    }

    fn write_enable(&mut self) {
        self.spi.0.spi_addr.write(|w| unsafe { w.bits(0) });
        self.spi.0.spi_cmd.write(|w| w.spi_write_enable().set_bit());

        while self.spi.0.spi_cmd.read().bits() > 0 {}
    }

    fn get_status(&mut self) -> u32 {
        self.spi.0.spi_addr.write(|w| unsafe { w.bits(0) });
        self.spi.0.spi_cmd.write(|w| w.spi_read_sr().set_bit());

        while self.spi.0.spi_cmd.read().bits() > 0 {}

        self.spi.0.spi_rd_status.read().bits()
    }
}

impl BlockDevice<u32, FlashSpi, DummyCS> for ESPFlash {
    /// Erase 4K sectors
    fn erase_sectors(&mut self, addr: u32, amount: usize) -> Result<(), Error<FlashSpi, DummyCS>> {
        self.write_enable();
        for i in 0..amount {
            self.spi.0.spi_addr.write(|w| unsafe { w.address().bits(addr + i as u32) });
            self.spi.0.spi_cmd.write(|w| w.spi_se().set_bit());

            while self.spi.0.spi_cmd.read().bits() > 0 {}

            while self.get_status() & 1 > 0 {}
        }

        Ok(())
    }

    fn erase_all(&mut self) -> Result<(), Error<FlashSpi, DummyCS>> {
        self.spi.0.spi_cmd.write(|w| w.spi_ce().set_bit());

        while self.spi.0.spi_cmd.read().bits() > 0 {}

        Ok(())
    }

    fn write_bytes(&mut self, addr: u32, data: &mut [u8]) -> Result<(), Error<FlashSpi, DummyCS>> {
        self.write_enable();
        // todo 64 byte chunks
        for (i, byte) in data.iter().enumerate() {
            self.spi.0.spi_addr.write(|w| unsafe { w.address().bits(addr + i as u32).size().bits(1) });
            self.spi.0.spi_w0.write(|w| unsafe { w.bits(*byte as u32) });
            self.spi.0.spi_cmd.write(|w| w.spi_pp().set_bit());

            while self.spi.0.spi_cmd.read().bits() > 0 {}

            while self.get_status() & 1 > 0 {}
        }

        Ok(())
    }
}

impl Read<u32, FlashSpi, DummyCS> for ESPFlash {
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Error<FlashSpi, DummyCS>> {
        // todo 64 byte chunks
        for (i, byte) in buf.iter_mut().enumerate() {
            self.spi.0.spi_addr.write(|w| unsafe { w.address().bits(addr + i as u32).size().bits(1) });
            self.spi.0.spi_cmd.write(|w| w.spi_read().set_bit());

            while self.spi.0.spi_cmd.read().bits() > 0 {}

            *byte = self.spi.0.spi_w0.read().bits() as u8;
        }

        Ok(())
    }
}
