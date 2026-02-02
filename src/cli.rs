use crate::{languages::{CollarAction, LanguageHolder}, queue::Queue};

/// API request structure
#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum Request {
    #[serde(rename = "socket")]
    Socket(Socket),
    #[serde(rename = "collar")]
    Collar(Collar),
}

/// API structure for a socket command
#[derive(serde::Deserialize)]
struct Socket {
    /// ID of the remote socket
    id: u8,
    /// Whether to turn it on or off
    state: bool
}

/// API structure for a collar command
#[derive(serde::Deserialize)]
struct Collar {
    /// ID of the collar
    id: u16,
    /// Channel of the collar
    channel: u8,
    /// Type of action to perform
    action: Action
}

/// Type of action to perform
#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum Action {
    /// Deliver a shock
    #[serde(rename = "shock")]
    Shock { intensity: u8, duration: u16 },
    /// Deliver a vibration
    #[serde(rename = "vibration")]
    Vibration { intensity: u8, duration: u16 },
    /// Beep the collar
    #[serde(rename = "beep")]
    Beep,
    /// Toggle the collar light
    #[serde(rename = "light")]
    Light { state: bool },
}

/// Process a command
pub fn process_command(command: &String, languages: &LanguageHolder, queue: &Queue) {
    // parse web request
    let request: Request = match serde_json::from_str(command) {
        Ok(req) => req,
        Err(e) => {
            println!("Malformed request received: {}", e);
            return;
        }
    };

    // convert request into actions
    let (signal, amount) = match request {
        Request::Socket(socket) => {
            let signal = languages.craft_socket(socket.id, socket.state.into());
            println!("Switching socket {} {}", socket.id, if socket.state { "ON" } else { "OFF" });

            (signal, 10)
        },
        Request::Collar(collar) => {
            let (action, intensity, duration) = match collar.action {
                Action::Shock { intensity, duration } => (CollarAction::Shock, intensity, duration),
                Action::Vibration { intensity, duration } => (CollarAction::Vibrate, intensity, duration),
                Action::Beep => (CollarAction::Beep, 50, 0),
                Action::Light { state } => (CollarAction::Light, if state { 1 } else { 0 }, 0)
            };

            let (signal, amount) = languages.craft_collar(collar.id, collar.channel, action, intensity, duration);
            println!("Collar command on ID {} channel {}: {:?} (intensity: {}, duration: {}ms)", collar.id, collar.channel, action, intensity, duration);

            (signal, amount)
        }
    };

    // send to queue
    for _ in 0..amount {
        queue.send(signal.clone());
    }
}
