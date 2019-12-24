#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rn2xx3;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Use something from the crate
    let _ = rn2xx3::Freq433;
    loop {}
}
