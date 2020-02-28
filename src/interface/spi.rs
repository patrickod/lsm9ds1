use super::Interface;
use super::Sensor;
use embedded_hal::{blocking::spi::Transfer, blocking::spi::Write, digital::v2::OutputPin};

/// R/W bit should be high for SPI Read operation
const SPI_READ: u8 = 0x80;

/// Errors in this crate
#[derive(Debug)]
pub enum Error<CommE, PinE> {
    /// Communication error
    Comm(CommE),
    /// Pin setting error
    Pin(PinE),
}

/// This combines the SPI Interface and a data/command pin
pub struct SpiInterface<SPI, AG, M> {
    spi: SPI,
    ag_cs: AG,
    m_cs: M,
}

impl<SPI, AG, M, CommE, PinE> SpiInterface<SPI, AG, M>
where
    SPI: Transfer<u8, Error = CommE> + Write<u8, Error = CommE>,
    AG: OutputPin<Error = PinE>,
    M: OutputPin<Error = PinE>,
{
    pub fn new(spi: SPI, ag_cs: AG, m_cs: M) -> Self {
        Self { spi, ag_cs, m_cs }
    }
}

impl<SPI, AG, M, CommE, PinE> Interface for SpiInterface<SPI, AG, M>
where
    SPI: Transfer<u8, Error = CommE> + Write<u8, Error = CommE>,
    AG: OutputPin<Error = PinE>,
    M: OutputPin<Error = PinE>,
{
    type Error = Error<CommE, PinE>;

    fn write_register(&mut self, sensor: Sensor, addr: u8, value: u8) -> Result<(), Self::Error> {
        let bytes = [addr, value];
        match sensor {
            Sensor::Accelerometer | Sensor::Gyro => {
                self.ag_cs.set_low().map_err(Error::Pin)?;
                self.spi.write(&bytes).map_err(Error::Comm)?;
                self.ag_cs.set_high().map_err(Error::Pin)?;
            }
            Sensor::Magnetometer => {
                self.m_cs.set_low().map_err(Error::Pin)?;
                self.spi.write(&bytes).map_err(Error::Comm)?;
                self.m_cs.set_high().map_err(Error::Pin)?;
            }
        }
        Ok(())
    }

    fn read_register(&mut self, sensor: Sensor, addr: u8) -> Result<u8, Self::Error> {
        let mut buffer = [0u8; 2];
        buffer[0] = SPI_READ | (addr & 0x3F);
        match sensor {
            Sensor::Accelerometer | Sensor::Gyro => {
                self.ag_cs.set_low().map_err(Error::Pin)?;
                self.spi.transfer(&mut buffer).map_err(Error::Comm)?;
                self.ag_cs.set_high().map_err(Error::Pin)?;
            }
            Sensor::Magnetometer => {
                self.m_cs.set_low().map_err(Error::Pin)?;
                self.spi.transfer(&mut buffer).map_err(Error::Comm)?;
                self.m_cs.set_high().map_err(Error::Pin)?;
            }
        }
        Ok(buffer[1])
    }

    fn read_bytes(
        &mut self,
        sensor: Sensor,
        addr: u8,
        bytes: &mut [u8],
    ) -> Result<(), Self::Error> {
        bytes[0] = SPI_READ | addr;
        match sensor {
            Sensor::Accelerometer | Sensor::Gyro => {
                self.ag_cs.set_low().map_err(Error::Pin)?;
                self.spi.transfer(bytes).map_err(Error::Comm)?;
                self.ag_cs.set_high().map_err(Error::Pin)?;
            }
            Sensor::Magnetometer => {
                self.m_cs.set_low().map_err(Error::Pin)?;
                self.spi.transfer(bytes).map_err(Error::Comm)?;
                self.m_cs.set_high().map_err(Error::Pin)?;
            }
        }
        Ok(())
    }
}
