use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter},
};

use super::page_storage::{Page, PageMemory, PageStorage, PAGE_SIZE, PAGE_SIZE_LOG2};

#[derive(Clone)]
pub struct BTreeStorage {
    pages: BTreeMap<u64, Box<Page>>,
}

impl Debug for BTreeStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.pages.len())
    }
}

impl Default for BTreeStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl PageStorage for BTreeStorage {
    fn new() -> Self {
        BTreeStorage {
            pages: BTreeMap::new(),
        }
    }

    fn get_page_id(&self, addr: u64) -> u64 {
        addr >> PAGE_SIZE_LOG2
    }

    fn get_page(&self, page_id: u64) -> Option<&Page> {
        self.pages
            .get(&page_id)
            .map(|boxed_page| boxed_page.as_ref())
    }

    fn get_page_mut(&mut self, page_id: u64) -> Option<&mut Page> {
        self.pages
            .get_mut(&page_id)
            .map(|boxed_page| boxed_page.as_mut())
    }

    fn get_page_or_create(&mut self, page_id: u64) -> &mut Page {
        self.pages
            .entry(page_id)
            .or_insert_with(|| Box::new(Page::new(page_id * PAGE_SIZE)))
    }

    fn len(&self) -> usize {
        self.pages.len()
    }
}

pub type BTreeMemory = PageMemory<BTreeStorage>;
