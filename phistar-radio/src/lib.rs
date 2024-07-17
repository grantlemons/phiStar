#![no_std]

const BUFFER_SIZE: usize = 256;
const FXOSC: u8 = 32;
const TWO_POW_19: u32 = 524288;

use core::{marker::PhantomData, usize};
use embedded_hal::i2c::{I2c, SevenBitAddress};

mod config_options;
mod constants;
use config_options::*;
use constants::*;

pub trait RState {}
pub trait ChangeFrequency: RState {}
pub trait Recieve: RState + ReadBuffer {}
pub trait Transmit: RState + WriteBuffer {}

pub trait ReadBuffer: RState {}
pub trait WriteBuffer: RState {}

macro_rules! add_state {
    ($n:ident,$fn:ident,$eq:path) => {
        pub struct $n;
        impl RState for $n {}
        impl<S: RState, P: PowerPin, I2C: I2c> RadioDevice<S, P, I2C> {
            pub fn $fn(mut self) -> RadioDevice<$n, P, I2C> {
                i2c_write_bits(&mut self.pins.i2c, REG_OP_MODE, 0b0000_0011, $eq.into())
                    .expect("Failed to change mode (I2C write error)");

                RadioDevice {
                    pins: self.pins,
                    options: self.options,
                    state: PhantomData::<$n>::default(),
                }
            }
        }
        impl<P: PowerPin, I2C: I2c> From<&RadioDevice<$n, P, I2C>> for RadioMode {
            fn from(_: &RadioDevice<$n, P, I2C>) -> Self {
                $eq
            }
        }
    };
}
add_state!(SleepState, sleep, RadioMode::SLEEP);
add_state!(StandByState, standby, RadioMode::STDBY);
add_state!(FSTXState, fstx, RadioMode::FSTX);
add_state!(TXState, tx, RadioMode::TX);
add_state!(FSRXState, fsrx, RadioMode::FSRX);
add_state!(RXContinuousState, rxcontinuous, RadioMode::RXCONTINUOUS);
add_state!(RXSingleState, rxsingle, RadioMode::RXSINGLE);
add_state!(CADState, cad, RadioMode::CAD);

impl ChangeFrequency for SleepState {}
impl ChangeFrequency for StandByState {}

impl ReadBuffer for RXContinuousState {}
impl ReadBuffer for RXSingleState {}
impl ReadBuffer for FSRXState {}
impl Recieve for RXContinuousState {}
impl Recieve for RXSingleState {}
impl Recieve for FSRXState {}

impl WriteBuffer for FSTXState {}
impl WriteBuffer for TXState {}
impl Transmit for FSTXState {}
impl Transmit for TXState {}

pub fn i2c_write_bits<I2C: I2c>(
    i2c: &mut I2C,
    address: SevenBitAddress,
    mask: u8,
    value: u8,
) -> Result<(), I2C::Error> {
    let mut address_contents = [0; 1];
    i2c.read(address, &mut address_contents)?;

    address_contents[0] &= !mask;
    address_contents[0] |= value & mask;

    i2c.write(address, &address_contents)?;

    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub struct RadioOptions {
    /// 2 to 17 dBm
    power: u8,
    /// 1 to 6
    gain: u8,
    /// 868.0 to 915.0 allowed
    frequency: f32,
    bandwith: BandwithOptions,
}

pub trait PowerPin {
    fn set_high(&mut self) -> Option<()>;
    fn set_low(&mut self) -> Option<()>;
}

pub struct Rfm95xPins<P: PowerPin, I2C: I2c> {
    pub i2c: I2C,
    pub reset: P, // RFM_RST
    pub dio5: P,
    pub dio4: P,
    pub dio3: P,
    pub dio2: P,
    pub dio1: P,
    pub dio0: P,
}

pub struct RadioDevice<T: RState, P: PowerPin, I2C: I2c> {
    pins: Rfm95xPins<P, I2C>,
    options: RadioOptions,
    state: PhantomData<T>,
}

impl<S: RState, P: PowerPin, I2C: I2c> RadioDevice<S, P, I2C> {
    pub fn set_power(&mut self, power: u8) -> Result<(), I2C::Error> {
        i2c_write_bits(&mut self.pins.i2c, REG_PA_CONFIG, 0b0000_1111, power)?;
        self.options.power = power;
        Ok(())
    }
    pub fn set_gain(&mut self, gain: u8) -> Result<(), I2C::Error> {
        i2c_write_bits(&mut self.pins.i2c, REG_LNA, 0b1110_0000, gain)?;
        self.options.gain = gain;
        Ok(())
    }
    pub fn set_bandwith(&mut self, bandwith: BandwithOptions) -> Result<(), I2C::Error> {
        i2c_write_bits(
            &mut self.pins.i2c,
            REG_MODEM_CONFIG_1,
            0b1111_0000,
            bandwith.into(),
        )?;
        self.options.bandwith = bandwith;
        Ok(())
    }
}

impl<S: ReadBuffer, P: PowerPin, I2C: I2c> RadioDevice<S, P, I2C> {
    pub fn read_buffer(&mut self) -> Result<([u8; BUFFER_SIZE], u8), I2C::Error> {
        let i2c = &mut self.pins.i2c;

        let mut addr = [0; 1];
        i2c.read(REG_FIFO_RX_CURRENT_ADDR, &mut addr)?;
        i2c.write(REG_FIFO_ADDR_PTR, &addr)?;

        let mut payload_length = [0; 1];
        i2c.read(REG_RX_NB_BYTES, &mut payload_length)?;

        let mut read_buffer = [0; BUFFER_SIZE];
        i2c.read(REG_FIFO, &mut read_buffer)?;

        Ok((read_buffer, payload_length[0]))
    }
}

impl<S: WriteBuffer, P: PowerPin, I2C: I2c> RadioDevice<S, P, I2C> {
    pub fn write_buffer(&mut self, data: &mut [u8]) -> Result<(), I2C::Error> {
        let i2c = &mut self.pins.i2c;

        let mut addr = [0; 1];
        i2c.read(REG_FIFO_TX_BASE_ADDR, &mut addr)?;
        i2c.write(REG_FIFO_ADDR_PTR, &addr)?;

        let mut payload_length = [0; 1];
        i2c.read(REG_RX_NB_BYTES, &mut payload_length)?;

        i2c.read(REG_FIFO, data)?;

        Ok(())
    }
}

impl<S: ChangeFrequency, P: PowerPin, I2C: I2c> RadioDevice<S, P, I2C> {
    pub fn set_frequency(&mut self, frequency: f32) -> Result<(), I2C::Error> {
        let freq: u32 = ((frequency * TWO_POW_19 as f32) / FXOSC as f32) as u32;
        let freq_msb_byte = (freq >> 16) as u8 & 0b1111_1111;
        let freq_mid_byte = (freq >> 8) as u8 & 0b1111_1111;
        let freq_lsb_byte = freq as u8 & 0b1111_1111;

        i2c_write_bits(&mut self.pins.i2c, REG_FR_MSB, 0b1111_1111, freq_msb_byte)?;
        i2c_write_bits(&mut self.pins.i2c, REG_FR_MID, 0b1111_1111, freq_mid_byte)?;
        i2c_write_bits(&mut self.pins.i2c, REG_FR_LSB, 0b1111_1111, freq_lsb_byte)?;
        self.options.frequency = frequency;

        Ok(())
    }
}

impl<P: PowerPin, I2C: I2c> RadioDevice<SleepState, P, I2C> {
    pub fn fsk_ook(&mut self) -> Result<(), I2C::Error> {
        i2c_write_bits(&mut self.pins.i2c, REG_OP_MODE, 0b1000_0000, 0b0)
    }
    pub fn lora(&mut self) -> Result<(), I2C::Error> {
        i2c_write_bits(&mut self.pins.i2c, REG_OP_MODE, 0b1000_0000, 0b1)
    }
}

#[derive(Debug)]
pub enum RadioError {
    InvalidOptions,
    StateError,
    TransmitError,
    RecieveError,
}

impl RadioOptions {
    pub fn verify(&self) -> bool {
        Self::verify_power_value(&self.power)
            && Self::verify_gain_value(&self.gain)
            && Self::verify_frequency_value(&self.frequency)
    }
    pub fn verify_power_value(power: &u8) -> bool {
        (2..17).contains(power)
    }

    pub fn verify_gain_value(gain: &u8) -> bool {
        (1..6).contains(gain)
    }

    pub fn verify_frequency_value(frequency: &f32) -> bool {
        (868.0..915.0).contains(frequency)
    }
}
