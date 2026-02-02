# rf-woofer
Control remote sockets & shock collars from a PiShock hub (as you can see this is a very use-case oriented project)

## Compilation (Arch Linux)

Install the required packages and ensure PATH is set:
```bash
$ pacman -S espup rustup libxml2-legacy
$ export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
```

Compile espflash:
```bash
$ rustup default stable
$ cargo install espflash
```

Download and install esp-rs:
```bash
$ espup install -f export-esp.sh
$ source export-esp.sh
```

Build & Flash rf-woofer
```bash
$ cargo +esp run --release --target xtensa-esp32-espidf
```

## Usage

After plugging in the device, you can send commands to it via JSON over serial.

For example, to turn on socket 1:
```json
{ "type": "socket", "id": 0, "state": true }
```

Or to vibrate the shock collar for 5 seconds:
```json
{ "type": "collar", "id": 1, "channel": 0, "action": { "type": "vibration", "intensity": 25, "duration": 5000 } }
```

In general, the `type` field specifies the device type (`socket` or `collar`), and the other fields depend on the device type and desired action.
- When controlling a socket, use `id` to specify the socket number (0-3) and `state` to turn it on (`true`) or off (`false`).
- When controlling a collar, use `id` to specify the collar number, channel (0-2) and `action` to specify the action to perform:
  - For vibration and shock, specify `intensity` (0-100) and `duration` (in milliseconds) in addition to the action type `vibration` or `shock`.
  - For beep, no additional parameters are needed beyond the action type `beep`.
  - For light, specify `state` and action type `light`.

## Credits

Large parts of the codebase originate from [Zebreus/serialcaixianlin](https://github.com/zebreus/serialcaixianlin/).
