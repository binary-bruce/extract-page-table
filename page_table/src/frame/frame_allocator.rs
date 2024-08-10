use lazy_static::*;

use crate::{
    addr::PhysPageNum,
    frame::{
        frame_allocator_trait::FrameAllocator, stack_frame_allocator::FrameAllocatorImpl,
        up_safe_cell::UPSafeCell,
    },
};

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
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}
