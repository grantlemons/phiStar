enum RadioState {
    Sleep,
    StandBy,
    FSTX,
    FSRX,
    TX,
    RXCONTINUOUS,
    RXSINGLE,
    CAD,
}

struct RadioOptions {
    /// 2 to 17 dBm
    pub power: i8,
    /// 1 to 6
    pub gain: i8,
    /// 868.0 to 915.0 allowed
    pub frequency: f32,
}

struct Rfm95x {
    state: RadioState,
    options: RadioOptions,
}

#[derive(Debug)]
enum RadioError {
    InvalidOptions,
    StateError,
    TransmitError,
    RecieveError,
}

impl Rfm95x {
    pub fn set_state(&mut self, state: RadioState) -> Result<(), RadioError> {
        self.state = state;
        todo!()
    }

    pub fn get_state(&self) -> &RadioState {
        &self.state
    }
}

impl RadioOptions {
    pub fn verify(self) -> bool {
        self.verify_power() && self.verify_gain() && self.verify_frequency()
    }

    fn verify_power(self) -> bool {
        (2..17).contains(&self.power)
    }
    fn verify_gain(self) -> bool {
        (1..6).contains(&self.gain)
    }
    fn verify_frequency(self) -> bool {
        (868.0..915.0).contains(&self.frequency)
    }
}

impl radio::Receive for Rfm95x {
    type Error = RadioError;
    type Info;

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

impl radio::Transmit for Rfm95x {
    type Error = RadioError;

    fn start_transmit(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn check_transmit(&mut self) -> Result<bool, Self::Error> {
        todo!()
    }
}

impl radio::Power for Rfm95x {
    type Error = RadioError;

    fn set_power(&mut self, power: i8) -> Result<(), Self::Error> {
        self.options.power = power;
        todo!()
    }
}

impl radio::Busy for Rfm95x {
    type Error = RadioError;

    fn is_busy(&mut self) -> Result<bool, Self::Error> {
        todo!()
    }
}
