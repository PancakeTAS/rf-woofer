use std::time::Duration;

use esp_idf_hal::rmt::{PinState, Pulse, TxRmtDriver, VariableLengthSignal};

/// Packet to send to the collar
///
/// Each packet is 32 bits in length and can be converted into a vector of bits:
/// [PREFIX        ] = SYNC
/// [TRANSMITTER ID] =     XXXXXXXXXXXXXXXX
/// [CHANNEL       ] =                     XXXX
/// [MODE          ] =                         XXXX
/// [STRENGTH      ] =                             XXXXXXXX
/// [CHECKSUM      ] =                                     XXXXXXXX
/// [END           ] =                                             000
#[derive(Debug, Clone)]
pub struct Packet {
    /// ID of the collar
    pub id: u16,
    /// Channel to send the message on
    pub channel: Channel,
    /// Action to perform
    pub action: Action,
    /// Intensity of the action (0-99)
    pub intensity: u8,
}

impl Into<Vec<bool>> for Packet {
    /// Convert a Packet to a vector of bits
    fn into(self) -> Vec<bool> {
        let mut bits = Vec::new();

        for i in (0..16).rev() {
            bits.push((self.id >> i) & 1 == 1);
        }

        let channel: [bool; 4] = self.channel.into();
        bits.extend(channel);

        let action: [bool; 4] = self.action.into();
        bits.extend(action);

        for i in (0..8).rev() {
            bits.push((self.intensity >> i) & 1 == 1);
        }

        let checksum = {
            let id_1 = bits[0..8].iter().fold(0, |acc, &x| (acc << 1) + x as u8);
            let id_2 = bits[8..16].iter().fold(0, |acc, &x| (acc << 1) + x as u8);
            let enums = bits[16..24].iter().fold(0, |acc, &x| (acc << 1) + x as u8);
            let intensity = bits[24..32].iter().fold(0, |acc, &x| (acc << 1) + x as u8);
            let checksum = id_1 as i32 + id_2 as i32 + enums as i32 + intensity as i32;
            let checksum = checksum % 256;
            let checksum: u8 = checksum.try_into().unwrap();

            let mut bits = Vec::new();
            for i in (0..8).rev() {
                bits.push((checksum >> i) & 1 == 1);
            }

            bits
        };
        bits.extend(checksum);

        for _ in 0..3 {
            bits.push(false);
        }

        bits
    }
}

/// Channels one can send a message on
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Channel {
    Zero,
    One,
    Two,
}

impl From<u8> for Channel {
    /// Convert a u8 to a Channel
    fn from(value: u8) -> Self {
        match value {
            0 => Channel::Zero,
            1 => Channel::One,
            2 => Channel::Two,
            _ => panic!("Invalid channel {}", value),
        }
    }
}

impl Into<[bool; 4]> for Channel {
    /// Convert a Channel to a vector of bits
    fn into(self) -> [bool; 4] {
        match self {
            Channel::Zero => [false, false, false, false],
            Channel::One => [false, false, false, true],
            Channel::Two => [false, false, true, false],
        }
    }
}

/// Actions one can perform
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Action {
    /// Shock the collar
    Shock = 1,
    /// Vibrate the collar
    Vibrate = 2,
    /// Beep the collar
    Beep = 3,
    /// Toggle the light on the collar
    Light = 4,
}

impl From<u8> for Action {
    /// Convert a u8 to an Action
    fn from(value: u8) -> Self {
        match value {
            1 => Action::Shock,
            2 => Action::Vibrate,
            3 => Action::Beep,
            4 => Action::Light,
            _ => panic!("Invalid action {}", value),
        }
    }
}

impl Into<[bool; 4]> for Action {
    /// Convert an Action to a vector of bits
    fn into(self) -> [bool; 4] {
        match self {
            Action::Shock => [false, false, false, true],
            Action::Vibrate => [false, false, true, false],
            Action::Beep => [false, false, true, true],
            Action::Light => [false, true, false, false],
        }
    }
}

/// Pulses used to encode bits
pub struct Pulses {
    sync_high: Pulse,
    sync_low: Pulse,
    one_high: Pulse,
    one_low: Pulse,
    zero_high: Pulse,
    zero_low: Pulse,
}

impl Pulses {
    /// Create a new set of pulses
    pub fn new(driver: &TxRmtDriver) -> Pulses {
        let ticks_hz = driver.counter_clock().unwrap();

        let create_pulse = |state, duration| {
            Pulse::new_with_duration(ticks_hz, state, &Duration::from_micros(duration)).unwrap()
        };
        Pulses {
            sync_high: create_pulse(PinState::High, 1400),
            sync_low:  create_pulse(PinState::Low,  800),
            zero_high: create_pulse(PinState::High, 300),
            zero_low:  create_pulse(PinState::Low,  800),
            one_high:  create_pulse(PinState::High, 800),
            one_low:   create_pulse(PinState::Low,  300),
        }
    }
    /// Encode a vector of bits into a signal
    pub fn encode(&self, bits: &Vec<bool>) -> VariableLengthSignal {
        let mut signal = VariableLengthSignal::with_capacity(1 + bits.len());
        signal
            .push([&self.sync_high, &self.sync_low])
            .unwrap();

        for bit in bits.iter() {
            if *bit {
                signal
                    .push([&self.one_high, &self.one_low])
                    .unwrap();
            } else {
                signal
                    .push([&self.zero_high, &self.zero_low])
                    .unwrap();
            }
        }
        return signal;
    }
}
