use rustc_hash::{FxBuildHasher, FxHashMap};

use super::{
    memory_core::MEMORY_CAPACITY,
    page_storage::{Page, PageMemory, PageStorage, PAGE_SIZE},
};

pub struct FxHashStorage {
    pages: FxHashMap<u32, Page>,
}

impl FxHashStorage {
    pub fn new() -> Self {
        FxHashStorage {
            pages: FxHashMap::with_capacity_and_hasher(MEMORY_CAPACITY, FxBuildHasher),
        }
    }
}

impl Default for FxHashStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageStorage for FxHashStorage {
    fn get_page_id(&self, addr: u32) -> u32 {
        addr / PAGE_SIZE
    }

    fn get_page(&self, page_id: u32) -> Option<&Page> {
        self.pages.get(&page_id)
    }

    fn get_page_mut(&mut self, page_id: u32) -> Option<&mut Page> {
        self.pages.get_mut(&page_id)
    }

    fn get_page_or_create(&mut self, page_id: u32) -> &mut Page {
        self.pages
            .entry(page_id)
            .or_insert_with(|| Page::new(page_id * PAGE_SIZE))
    }

    fn len(&self) -> usize {
        self.pages.len()
    }
}

pub type FxHashMemory = PageMemory<FxHashStorage>;

impl FxHashMemory {
    pub fn new() -> Self {
        PageMemory {
            storage: FxHashStorage::new(),
        }
    }
}

impl Default for FxHashMemory {
    fn default() -> Self {
        Self::new()
    }
}
