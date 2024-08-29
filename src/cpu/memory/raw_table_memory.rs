use std::fmt::{Debug, Formatter};

use super::{
    memory_core::MEMORY_SIZE,
    raw_page_storage::{Page, RawPageMemory, RawPageStorage, PAGE_SIZE, PAGE_SIZE_LOG2},
};

pub struct RawTableMemoryPageStorage {
    pages: [Option<Box<Page>>;
        (((MEMORY_SIZE as u64) + (PAGE_SIZE as u64) - 1) / (PAGE_SIZE as u64)) as usize],
}

impl Debug for RawTableMemoryPageStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.pages.len())
    }
}

impl Default for RawTableMemoryPageStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl RawPageStorage for RawTableMemoryPageStorage {
    fn new() -> Self {
        RawTableMemoryPageStorage {
            pages: array_init::array_init(|_i| None),
        }
    }

    fn get_page_id(&self, addr: u32) -> u32 {
        addr >> PAGE_SIZE_LOG2
    }

    fn get_page(&self, page_id: u32) -> Option<&Page> {
        unsafe { self.pages.get_unchecked(page_id as usize).as_deref() }
    }

    fn get_page_mut(&mut self, page_id: u32) -> Option<&mut Page> {
        unsafe {
            self.pages
                .get_unchecked_mut(page_id as usize)
                .as_deref_mut()
        }
    }

    fn get_page_or_create(&mut self, page_id: u32) -> &mut Page {
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

pub type RawTableMemory = RawPageMemory<RawTableMemoryPageStorage>;
