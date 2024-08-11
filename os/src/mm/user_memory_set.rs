//! Implementation of [`MapArea`] and [`MemorySet`].

use crate::config::{TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};

use memory_set::*;
use page_table::{PhysAddr, VirtAddr, VirtPageNum, PAGE_SIZE};

extern "C" {
    fn strampoline();
}

/// Include sections in elf and trampoline and TrapContext and user stack,
/// also returns user_sp and entry point.
pub fn from_elf(elf_data: &[u8]) -> (MemorySet, usize, usize) {
    let mut memory_set = MemorySet::new_bare();
    // map trampoline
    memory_set.map_trampoline(
        VirtAddr::from(TRAMPOLINE).into(),
        PhysAddr::from(strampoline as usize).into(),
    );
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
