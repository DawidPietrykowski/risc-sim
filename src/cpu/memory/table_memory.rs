use std::fmt::{Debug, Formatter};

use super::{
    memory_core::MEMORY_SIZE,
    page_storage::{Page, PageMemory, PageStorage, PAGE_SIZE, PAGE_SIZE_LOG2},
};

#[derive(Clone)]
pub struct TableMemoryPageStorage {
    pages: [Option<Box<Page>>; (((MEMORY_SIZE) + (PAGE_SIZE) - 1) / (PAGE_SIZE)) as usize],
}

impl Debug for TableMemoryPageStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.pages.len())
    }
}

impl Default for TableMemoryPageStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl PageStorage for TableMemoryPageStorage {
    fn new() -> Self {
        TableMemoryPageStorage {
            pages: array_init::array_init(|_i| None),
        }
    }

    fn get_page_id(&self, addr: u64) -> u64 {
        addr >> PAGE_SIZE_LOG2
    }

    fn get_page(&self, page_id: u64) -> Option<&Page> {
        unsafe { self.pages.get_unchecked(page_id as usize).as_deref() }
    }

    fn get_page_mut(&mut self, page_id: u64) -> Option<&mut Page> {
        unsafe {
            self.pages
                .get_unchecked_mut(page_id as usize)
                .as_deref_mut()
        }
    }

    fn get_page_or_create(&mut self, page_id: u64) -> &mut Page {
        unsafe {
            if self.pages.get_unchecked_mut(page_id as usize).is_some() {
                return self
                    .pages
                    .get_unchecked_mut(page_id as usize)
                    .as_deref_mut()
                    .unwrap();
            }

            let position = page_id * PAGE_SIZE;
            *self.pages.get_unchecked_mut(page_id as usize) = Some(Box::new(Page::new(position)));
            self.pages
                .get_unchecked_mut(page_id as usize)
                .as_deref_mut()
                .unwrap()
        }
    }

    fn len(&self) -> usize {
        self.pages.len()
    }
}

pub type TableMemory = PageMemory<TableMemoryPageStorage>;
