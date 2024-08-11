//! Implementation of [`MapArea`] and [`MemorySet`].

use crate::config::{MEMORY_END, MMIO, TRAMPOLINE};

use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use lazy_static::*;
use memory_set::*;
use page_table::{PhysAddr, VirtAddr};

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(unsafe { UPSafeCell::new(new_kernel()) });
}

/// Without kernel stacks.
fn new_kernel() -> MemorySet {
    let mut memory_set = MemorySet::new_bare();
    // map trampoline
    memory_set.map_trampoline(
        VirtAddr::from(TRAMPOLINE).into(),
        PhysAddr::from(strampoline as usize).into(),
    );

    // map kernel sections
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    println!(
        ".bss [{:#x}, {:#x})",
        sbss_with_stack as usize, ebss as usize
    );
    println!("mapping .text section");
    memory_set.push(
        MapArea::new(
            (stext as usize).into(),
            (etext as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::X,
        ),
        None,
    );
    println!("mapping .rodata section");
    memory_set.push(
        MapArea::new(
            (srodata as usize).into(),
            (erodata as usize).into(),
            MapType::Identical,
            MapPermission::R,
        ),
        None,
    );
    println!("mapping .data section");
    memory_set.push(
        MapArea::new(
            (sdata as usize).into(),
            (edata as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ),
        None,
    );
    println!("mapping .bss section");
    memory_set.push(
        MapArea::new(
            (sbss_with_stack as usize).into(),
            (ebss as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ),
        None,
    );
    println!("mapping physical memory");
    memory_set.push(
        MapArea::new(
            (ekernel as usize).into(),
            MEMORY_END.into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ),
        None,
    );
    println!("mapping memory-mapped registers");
    for pair in MMIO {
        memory_set.push(
            MapArea::new(
                (*pair).0.into(),
                ((*pair).0 + (*pair).1).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
    }
    memory_set
}
