#![no_std]
#![no_main]

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use adafruit_feather_rp2040_rfm9x::{self as bsp, hal::gpio::*};

#[allow(unused_imports)]
use defmt::*;

use bsp::entry;
use defmt_rtt as _;
use embedded_hal::digital::OutputPin;
use panic_probe as _;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use bsp::{Pins, XOSC_CRYSTAL_FREQ};
use fugit::RateExtU32;

use phistar_radio::{BandwithOptions, RadioDevice, RadioOptions, Rfm95xPins};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let i2c = bsp::hal::i2c::I2C::i2c1(
        pac.I2C1,
        pins.sda.reconfigure(),
        pins.scl.reconfigure(),
        400.kHz(),
        &mut pac.RESETS,
        125_000_000.Hz(),
    );

    let radio_pins = Rfm95xPins {
        i2c,
        reset: &mut PinWrapper::from(pins.rfm_rst.into_push_pull_output()),
        dio5: &mut PinWrapper::from(pins.rfm_io5.into_push_pull_output()),
        dio4: &mut PinWrapper::from(pins.rfm_io4.into_push_pull_output()),
        dio3: &mut PinWrapper::from(pins.rfm_io3.into_push_pull_output()),
        dio2: &mut PinWrapper::from(pins.rfm_io2.into_push_pull_output()),
        dio1: &mut PinWrapper::from(pins.rfm_io1.into_push_pull_output()),
        dio0: &mut PinWrapper::from(pins.rfm_io0.into_push_pull_output()),
    };
    let radio_options = RadioOptions::builder()
        .power(2)
        .gain(4)
        .frequency(910.)
        .bandwith(BandwithOptions::Bw062_5)
        .build()
        .unwrap();
    let radio = RadioDevice::new(radio_pins, &radio_options)
        .expect("Unable to create radio device!")
        .cad();

    loop {}
}

struct PinWrapper<T>(Pin<T, FunctionSio<SioOutput>, <T as DefaultTypeState>::PullType>)
where
    T: DefaultTypeState + adafruit_feather_rp2040_rfm9x::hal::gpio::PinId;

impl<T> From<Pin<T, FunctionSio<SioOutput>, <T as DefaultTypeState>::PullType>> for PinWrapper<T>
where
    T: DefaultTypeState + adafruit_feather_rp2040_rfm9x::hal::gpio::PinId,
{
    fn from(value: Pin<T, FunctionSio<SioOutput>, <T as DefaultTypeState>::PullType>) -> Self {
        Self(value)
    }
}

impl<T> phistar_radio::PowerPin for PinWrapper<T>
where
    T: DefaultTypeState + adafruit_feather_rp2040_rfm9x::hal::gpio::PinId,
{
    fn set_high(&mut self) -> Result<(), core::convert::Infallible> {
        self.0.set_high()
    }
    fn set_low(&mut self) -> Result<(), core::convert::Infallible> {
        self.0.set_low()
    }
}
