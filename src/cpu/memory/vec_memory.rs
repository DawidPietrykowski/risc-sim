use std::{fmt::Debug, fmt::Formatter};

use super::{
    memory_core::MEMORY_CAPACITY,
    page_storage::{Page, PageMemory, PageStorage, PAGE_SIZE},
};

pub struct VecPageStorage {
    pages: Vec<(u32, Page)>,
}

impl Debug for VecPageStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.pages.len())
    }
}

impl Default for VecPageStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl PageStorage for VecPageStorage {
    fn new() -> Self {
        VecPageStorage {
            pages: Vec::with_capacity(MEMORY_CAPACITY),
        }
    }

    fn get_page_id(&self, addr: u32) -> u32 {
        addr / PAGE_SIZE
    }

    fn get_page(&self, page_id: u32) -> Option<&Page> {
        self.pages.iter().find(|p| p.0 == page_id).map(|p| &p.1)
    }

    fn get_page_mut(&mut self, page_id: u32) -> Option<&mut Page> {
        self.pages
            .iter_mut()
            .find(|p| p.0 == page_id)
            .map(|p| &mut p.1)
    }

    fn get_page_or_create(&mut self, page_id: u32) -> &mut Page {
        if let Some(i) = self.pages.iter_mut().position(|p| p.0 == page_id) {
            return &mut self.pages[i].1;
        }

        let position = page_id * PAGE_SIZE;
        let index = self.pages.len();
        self.pages.push((page_id, Page::new(position)));
        &mut self.pages[index].1
    }

    fn len(&self) -> usize {
        self.pages.len()
    }
}

pub type VecMemory = PageMemory<VecPageStorage>;