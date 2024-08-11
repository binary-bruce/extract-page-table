//! Implementation of [`MapArea`] and [`MemorySet`].

use crate::config::{MEMORY_END, MMIO, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};

use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use lazy_static::*;
use page_table::{PTEFlags, PageTable, PageTableEntry, PhysAddr, VirtAddr, VirtPageNum, PAGE_SIZE};
use riscv::register::satp;

use super::map_permission::MapPermission;
use super::map_type::MapType;
use super::memory_area::MapArea;

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
        Arc::new(unsafe { UPSafeCell::new(MemorySet::new_kernel()) });
}

/// memory set structure, controls virtual-memory space
pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    /// Assume that no conflicts.
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }
    /// Mention that trampoline is not collected by areas.
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }
    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
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
    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        // used in sbrk
        memory_set.push(
            MapArea::new(
                user_stack_top.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        // map TrapContext
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }
    #[allow(unused)]
    pub fn shrink_to(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
        if let Some(area) = self
            .areas
            .iter_mut()
            .find(|area| area.vpn_range.get_start() == start.floor())
        {
            area.shrink_to(&mut self.page_table, new_end.ceil());
            true
        } else {
            false
        }
    }
    #[allow(unused)]
    pub fn append_to(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
        if let Some(area) = self
            .areas
            .iter_mut()
            .find(|area| area.vpn_range.get_start() == start.floor())
        {
            area.append_to(&mut self.page_table, new_end.ceil());
            true
        } else {
            false
        }
    }
}

#[allow(unused)]
pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert!(!kernel_space
        .page_table
        .translate(mid_text.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .translate(mid_rodata.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .translate(mid_data.floor())
        .unwrap()
        .executable(),);
    println!("remap_test passed!");
}
