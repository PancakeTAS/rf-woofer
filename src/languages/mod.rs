use std::cmp;

use esp_idf_hal::rmt::{TxRmtDriver, VariableLengthSignal};

mod socket;
mod collar;

pub use socket::Action as SocketAction;
pub use collar::Action as CollarAction;

/// Struct holding pulses for all languages
pub struct LanguageHolder {
    /// Remote socket pulses
    socket: socket::Pulses,
    /// Shock collar pulses
    collar: collar::Pulses
}

impl LanguageHolder {
    /// Create a new LanguageHolder
    pub fn new(drv: &TxRmtDriver) -> LanguageHolder {
        LanguageHolder {
            socket: socket::Pulses::new(drv),
            collar: collar::Pulses::new(drv),
        }
    }
    /// Craft a socket signal
    pub fn craft_socket(
        &self,
        id: u8,
        action: SocketAction
    ) -> VariableLengthSignal {
        // create packet
        let packet = socket::Packet {
            id,
            action
        };

        // encode packet
        let bits: Vec<bool> = packet.into();
        self.socket.encode(&bits)
    }
    /// Craft a collar signal with a given duration in milliseconds
    pub fn craft_collar(
        &self,
        id: u16,
        channel: u8,
        action: CollarAction,
        intensity: u8,
        duration: u16
    ) -> (VariableLengthSignal, u16) {
        // validate inputs
        let channel = cmp::min(channel, 2);
        let intensity = cmp::min(intensity, 99);
        let duration = cmp::max(300, cmp::min(duration, 10000));

        // create packet
        let channel = collar::Channel::from(channel);
        let packet = collar::Packet {
            id,
            channel,
            action,
            intensity
        };

        // encode packet
        let bits: Vec<bool> = packet.into();
        let signal = self.collar.encode(&bits);

        (signal, duration / 50)
    }
}
