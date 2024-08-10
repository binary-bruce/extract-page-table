//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod heap_allocator;
mod memory_set;
mod page_table_util;

use crate::board::MEMORY_END;
use crate::page_table;
pub use crate::page_table::PhysPageNum;
pub use memory_set::remap_test;
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub use page_table_util::translated_byte_buffer;
use page_table_util::PageTable;

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    extern "C" {
        fn ekernel();
    }

    heap_allocator::init_heap();
    page_table::init_frame_allocator(ekernel as usize, MEMORY_END);
    KERNEL_SPACE.exclusive_access().activate();
}
