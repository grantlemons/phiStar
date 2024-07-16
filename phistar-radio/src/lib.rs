#![no_std]

const BUFFER_SIZE: usize = 256;

use core::{marker::PhantomData, usize};
use embedded_hal::i2c::{I2c, SevenBitAddress};

mod constants;
use constants::*;

pub trait RState {}
pub enum RadioMode {
    SLEEP,
    STDBY,
    FSTX,
    TX,
    FSRX,
    RXCONTINUOUS,
    RXSINGLE,
    CAD,
}

impl Into<u8> for RadioMode {
    fn into(self) -> u8 {
        match self {
            RadioMode::SLEEP => 0b000,
            RadioMode::STDBY => 0b010,
            RadioMode::FSTX => 0b010,
            RadioMode::TX => 0b011,
            RadioMode::FSRX => 0b100,
            RadioMode::RXCONTINUOUS => 0b101,
            RadioMode::RXSINGLE => 0b110,
            RadioMode::CAD => 0b111,
        }
    }
}

pub enum BandwithOptions {
    Bw007_8,
    Bw010_4,
    Bw015_6,
    Bw020_8,
    Bw031_25,
    Bw041_7,
    Bw062_5,
    Bw125_0,
    Bw250_0,
    Bw500_0,
}

impl Into<u8> for BandwithOptions {
    fn into(self) -> u8 {
        match self {
            BandwithOptions::Bw007_8 => 0b0000,
            BandwithOptions::Bw010_4 => 0b0001,
            BandwithOptions::Bw015_6 => 0b0010,
            BandwithOptions::Bw020_8 => 0b0011,
            BandwithOptions::Bw031_25 => 0b0100,
            BandwithOptions::Bw041_7 => 0b0101,
            BandwithOptions::Bw062_5 => 0b0110,
            BandwithOptions::Bw125_0 => 0b0111,
            BandwithOptions::Bw250_0 => 0b1000,
            BandwithOptions::Bw500_0 => 0b1001,
        }
    }
}

pub fn i2c_write_bits(i2c: &mut impl I2c, address: SevenBitAddress, mask: u8, value: u8) {
    let mut address_contents = [0; 1];
    i2c.read(address, &mut address_contents)
        .expect("I2C mode-change transaction failed!");

    address_contents[0] &= !mask;
    address_contents[0] |= value & mask;

    i2c.write(address, &address_contents)
        .expect("I2C mode-change transaction failed!");
}

macro_rules! add_state {
    ($n:ident,$fn:ident,$eq:path) => {
        pub struct $n;
        impl RState for $n {}
        impl<S: RState, P: PowerPin, I2C: I2c> RadioDevice<S, P, I2C> {
            pub fn $fn(mut self) -> RadioDevice<$n, P, I2C> {
                i2c_write_bits(
                    &mut self.rfm95x.pins.i2c,
                    REG_OP_MODE,
                    0b0000_0011,
                    $eq.into(),
                );

                RadioDevice {
                    rfm95x: self.rfm95x,
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

#[derive(Clone, Copy, Debug)]
pub struct RadioOptions {
    /// 2 to 17 dBm
    pub power: i8,
    /// 1 to 6
    pub gain: i8,
    /// 868.0 to 915.0 allowed
    pub frequency: f32,
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

pub struct Rfm95x<P: PowerPin, I2C: I2c> {
    pins: Rfm95xPins<P, I2C>,
    state: RadioMode,
    options: RadioOptions,
}

pub struct RadioDevice<T: RState, P: PowerPin, I2C: I2c> {
    rfm95x: Rfm95x<P, I2C>,
    state: PhantomData<T>,
}

impl<P: PowerPin, I2C: I2c> RadioDevice<SleepState, P, I2C> {
    pub fn fsk_ook(&mut self) {
        i2c_write_bits(&mut self.rfm95x.pins.i2c, REG_OP_MODE, 0b1000_0000, 0b0);
    }
    pub fn lora(&mut self) {
        i2c_write_bits(&mut self.rfm95x.pins.i2c, REG_OP_MODE, 0b1000_0000, 0b1);
    }
}

impl<S: RState, P: PowerPin, I2C: I2c> RadioDevice<S, P, I2C> {
    fn set_gain(&mut self, gain: u8) {
        i2c_write_bits(&mut self.rfm95x.pins.i2c, REG_LNA, 0b1110_0000, gain);
    }
    fn set_bandwith(&mut self, bandwith: BandwithOptions) {
        i2c_write_bits(
            &mut self.rfm95x.pins.i2c,
            REG_MODEM_CONFIG_1,
            0b1111_0000,
            bandwith.into(),
        );
    }
}

impl<P: PowerPin, I2C: I2c> Rfm95x<P, I2C> {
    pub fn new(pins: Rfm95xPins<P, I2C>, options: RadioOptions) -> Self {
        Self {
            pins,
            state: RadioMode::STDBY,
            options,
        }
    }

    pub fn buffer(&mut self) -> ([u8; BUFFER_SIZE], u8) {
        let i2c = &mut self.pins.i2c;

        let mut current_addr = [0; 1];
        i2c.read(REG_FIFO_RX_CURRENT_ADDR, &mut current_addr)
            .expect("Unable to read from I2C");
        i2c.write(REG_FIFO_ADDR_PTR, &current_addr)
            .expect("Unable to write to I2C");

        let mut payload_length = [0; 1];
        i2c.read(REG_RX_NB_BYTES, &mut payload_length)
            .expect("Unable to read from I2C");

        let mut read_buffer = [0; BUFFER_SIZE];
        i2c.read(REG_FIFO, &mut read_buffer)
            .expect("Unable to read from I2C");

        (read_buffer, payload_length[0])
    }
}

#[derive(Debug)]
pub enum RadioError {
    InvalidOptions,
    StateError,
    TransmitError,
    RecieveError,
}

impl<P: PowerPin, I2C: I2c> Rfm95x<P, I2C> {
    pub fn set_state(&mut self, state: RadioMode) -> Result<(), RadioError> {
        self.state = state;
        todo!()
    }

    pub fn get_state(&self) -> &RadioMode {
        &self.state
    }
}

impl RadioOptions {
    pub fn verify(&self) -> bool {
        Self::verify_power_value(&self.power)
            && Self::verify_gain_value(&self.gain)
            && Self::verify_frequency_value(&self.frequency)
    }

    pub fn verify_power_value(power: &i8) -> bool {
        (2..17).contains(power)
    }

    pub fn verify_gain_value(gain: &i8) -> bool {
        (1..6).contains(gain)
    }

    pub fn verify_frequency_value(frequency: &f32) -> bool {
        (868.0..915.0).contains(frequency)
    }
}
