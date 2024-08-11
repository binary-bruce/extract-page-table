use memory_set::{MapArea, MapPermission, MapType, MemorySet};

pub struct MemorySetBuilder {
    memory_set: MemorySet,
}

impl MemorySetBuilder {
    pub fn new() -> Self {
        Self {
            memory_set: MemorySet::new_bare(),
        }
    }

    /// Identical map type only
    pub fn push_memory_area(
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

    pub fn map_trampoline(mut self, vpn: usize, ppn: usize) -> Self {
        self.memory_set.map_trampoline(vpn.into(), ppn.into());
        self
    }

    pub fn build(self) -> MemorySet {
        self.memory_set
    }
}
