use memory_set::{MapArea, MapPermission, MapType, MemorySet};
use page_table::{PhysAddr, VirtAddr};

pub(crate) struct MemorySetBuilder {
    memory_set: MemorySet,
}

impl MemorySetBuilder {
    pub(crate) fn new() -> Self {
        Self {
            memory_set: MemorySet::new_bare(),
        }
    }

    /// Identical map type only
    pub(crate) fn push_memory_area(
        mut self,
        start_va: usize,
        end_va: usize,
        map_perm: MapPermission,
    ) -> Self {
        self.memory_set.push(
            MapArea::new(start_va.into(), end_va.into(), MapType::Identical, map_perm),
            None,
        );

        return self;
    }

    pub(crate) fn map_trampoline(mut self, va: usize, pa: usize) -> Self {
        self.memory_set
            .map_trampoline(VirtAddr::from(va).into(), PhysAddr::from(pa).into());
        self
    }

    pub(crate) fn build(self) -> MemorySet {
        self.memory_set
    }
}
