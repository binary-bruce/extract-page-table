//! Types related to task management
use memory_set::*;
use page_table::{PhysPageNum, VirtAddr};
use task::{TaskContext, TaskControlBlock, TaskStatus, TrapContext};

use crate::config::{kernel_stack_position, TRAP_CONTEXT};
use crate::mm::{from_elf, KERNEL_SPACE};
use crate::trap::{trap_handler, trap_return};

/// create a new task control block
pub fn new_tcb(elf_data: &[u8], app_id: usize) -> TaskControlBlock {
    // memory_set with elf program headers/trampoline/trap context/user stack
    let (memory_set, user_sp, entry_point) = from_elf(elf_data);

    let kernel_stack_top = map_kernel_stack(app_id);
    let trap_cx_ppn = get_trap_cx_ppn(&memory_set);
    let task_control_block = TaskControlBlock {
        task_status: TaskStatus::Ready,
        task_cx: TaskContext::init(trap_return as usize, kernel_stack_top),
        memory_set,
        trap_cx_ppn,
        base_size: user_sp,
        heap_bottom: user_sp,
        program_brk: user_sp,
    };

    // prepare TrapContext in user space
    let trap_cx = task_control_block.get_trap_cx();
    *trap_cx = TrapContext::app_init_context(
        entry_point,
        user_sp,
        KERNEL_SPACE.exclusive_access().token(),
        kernel_stack_top,
        trap_handler as usize,
    );

    task_control_block
}

fn get_trap_cx_ppn(memory_set: &MemorySet) -> PhysPageNum {
    let trap_context_vpn = VirtAddr::from(TRAP_CONTEXT).into();
    memory_set.translate(trap_context_vpn).unwrap().ppn()
}

/// map kernel stack area
/// return kernel_stack_top
fn map_kernel_stack(app_id: usize) -> usize {
    let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
    KERNEL_SPACE.exclusive_access().insert_framed_area(
        kernel_stack_bottom.into(),
        kernel_stack_top.into(),
        MapPermission::R | MapPermission::W,
    );

    kernel_stack_top
}
