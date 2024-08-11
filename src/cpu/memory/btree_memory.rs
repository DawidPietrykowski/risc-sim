use std::{collections::BTreeMap, fmt::{Debug, Formatter}};

use super::page_storage::{Page, PageMemory, PageStorage, PAGE_SIZE};

pub struct BTreeStorage {
    pages: BTreeMap<u32, Page>,
}

impl Debug for BTreeStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.pages.len())
    }
}

impl BTreeStorage {
    pub fn new() -> Self {
        BTreeStorage {
            pages: BTreeMap::new(),
        }
    }
}

impl Default for BTreeStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl PageStorage for BTreeStorage {
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

pub type BTreeMemory = PageMemory<BTreeStorage>;

impl BTreeMemory {
    pub fn new() -> Self {
        PageMemory {
            storage: BTreeStorage::new(),
        }
    }
}
