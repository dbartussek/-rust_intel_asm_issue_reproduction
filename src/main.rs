#![no_std]
#![no_main]
#![feature(asm)]

///! A small reproduction case
///!
///! Build with `cargo xbuild --target x86_64-unknown-uefi -Z unstable-options --profile kernel_debug`

mod sub_module;

use core::panic::PanicInfo;
use sub_module::call_closure_with_stack;

#[no_mangle]
pub unsafe fn _start(stack: *mut u8) {
    call_closure_with_stack(|| (), stack);
}

#[no_mangle]
pub unsafe fn efi_main(stack: *mut u8) {
    call_closure_with_stack(|| (), stack);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}
