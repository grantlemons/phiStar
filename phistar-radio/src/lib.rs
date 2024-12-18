#![no_std]

const BUFFER_SIZE: usize = 256;
const FXOSC: u8 = 32;
const TWO_POW_19: u32 = 524288;

use core::marker::PhantomData;
use embedded_hal::i2c::{Error, I2c, SevenBitAddress};

mod config_options;
mod constants;
pub use config_options::*;
use constants::*;

pub trait RState {}
pub trait ChangeFrequency: RState {}
pub trait Recieve: RState + ReadBuffer {}
pub trait Transmit: RState + WriteBuffer {}

pub trait ReadBuffer: RState {}
pub trait WriteBuffer: RState {}

macro_rules! implement_marker_traits {
    ($n:ident, $t:ident $(,$ts:ident)+) => {
        impl $t for $n {}
        implement_marker_traits!($n $(,$ts)+);
    };
    ($n:ident, $t:ident) => {
        impl $t for $n {}
    };
    ($n:ident) => {};
}
macro_rules! add_state {
    ($n:ident, $fn:ident, $eq:path $(,$ts:ident)*) => {
        pub struct $n;
        impl RState for $n {}
        impl<'a, S: RState, I2C: I2c> RadioDevice<'a, S, I2C> {
            pub fn $fn(mut self) -> RadioDevice<'a, $n, I2C> {
                i2c_write_bits(&mut self.pins.i2c, REG_OP_MODE, $eq.into(), 2, 0)
                    .expect("Failed to change mode (I2C write error)");

                RadioDevice {
                    pins: self.pins,
                    options: self.options,
                    state: PhantomData::<$n>::default(),
                }
            }
        }
        impl<'a, I2C: I2c> From<&RadioDevice<'a, $n, I2C>> for RadioMode {
            fn from(_: &RadioDevice<$n, I2C>) -> Self {
                $eq
            }
        }
        implement_marker_traits!($n $(,$ts)*);
    };
}
add_state!(SleepState, sleep, RadioMode::SLEEP, ChangeFrequency);
add_state!(StandByState, standby, RadioMode::STDBY, ChangeFrequency);
add_state!(FSTXState, fstx, RadioMode::FSTX, WriteBuffer, Transmit);
add_state!(TXState, tx, RadioMode::TX, WriteBuffer, Transmit);
add_state!(FSRXState, fsrx, RadioMode::FSRX, ReadBuffer, Recieve);
add_state!(
    RXContinuousState,
    rxcontinuous,
    RadioMode::RXCONTINUOUS,
    ReadBuffer,
    Recieve
);
add_state!(
    RXSingleState,
    rxsingle,
    RadioMode::RXSINGLE,
    ReadBuffer,
    Recieve
);
add_state!(CADState, cad, RadioMode::CAD);

pub fn i2c_write_bits<I2C: I2c>(
    i2c: &mut I2C,
    address: SevenBitAddress,
    value: u8,
    mask_start: usize,
    mask_end: usize,
) -> Result<(), embedded_hal::i2c::ErrorKind> {
    let mut address_contents = [0; 1];
    i2c.read(address, &mut address_contents)
        .map_err(|e| e.kind())?;

    let mask_len = (mask_start - mask_end) + 1;
    let mask = ((1 << mask_len) - 1) << mask_end;
    address_contents[0] &= !mask;
    address_contents[0] |= (value << mask_end) & mask;

    i2c.write(address, &address_contents)
        .map_err(|e| e.kind())?;

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
    fn set_high(&mut self) -> Result<(), core::convert::Infallible>;
    fn set_low(&mut self) -> Result<(), core::convert::Infallible>;
}

pub struct Rfm95xPins<'a, I2C: I2c> {
    pub i2c: I2C,
    pub reset: &'a dyn PowerPin, // RFM_RST
    pub dio5: &'a dyn PowerPin,
    pub dio4: &'a dyn PowerPin,
    pub dio3: &'a dyn PowerPin,
    pub dio2: &'a dyn PowerPin,
    pub dio1: &'a dyn PowerPin,
    pub dio0: &'a dyn PowerPin,
}

pub struct RadioDevice<'a, T: RState, I2C: I2c> {
    pins: Rfm95xPins<'a, I2C>,
    options: RadioOptions,
    state: PhantomData<T>,
}

impl<'a, I2C: I2c> RadioDevice<'a, SleepState, I2C> {
    pub fn new(pins: Rfm95xPins<'a, I2C>, options: &RadioOptions) -> Result<Self, RadioError> {
        let mut new = Self {
            pins,
            options: *options,
            state: PhantomData,
        };
        new.apply_options(options)?;
        new.lora()?;

        Ok(new)
    }
}

#[non_exhaustive]
#[derive(Debug, thiserror_no_std::Error)]
pub enum RadioError {
    #[error("I2c bus error!")]
    I2cError(#[from] embedded_hal::i2c::ErrorKind),
    #[error("Invalid parameters")]
    InvalidParameters,
}

impl<'a, S: ReadBuffer, I2C: I2c> RadioDevice<'a, S, I2C> {
    pub fn read_buffer(&mut self) -> Result<([u8; BUFFER_SIZE], u8), RadioError> {
        let i2c = &mut self.pins.i2c;

        let mut addr = [0; 1];
        i2c.read(REG_FIFO_RX_CURRENT_ADDR, &mut addr)
            .map_err(|e| e.kind())?;
        i2c.write(REG_FIFO_ADDR_PTR, &addr).map_err(|e| e.kind())?;

        let mut payload_length = [0; 1];
        i2c.read(REG_RX_NB_BYTES, &mut payload_length)
            .map_err(|e| e.kind())?;

        let mut read_buffer = [0; BUFFER_SIZE];
        i2c.read(REG_FIFO, &mut read_buffer).map_err(|e| e.kind())?;

        Ok((read_buffer, payload_length[0]))
    }
}

impl<'a, S: WriteBuffer, I2C: I2c> RadioDevice<'a, S, I2C> {
    pub fn write_buffer(&mut self, data: &[u8]) -> Result<(), RadioError> {
        let i2c = &mut self.pins.i2c;

        let mut addr = [0; 1];
        i2c.read(REG_FIFO_TX_BASE_ADDR, &mut addr)
            .map_err(|e| e.kind())?;
        i2c.write(REG_FIFO_ADDR_PTR, &addr).map_err(|e| e.kind())?;

        let mut payload_length = [0; 1];
        i2c.read(REG_RX_NB_BYTES, &mut payload_length)
            .map_err(|e| e.kind())?;

        i2c.write(REG_FIFO, data).map_err(|e| e.kind())?;

        Ok(())
    }
}

impl<'a, S: RState, I2C: I2c> RadioDevice<'a, S, I2C> {
    pub fn set_power(&mut self, power: u8) -> Result<(), RadioError> {
        if !RadioOptions::verify_power_value(&power) {
            return Err(RadioError::InvalidParameters);
        }

        i2c_write_bits(&mut self.pins.i2c, REG_PA_CONFIG, power, 3, 0)?;
        self.options.power = power;
        Ok(())
    }
    pub fn set_gain(&mut self, gain: u8) -> Result<(), RadioError> {
        if !RadioOptions::verify_gain_value(&gain) {
            return Err(RadioError::InvalidParameters);
        }

        i2c_write_bits(&mut self.pins.i2c, REG_LNA, gain, 7, 5)?;
        self.options.gain = gain;
        Ok(())
    }
    pub fn set_bandwith(&mut self, bandwith: BandwithOptions) -> Result<(), RadioError> {
        i2c_write_bits(
            &mut self.pins.i2c,
            REG_MODEM_CONFIG_1,
            bandwith.into(),
            7,
            4,
        )?;
        self.options.bandwith = bandwith;
        Ok(())
    }
}

impl<'a, S: ChangeFrequency, I2C: I2c> RadioDevice<'a, S, I2C> {
    pub fn apply_options(&mut self, options: &RadioOptions) -> Result<(), RadioError> {
        self.set_power(options.power)?;
        self.set_gain(options.gain)?;
        self.set_bandwith(options.bandwith)?;
        self.set_frequency(options.frequency)?;

        Ok(())
    }
    pub fn set_frequency(&mut self, frequency: f32) -> Result<(), RadioError> {
        if !RadioOptions::verify_frequency_value(&frequency) {
            return Err(RadioError::InvalidParameters);
        }

        let freq: u32 = ((frequency * TWO_POW_19 as f32) / FXOSC as f32) as u32;
        let freq_msb_byte = (freq >> 16) as u8;
        let freq_mid_byte = (freq >> 8) as u8;
        let freq_lsb_byte = freq as u8;

        i2c_write_bits(&mut self.pins.i2c, REG_FR_MSB, freq_msb_byte, 7, 0)?;
        i2c_write_bits(&mut self.pins.i2c, REG_FR_MID, freq_mid_byte, 7, 0)?;
        i2c_write_bits(&mut self.pins.i2c, REG_FR_LSB, freq_lsb_byte, 7, 0)?;
        self.options.frequency = frequency;

        Ok(())
    }
}

impl<'a, I2C: I2c> RadioDevice<'a, SleepState, I2C> {
    pub fn fsk_ook(&mut self) -> Result<(), RadioError> {
        i2c_write_bits(&mut self.pins.i2c, REG_OP_MODE, 0, 7, 7).map_err(|e| e.into())
    }
    pub fn lora(&mut self) -> Result<(), RadioError> {
        i2c_write_bits(&mut self.pins.i2c, REG_OP_MODE, 1, 7, 7).map_err(|e| e.into())
    }
}
