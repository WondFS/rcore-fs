use std::sync::Arc;
use std::cell::RefCell;
use super::raw_entry;
use super::block_iter;
use crate::buf;

pub struct FileIter {
    pub block_id: u32,
    pub block_num: usize,
    pub iter: usize,
    pub block_iter: block_iter::BlockIter,
    pub read_buf: Arc<RefCell<buf::BufCache>>,
}

impl FileIter {
    pub fn new(block_id: u32, block_num: usize, read_buf: Arc<RefCell<buf::BufCache>>) -> FileIter {
        let block_iter = block_iter::BlockIter::new(block_id, Arc::clone(&read_buf));
        FileIter {
            block_id,
            block_num,
            block_iter,
            read_buf,
            iter: 0,
        }
    }

    pub fn has_next(&mut self) -> bool {
        if self.block_iter.has_next() == false {
            if self.iter as usize == self.block_num - 1 {
                return false;
            } else {
                self.iter += 1;
                self.block_iter = block_iter::BlockIter::new(self.block_id+self.iter as u32, Arc::clone(&self.read_buf));
            }
        }
        true
    }

    pub fn next(&mut self) -> Option<raw_entry::Entry> {
        if self.has_next() == false {
            return None;
        }
        Some(self.block_iter.next())
    }

    pub fn get(&mut self, key: &Vec<u8>) -> Option<Vec<u8>> {
        while self.has_next() {
            let other = self.next().unwrap();
            if *key == other.key {
                return Some(other.value);
            }
        }
        None
    }
}