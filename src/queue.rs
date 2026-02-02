use esp_idf_hal::{
    peripheral::Peripheral,
    prelude::Peripherals,
    rmt::{PinState, RmtTransmitConfig, TxRmtDriver, VariableLengthSignal},
};
use esp_idf_sys::rmt_register_tx_end_callback;
use std::{
    ptr::null_mut, sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
    },
};

/// Atomic boolean tracking whether the transmitter is currently transmitting.
static TRANSMITTING: AtomicBool = AtomicBool::new(false);

/// Callback for when the transmitter finishes transmitting.
extern "C" fn transmit_finish(_channel: u32, _arg: *mut std::ffi::c_void) {
    TRANSMITTING.store(false, Ordering::Relaxed);
}

/// Queue struct
pub struct Queue {
    /// Transmitter queue
    tx: Sender<VariableLengthSignal>,
    rx: Receiver<VariableLengthSignal>,
    /// Driver for the transmitter
    driver: TxRmtDriver<'static>,
}

impl Queue {
    /// Create a new queue.
    ///
    /// # Panics
    ///
    /// This function will panic if executed more than once!
    ///
    pub unsafe fn new() -> Self {
        // register the transmit finish callback
        rmt_register_tx_end_callback(Some(transmit_finish), null_mut());

        // create channels
        let (tx, rx) = std::sync::mpsc::channel();

        // create the transmitter
        let mut config = RmtTransmitConfig::new();
        config = config
            .carrier(None)
            .clock_divider(10)
            .idle(Some(PinState::Low));

        let mut peripherals = Peripherals::take().unwrap();
        let driver = TxRmtDriver::new(
            unsafe { peripherals.rmt.channel1.clone_unchecked() },
            unsafe { peripherals.pins.gpio12.clone_unchecked() },
            &config,
        )
        .unwrap();

        Self {
            tx,
            rx,
            driver,
        }
    }

    /// Get the driver.
    pub fn driver(&self) -> &TxRmtDriver<'static> {
        &self.driver
    }

    /// Send a packet.
    pub fn send(&self, packet: VariableLengthSignal) {
        self.tx.send(packet).unwrap();
    }

    /// Tick the transmitter.
    pub fn tick(&mut self) {
        // skip if transmitting
        if TRANSMITTING.load(Ordering::Relaxed) {
            return;
        }

        // transmit all packets
        while let Ok(signal) = self.rx.try_recv() {
            TRANSMITTING.store(true, Ordering::Relaxed);
            self.driver.start(signal).unwrap();
        }
    }
}
