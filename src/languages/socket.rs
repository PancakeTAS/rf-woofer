use std::time::Duration;

use esp_idf_hal::rmt::{PinState, Pulse, TxRmtDriver, VariableLengthSignal};

/// Packet to send to the remote socket
///
/// Each packet is 24 bits in length and can be converted into a vector of bits:
/// [PREFIX] = SYNC
/// [ACTION] =     XXXXXXXXXXXXXXXXXXXX
/// [ID    ] =                         XXXX
#[derive(Debug, Clone)]
pub struct Packet {
    /// ID of the remote socket
    pub id: u8,
    /// Action to perform
    pub action: Action
}

impl Into<Vec<bool>> for Packet {
    /// Convert a Packet to a vector of bits
    fn into(self) -> Vec<bool> {
        let mut bits = Vec::new();

        let action: [bool; 20] = self.action.into();
        bits.extend(action);

        for i in (0..4).rev() {
            bits.push((self.id >> i) & 1 == 1);
        }

        bits
    }
}

/// Actions one can perform
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Action {
    /// Enable the remote socket
    Enable = 1,
    /// Disable the remote socket
    Disable = 2
}

impl From<u8> for Action {
    /// Convert a u8 to an Action
    fn from(value: u8) -> Self {
        match value {
            1 => Action::Enable,
            2 => Action::Disable,
            _ => panic!("Invalid action {}", value),
        }
    }
}

impl From<bool> for Action {
    /// Convert a bool to an Action
    fn from(value: bool) -> Self {
        if value {
            Action::Enable
        } else {
            Action::Disable
        }
    }
}

impl Into<[bool; 20]> for Action {
    /// Convert an Action to a vector of bits
    fn into(self) -> [bool; 20] {
        match self {
            Action::Enable => [
                false, false, false, false,
                false, false, false,  true,
                 true,  true,  true,  true,
                 true, false, false, false,
                false,  true,  true,  true,
            ],
            Action::Disable => [
                false, false, false, false,
                false, false,  true, false,
                false, false,  true,  true,
                 true,  true,  true,  true,
                false, false, false,  true,
            ],
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
            sync_high: create_pulse(PinState::High, 380),
            sync_low:  create_pulse(PinState::Low,  2280),
            zero_high: create_pulse(PinState::High, 380),
            zero_low:  create_pulse(PinState::Low,  1140),
            one_high:  create_pulse(PinState::High, 1140),
            one_low:   create_pulse(PinState::Low,  380),
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
