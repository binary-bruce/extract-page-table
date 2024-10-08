//! The main module and entrypoint
//!
//! Various facilities of the kernels are implemented as submodules. The most
//! important ones are:
//!
//! - [`trap`]: Handles all cases of switching from userspace to the kernel
//! - [`task`]: Task management
//! - [`syscall`]: System call handling and implementation
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)
//!
//! We then call [`task::run_first_task()`] and for the first time go to
//! userspace.

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

// #[macro_use]
// extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod lang_items;
mod loader;
mod mm;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;

core::arch::global_asm!(include_str!("entry.asm"));
core::arch::global_asm!(include_str!("link_app.S"));

/// clear BSS segment
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    unsafe {
        let data = sbss as usize as *mut u8;
        let len = ebss as usize - sbss as usize;
        core::slice::from_raw_parts_mut(data, len).fill(0);
    }
}

#[no_mangle]
/// the rust entry-point of os
pub fn rust_main() -> ! {
    println!("[kernel] clear_bss");
    clear_bss();

    println!("[kernel] Hello, world!");

    println!("[kernel] mm::init");
    mm::init();

    println!("[kernel] trap::init");
    trap::init();

    println!("[kernel] trap::enable_timer_interrupt");
    //trap::enable_interrupt();
    trap::enable_timer_interrupt();

    println!("[kernel] timer::set_next_trigger");
    timer::set_next_trigger();

    println!("[kernel] task::run_first_task");
    task::run_first_task();

    panic!("Unreachable in rust_main!");
}
