extern crate alloc;
use spin::RwLock;
use alloc::sync::Arc;
use super::raw_entry;
use super::block_iter;
use crate::buf;

pub struct FileIter {
    pub block_id: u32,
    pub block_num: usize,
    pub block_iter: Vec<Option<block_iter::BlockIter>>,
    pub read_buf: Arc<RwLock<buf::BufCache>>,
}

impl FileIter {
    pub fn new(block_id: u32, block_num: usize, read_buf: Arc<RwLock<buf::BufCache>>) -> FileIter {
        let block_iter = vec![None; block_num];
        FileIter {
            block_id,
            block_num,
            block_iter,
            read_buf,
        }
    }

    pub fn get(&mut self, key: &Vec<u8>) -> Option<Vec<u8>> {
        for i in 0..self.block_num {
            if self.block_iter[i].is_none() {
                self.block_iter[i] = Some(block_iter::BlockIter::new(self.block_id+i as u32, Arc::clone(&self.read_buf)));
            }
            let ret = self.block_iter[i].as_ref().unwrap().get(key);
            if ret.is_some() {
                return Some(ret.unwrap());
            }
        }
        None
    }
}