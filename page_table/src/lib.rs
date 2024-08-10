#![no_std]

mod addr;
mod config;
mod page_table_entry;
mod pte_flags;

extern crate alloc;

pub use crate::page_table_entry::PageTableEntry;
pub use crate::pte_flags::PTEFlags;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);

        let a = PTEFlags::V;
        assert_eq!(PTEFlags::V, a);
    }
}
