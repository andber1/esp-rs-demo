//! Driver for a RGB WS2812 LED

use core::time::Duration;
use esp_idf_svc::hal::rmt::{FixedLengthSignal, PinState, Pulse, TxRmtDriver};
use rgb::RGB8;

pub trait LedDriver {
    fn set_color(&mut self, color: RGB8) -> anyhow::Result<()>;
}

impl LedDriver for TxRmtDriver<'_> {
    fn set_color(&mut self, color: RGB8) -> anyhow::Result<()> {
        let rgb: u32 = (u32::from(color.b.reverse_bits()) << 16)
            + (u32::from(color.r.reverse_bits()) << 8)
            + u32::from(color.g.reverse_bits());

        let ticks_hz = self.counter_clock()?;
        let t0h = Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?;
        let t0l = Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(1000))?;
        let t1h = Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(1000))?;
        let t1l = Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(350))?;

        let mut signal = FixedLengthSignal::<24>::new();
        for i in 0..24 {
            let bit = 2_u32.pow(i) & rgb != 0;
            let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
            signal.set(i as usize, &(high_pulse, low_pulse))?;
        }
        self.start_blocking(&signal)?;
        Ok(())
    }
}

#[allow(dead_code)]
pub fn hue_to_color(hue: u8) -> RGB8 {
    let value = ((u16::from(hue) * 6) % 256) as u8;
    let sector = (u16::from(hue) * 6) / 256;
    match sector {
        0 => RGB8::new(255, value, 0),
        1 => RGB8::new(255 - value, 255, 0),
        2 => RGB8::new(0, 255, value),
        3 => RGB8::new(0, 255 - value, 255),
        4 => RGB8::new(value, 0, 255),
        5 => RGB8::new(255, 0, 255 - value),
        s => panic!("Unsupported color sector {s}"),
    }
}
