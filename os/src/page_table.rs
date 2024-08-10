mod addr;
mod config;
mod frame;
mod page_table_entry;
mod pte_flags;

pub use addr::pa::*;
pub use addr::ppn::*;
pub use addr::simple_range::*;
pub use addr::va::*;
pub use addr::vpn::*;
pub use config::*;
use frame::frame_allocator::FRAME_ALLOCATOR;
pub use frame::frame_allocator::*;
pub use frame::frame_tracker::FrameTracker;
pub use page_table_entry::*;
pub use pte_flags::PTEFlags;

pub fn init_frame_allocator(from: usize, to: usize) {
    FRAME_ALLOCATOR
        .exclusive_access()
        .init(PhysAddr::from(from).ceil(), PhysAddr::from(to).floor());
}
