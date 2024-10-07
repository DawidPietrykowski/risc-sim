use rustc_hash::{FxBuildHasher, FxHashMap};

use super::{
    memory_core::MEMORY_CAPACITY,
    page_storage::{Page, PageMemory, PageStorage, PAGE_SIZE, PAGE_SIZE_LOG2},
};

#[derive(Clone)]
pub struct FxHashStorage {
    pages: FxHashMap<u64, Page>,
}

impl Default for FxHashStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageStorage for FxHashStorage {
    fn new() -> Self {
        FxHashStorage {
            pages: FxHashMap::with_capacity_and_hasher(MEMORY_CAPACITY, FxBuildHasher),
        }
    }

    fn get_page_id(&self, addr: u64) -> u64 {
        addr >> PAGE_SIZE_LOG2
    }

    fn get_page(&self, page_id: u64) -> Option<&Page> {
        self.pages.get(&page_id)
    }

    fn get_page_mut(&mut self, page_id: u64) -> Option<&mut Page> {
        self.pages.get_mut(&page_id)
    }

    fn get_page_or_create(&mut self, page_id: u64) -> &mut Page {
        self.pages
            .entry(page_id)
            .or_insert_with(|| Page::new(page_id * PAGE_SIZE))
    }

    fn len(&self) -> usize {
        self.pages.len()
    }
}

pub type FxHashMemory = PageMemory<FxHashStorage>;

impl Default for FxHashMemory {
    fn default() -> Self {
        Self::new()
    }
}
