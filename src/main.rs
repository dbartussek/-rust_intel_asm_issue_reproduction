#![feature(asm)]

///! A small reproduction case
///!
///! Build with `cargo build`
///!
///! Notes:
///! - setting opt-level to 0 or 1 compiles
///! - opt-level 2, 3, "s", "z" cause the error
///! - a release build works just fine, just a debug build with optimization has the issue

use std::mem::{ManuallyDrop, MaybeUninit};

pub unsafe fn call_with_stack<T>(
    arg: &mut T,
    function: extern "sysv64" fn(&mut T) -> (),
    stack: *mut u8,
) {
    asm!(r#"
    mov rbp, rsp
    mov rsp, $2

    call $1

    mov rsp, rbp
    "#
    : // Return values
    : "{rdi}"(arg), "r"(function), "r"(stack) // Arguments
    : "rbp", "cc", "memory" // Clobbers
    : "volatile", "intel" // Options
    );
}

/// Calls a closure and returns the result
///
/// This function is unsafe because it changes the stack pointer to stack.
/// stack must be suitable to be used as a stack pointer on the target system.
pub unsafe fn call_closure_with_stack<F, R>(closure: F, stack: *mut u8) -> R
where
    F: FnOnce() -> R,
{
    extern "sysv64" fn inner<F, R>(data: &mut (ManuallyDrop<F>, MaybeUninit<R>))
    where
        F: FnOnce() -> R,
    {
        let result = {
            // Read the closure from context, taking ownership of it
            let function = unsafe { ManuallyDrop::take(&mut data.0) };

            // Call the closure.
            // This consumes it and returns the result
            function()
        };

        // Write the result into the context
        data.1 = MaybeUninit::new(result);
    }

    // The context contains the closure and uninitialized memory for the return value
    let mut context = (ManuallyDrop::new(closure), MaybeUninit::uninit());

    call_with_stack(
        &mut context,
        // We create a new, internal function that does not close over anything
        // and takes a context reference as its argument
        inner,
        stack,
    );

    // Read the result from the context
    // No values are in the context anymore afterwards
    context.1.assume_init()
}

#[no_mangle]
pub unsafe fn external(stack: *mut u8) {
    call_closure_with_stack(|| (), stack);
}

pub fn main() {
    unsafe {
        call_closure_with_stack(|| (), std::ptr::null_mut());
    }
}
