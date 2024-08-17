//! Rust wrapper around `__switch`.
//!
//! Switching to a different task's context happens here. The actual
//! implementation must not be in Rust and (essentially) has to be in assembly
//! language (Do you know why?), so this module really is just a wrapper around
//! `switch.S`.

use task::TaskContext;

core::arch::global_asm!(include_str!("switch.S"));

extern "C" {
    /// same with normal function call, except:
    /// 1. the stack(kernel) get changed
    /// 2. return based on `ra` value stored in the context of the next task
    pub fn __switch(current_task_cx: *mut TaskContext, next_task_cx: *const TaskContext);
}
