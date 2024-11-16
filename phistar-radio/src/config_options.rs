use super::RadioOptions;

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
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

impl Default for RadioOptions {
    fn default() -> Self {
        Self {
            power: 10,
            gain: 4,
            frequency: 910.,
            bandwith: BandwithOptions::Bw020_8,
        }
    }
}

impl RadioOptions {
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::default()
    }

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

#[derive(Clone, Copy, Debug, Default)]
pub struct OptionsBuilder(RadioOptions);
macro_rules! builder {
    ($b:ident,$f:ident,$t:ty) => {
        impl $b {
            pub fn $f(self, value: $t) -> Self {
                let mut new = self.clone();
                new.0.$f = value;
                new
            }
        }
    };
}
builder!(OptionsBuilder, power, u8);
builder!(OptionsBuilder, gain, u8);
builder!(OptionsBuilder, frequency, f32);
builder!(OptionsBuilder, bandwith, BandwithOptions);

impl OptionsBuilder {
    pub fn build(self) -> Option<RadioOptions> {
        if self.0.verify() {
            Some(self.0)
        } else {
            None
        }
    }
}
