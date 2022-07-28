extern crate alloc;
use spin::RwLock;
use alloc::sync::Arc;
use crate::buf;
use super::memtable;
use super::entry;
use super::sstable_manager;

pub struct LSMTree {
    memtable: memtable::Memtable,
    sstable_manager: sstable_manager::SSTableManager,
}

impl LSMTree {
    pub fn new(buf: Arc<RwLock<buf::BufCache>>) -> LSMTree {
        LSMTree {
            memtable: memtable::Memtable::new(128 * 4096),
            sstable_manager: sstable_manager::SSTableManager::new(5, 10, buf),
        }
    }

    pub fn flush(&mut self) {
        self.sstable_manager.flush(&self.memtable.flush());
    }

    pub fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>) {
        if !self.memtable.can_put(key.len() + value.len() + 12) {
            self.flush();
        }
        self.memtable.put(key, value);
    }

    pub fn get(&mut self, key: &Vec<u8>) -> Option<Vec<u8>> {
        match self.memtable.get(key) {
            Some(v) => {
                if v != entry::TOMBSTONE.as_bytes().to_vec() {
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
        let value = entry::TOMBSTONE.as_bytes().to_vec();
        if !self.memtable.can_put(key.len() + value.len() + 12) {
            self.flush();
        }
        self.memtable.put(key, &value);
    }
}