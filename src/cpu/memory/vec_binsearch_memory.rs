use std::{fmt::Debug, fmt::Formatter};

use super::page_storage::{Page, PageMemory, PageStorage, PAGE_SIZE};

pub struct VecBSearchPageStorage {
    pages: Vec<(u32, Page)>,
}

impl Debug for VecBSearchPageStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.pages.len())
    }
}

impl VecBSearchPageStorage {
    pub fn new() -> Self {
        VecBSearchPageStorage {
            pages: Vec::with_capacity(1024),
        }
    }
}

impl Default for VecBSearchPageStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl PageStorage for VecBSearchPageStorage {
    fn get_page_id(&self, addr: u32) -> u32 {
        addr / PAGE_SIZE
    }

    fn get_page(&self, page_id: u32) -> Option<&Page> {
        // self.pages.iter().find(|p| p.0 == page_id).map(|p| &p.1)
        let index = self.pages.binary_search_by(|p| p.0.cmp(&page_id));
        match index {
            Ok(i) => Some(&self.pages[i].1),
            Err(_) => None,
        }
    }

    fn get_page_mut(&mut self, page_id: u32) -> Option<&mut Page> {
        // self.pages
        //     .iter_mut()
        //     .find(|p| p.0 == page_id)
        //     .map(|p| &mut p.1)
        let index = self.pages.binary_search_by(|p| p.0.cmp(&page_id));
        match index {
            Ok(i) => Some(&mut self.pages[i].1),
            Err(_) => None,
        }
    }

    fn get_page_or_create(&mut self, page_id: u32) -> &mut Page {
        // if let Some(i) = self.pages.iter_mut().position(|p| p.0 == page_id) {
        let res = self.pages.binary_search_by(|p| p.0.cmp(&page_id));
        if let Ok(i) = res {
            return &mut self.pages[i].1;
        }

        return match res {
            Ok(i) => {
                &mut self.pages[i].1
            },
            Err(i) => {
                let position = page_id * PAGE_SIZE;
                let new_page_entry = (page_id, Page::new(position));
                self.pages.insert(i, new_page_entry);
                &mut self.pages[i].1
            }
        };



        // let res = self.pages.binary_search_by(|p| p.0.cmp(&page_id));
        // // assert!(res.is_err());
        // let index = res.err().unwrap();
        // self.pages.insert(index, new_page_entry);

    }

    fn len(&self) -> usize {
        self.pages.len()
    }
}

pub type VecBsearchMemory = PageMemory<VecBSearchPageStorage>;

impl VecBsearchMemory {
    pub fn new() -> Self {
        PageMemory {
            storage: VecBSearchPageStorage::new(),
        }
    }
}
