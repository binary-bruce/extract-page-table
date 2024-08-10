#![no_std]

mod addr;
mod config;
mod frame;
mod page_table;
mod page_table_entry;
mod pte_flags;

extern crate alloc;

pub use crate::page_table_entry::PageTableEntry;
pub use crate::pte_flags::PTEFlags;
pub use addr::*;
pub use config::*;
pub use frame::*;
pub use page_table::*;

pub fn init_frame_allocator(from: usize, to: usize) {
    FRAME_ALLOCATOR
        .exclusive_access()
        .init(PhysAddr::from(from).ceil(), PhysAddr::from(to).floor());
}
