use std::sync::Arc;
use std::cell::RefCell;
use crate::buf;
use super::memtable;
use super::raw_entry;
use super::sstable_manager;

pub struct LSMTree {
    memtable: memtable::Memtable,
    sstable_manager: sstable_manager::SSTableManager,
}

impl LSMTree {
    pub fn new(buf: Arc<RefCell<buf::BufCache>>) -> LSMTree {
        LSMTree {
            memtable: memtable::Memtable::new(128 * 4096),
            sstable_manager: sstable_manager::SSTableManager::new(10, 10, buf),
        }
    }

    pub fn flush(&mut self) {
        self.sstable_manager.flush(self.memtable.flush().clone());
    }

    pub fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>) {
        let mut entry = raw_entry::Entry::new(key.to_owned(), value.to_owned());
        if !self.memtable.can_put(entry.encode_entry().len()) {
            self.flush();
        }
        self.memtable.put(key, value);
    }

    pub fn get(&mut self, key: &Vec<u8>) -> Option<Vec<u8>> {
        match self.memtable.get(key) {
            Some(v) => {
                if v != raw_entry::TOMBSTONE.as_bytes().to_vec() {
                    return Some(v);
                } else {
                    return None;
                }
            }
            _ => {
                return self.sstable_manager.get(key);
            }
        }
    }

    pub fn delete(&mut self, key: &Vec<u8>) {
        self.memtable.put(key, &raw_entry::TOMBSTONE.as_bytes().to_vec());
    }
}