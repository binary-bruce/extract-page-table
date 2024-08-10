use crate::page_table::frame::frame_allocator_trait::FrameAllocator;
use crate::page_table::frame::up_safe_cell::UPSafeCell;
use crate::page_table::{frame::stack_frame_allocator::FrameAllocatorImpl, PhysPageNum};

use lazy_static::*;

use super::frame_tracker::FrameTracker;

lazy_static! {
    /// frame allocator instance through lazy_static!
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}

/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}

/// deallocate a frame
pub(crate) fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}
