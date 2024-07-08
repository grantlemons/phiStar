use embedded_hal::spi::SpiDevice;
use radio::RadioState;

pub enum RadioState {
    Sleep,
    StandBy,
    FSTX,
    FSRX,
    TX,
    RXCONTINUOUS,
    RXSINGLE,
    CAD,
}

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

pub struct Rfm95xPins<P: PowerPin, SPI: SpiDevice> {
    pub spi: SPI,
    pub reset: P, // RFM_RST
    pub dio5: P,
    pub dio4: P,
    pub dio3: P,
    pub dio2: P,
    pub dio1: P,
    pub dio0: P,
}

pub struct Rfm95x<P: PowerPin, SPI: SpiDevice> {
    pins: Rfm95xPins<P, SPI>,
    state: RadioState,
    options: RadioOptions,
    buffer: [u8; 128],
}

impl<P: PowerPin, SPI: SpiDevice> Rfm95x<P, SPI> {
    pub fn new(pins: Rfm95xPins<P, SPI>, options: RadioOptions) -> Self {
        Self {
            pins,
            state: RadioState::StandBy,
            options,
            buffer: [0; 128],
        }
    }
}

#[derive(Debug, Default)]
pub struct ReceiveInfo {
    rssi: i16,
}

impl radio::ReceiveInfo for ReceiveInfo {
    fn rssi(&self) -> i16 {
        self.rssi
    }
}

#[derive(Debug)]
enum RadioError {
    InvalidOptions,
    StateError,
    TransmitError,
    RecieveError,
}

impl<P: PowerPin, SPI: SpiDevice> Rfm95x<P, SPI> {
    pub fn set_state(&mut self, state: RadioState) -> Result<(), RadioError> {
        self.state = state;
        todo!()
    }

    pub fn get_state(&self) -> &RadioState {
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

impl<P: PowerPin, SPI: SpiDevice> radio::Receive for Rfm95x<P, SPI> {
    type Error = RadioError;
    type Info = ReceiveInfo;

    fn start_receive(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn check_receive(&mut self, restart: bool) -> Result<bool, Self::Error> {
        todo!()
    }

    fn get_received(&mut self, buff: &mut [u8]) -> Result<(usize, Self::Info), Self::Error> {
        todo!()
    }
}

impl<P: PowerPin, SPI: SpiDevice> radio::Transmit for Rfm95x<P, SPI> {
    type Error = RadioError;

    fn start_transmit(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn check_transmit(&mut self) -> Result<bool, Self::Error> {
        todo!()
    }
}

impl<P: PowerPin, SPI: SpiDevice> radio::Power for Rfm95x<P, SPI> {
    type Error = RadioError;

    fn set_power(&mut self, power: i8) -> Result<(), Self::Error> {
        if !RadioOptions::verify_power_value(&power) {
            return Err(RadioError::InvalidOptions);
        }
        self.options.power = power;
        todo!()
    }
}

impl<P: PowerPin, SPI: SpiDevice> radio::Busy for Rfm95x<P, SPI> {
    type Error = RadioError;

    fn is_busy(&mut self) -> Result<bool, Self::Error> {
        todo!()
    }
}
