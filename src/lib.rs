#![no_std]

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;
use esp8266::SPI0;
use spi_memory::{BlockDevice, Error, Read};
use void::Void;
use xtensa_lx106_rt::rom::{SPIEraseChip, SPIEraseSector, SPIRead, SPIWrite};

const SECTOR_SIZE: u32 = 4096;

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

impl ESPFlashError {
    fn from(result: u32) -> Result<(), Self> {
        match result {
            0 => Ok(()),
            2 => Err(ESPFlashError::Timeout),
            _ => Err(ESPFlashError::Err),
        }
    }
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
}

impl BlockDevice<u32, FlashSpi, DummyCS> for ESPFlash {
    /// Erase 4K sectors
    fn erase_sectors(&mut self, addr: u32, amount: usize) -> Result<(), Error<FlashSpi, DummyCS>> {
        let start_sector = addr / SECTOR_SIZE;
        for i in 0..(amount as u32) {
            ESPFlashError::from(unsafe { SPIEraseSector(start_sector + i) })?;
        }
        Ok(())
    }

    fn erase_all(&mut self) -> Result<(), Error<FlashSpi, DummyCS>> {
        ESPFlashError::from(unsafe { SPIEraseChip() })?;

        Ok(())
    }

    fn write_bytes(&mut self, addr: u32, data: &mut [u8]) -> Result<(), Error<FlashSpi, DummyCS>> {
        ESPFlashError::from(unsafe { SPIWrite(addr, data.as_ptr(), data.len() as u32) })?;

        Ok(())
    }
}

impl Read<u32, FlashSpi, DummyCS> for ESPFlash {
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Error<FlashSpi, DummyCS>> {
        ESPFlashError::from(unsafe {
            SPIRead(addr, buf.as_mut_ptr() as *mut _, buf.len() as u32)
        })?;

        Ok(())
    }
}
