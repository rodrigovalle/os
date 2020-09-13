#![no_std] // avoid linking Rust standard library that makes use of OS features
#![no_main] // do away with linking crt0 to start any runtimes
#![feature(custom_test_frameworks)] // DIY no_std test framework
#![test_runner(os::test_runner)] // use custom test_runner fn
#![reexport_test_harness_main = "test_main"] // call this fn to run tests

use core::panic::PanicInfo;
use os::println;

/// The entry point of the OS called by the bootloader. This function never
/// returns because it's not well defined what this means (where do we return
/// to?).
#[no_mangle] // keep the function name in generated code
pub extern "C" fn _start() -> ! {
    os::init();

    #[cfg(test)]
    test_main();
    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

/// This function is called on panic from inside a unittest.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}
