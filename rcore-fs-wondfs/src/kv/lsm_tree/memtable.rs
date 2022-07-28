use std::collections::BTreeSet;
use super::entry;

pub struct Memtable {
    pub size: usize,
    pub threshold: usize,
    pub entries: BTreeSet<entry::Entry>,
}

impl Memtable {
    pub fn new(threshold: usize) -> Memtable {
        Memtable {
            threshold,
            size: 0,
            entries: BTreeSet::new(),
        }
    }

    pub fn get(&self, key: &Vec<u8>) -> Option<Vec<u8>> {
        let query = entry::Entry {
            key: key.to_owned(),
            value: vec![],
        };
        if let Some(entry) = self.entries.get(&query) {
            Some(entry.value.clone())
        } else {
            None
        }
    }

    pub fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>) {
        let query = entry::Entry {
            key: key.to_owned(),
            value: value.to_owned(),
        };
        let ret = self.entries.replace(query);
        self.size += key.len() + value.len() + 12;
        if ret.is_some() {
            self.size -= ret.unwrap().get_size() + 12;
        }
    }

    pub fn flush(&mut self) -> Vec<entry::Entry> {
        let mut entries: Vec<entry::Entry> = Vec::new();
        for entry in &self.entries {
            entries.push(entry.clone());
        }
        self.clear();
        entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.size = 0;
    }

    pub fn can_put(&self, size: usize) -> bool {
        // trick 25
        self.size + size <= self.threshold - 50
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
}