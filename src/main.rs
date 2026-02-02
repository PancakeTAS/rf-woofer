use std::ptr::null_mut;

use esp_idf_sys::{esp, esp_vfs_dev_uart_use_driver, uart_driver_install};

fn main() {
    // setup the peripherals
    esp_idf_svc::sys::link_patches();

    unsafe {
        esp!(uart_driver_install(0, 512, 512, 10, null_mut(), 0)).unwrap();
        esp_vfs_dev_uart_use_driver(0);
    }

    // setup the logger
    esp_idf_svc::log::EspLogger::initialize_default();
    println!("meow :3 arf~");
}
