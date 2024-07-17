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
