extern crate alloc;
use spin::RwLock;
use alloc::sync::Arc;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::cmp::Ordering;
use std::cmp::min;
use super::file_iter;
use super::entry;
use super::raw_entry;
use crate::buf;

pub const MAGIC_NUMBER: u32 = 0x2222ffff;

pub struct SSTableManager {
    pub sstable_num: usize,
    pub sstable_max_id: u32,
    pub cur_block_id: u32,
    pub block_num: usize,
    pub block_id: u32,
    pub files: BTreeMap<u32, (u32, usize)>,
    pub file_table: HashMap<u32, Vec<u32>>,
    pub block_table: HashMap<u32, bool>,
    pub file_iter: HashMap<u32, file_iter::FileIter>,
    pub buf: Arc<RwLock<buf::BufCache>>,
}

impl SSTableManager {
    pub fn new(block_id: u32, block_num: usize, buf: Arc<RwLock<buf::BufCache>>) -> SSTableManager {
        SSTableManager {
            buf,
            block_num,
            block_id,
            cur_block_id: block_id,
            sstable_max_id: 0,
            sstable_num: 0,
            files: BTreeMap::new(),
            file_table: HashMap::new(),
            block_table: HashMap::new(),
            file_iter: HashMap::new(),
        }
    }

    pub fn build(&mut self) {
        let mut index = self.block_id;
        while index <= self.block_id+self.block_num as u32 {
            let address = index * 128;
            let data = self.buf.write().read(0, address);
            if !(data[0] == 0x22 && data[1] == 0x22 && data[2] == 0xff && data[3] == 0xff) {
                index += 1;
                continue;
            }
            let num = data[4];
            let file_id = (data[5] as u32) << 16 | (data[6] as u32) << 8 | data[7] as u32;
            self.files.insert(file_id, (index, num as usize));
            self.sstable_num += 1;
            if file_id > self.sstable_max_id {
                self.sstable_max_id = file_id;
            }
            let mut blocks = vec![];
            for i in 0..num {
                self.block_table.insert(index + i as u32, true);
                blocks.push(index + i as u32)
            }
            self.file_table.insert(file_id, blocks);
            index += num as u32;
        }
        for i in self.block_id..self.block_id+self.block_num as u32 {
            if self.block_table.get(&i).is_some() && *self.block_table.get(&i).unwrap() {
                if i == self.block_id + self.block_num as u32 - 1 {
                    panic!()
                }
                continue;
            }
            self.cur_block_id = i;
            break;
        }
    }

    pub fn get(&mut self, key: &Vec<u8>) -> Option<Vec<u8>> {
        for (file_id, entry) in self.files.iter().rev() {
            let mut file_iter = self.file_iter.get_mut(file_id);
            if file_iter.is_none() {
                let mut file_iter = file_iter::FileIter::new(entry.0, entry.1, Arc::clone(&self.buf));
                if let Some(val) = file_iter.get(key) {
                    self.file_iter.insert(*file_id, file_iter);
                    return Some(val);
                }
                self.file_iter.insert(*file_id, file_iter);
            } else {
                if let Some(val) = file_iter.as_mut().unwrap().get(key) {
                    return Some(val);
                }
            }
        }
        None
    }

    pub fn flush(&mut self, entries: &Vec<entry::Entry>) {
        self.sstable_max_id += 1;
        self.sstable_num += 1;
        self.files.insert(self.sstable_max_id, (self.cur_block_id, 1));
        self.file_table.insert(self.sstable_max_id, vec![self.cur_block_id]);
        self.block_table.insert(self.cur_block_id, true);
        let mut page_data: [u8; 4096] = [0; 4096];
        page_data[0] = 0x22;
        page_data[1] = 0x22;
        page_data[2] = 0xff;
        page_data[3] = 0xff;
        page_data[4] = 1;
        page_data[5] = (self.sstable_max_id >> 16) as u8;
        page_data[6] = (self.sstable_max_id >> 8) as u8;
        page_data[7] = self.sstable_max_id as u8;
        let mut index = 0;
        let mut size = 12;
        for entry in entries.iter() {
            let mut raw_entry = raw_entry::Entry::new(entry.key.clone(), entry.value.clone());
            let data = raw_entry.encode_entry();
            let write_num = min(4096 - size, data.len());
            page_data[size..size+write_num].copy_from_slice(&data[..write_num]);
            size += write_num;
            if write_num != data.len() {
                self.buf.write().write(0, self.cur_block_id * 128 + index, &page_data);
                index += 1;
                size = 0;
                page_data = [0; 4096];
                let mut write_num = write_num;
                let mut remain_num = data.len() - write_num;
                while remain_num != 0 {
                    write_num = min(4096, remain_num);
                    page_data[..write_num].copy_from_slice(&data[data.len()-remain_num..data.len()-remain_num+write_num]);
                    size += write_num;
                    remain_num -= write_num;
                    if remain_num != 0 {
                        self.buf.write().write(0, self.cur_block_id * 128 + index, &page_data);
                        index += 1;
                        size = 0;
                        page_data = [0; 4096];
                    }
                }
            }
        }
        let mut eof_entry = raw_entry::Entry::new(raw_entry::EOF.as_bytes().to_vec(), raw_entry::EOF.as_bytes().to_vec());
        let data = eof_entry.encode_entry();
        page_data[size..size+data.len()].copy_from_slice(&data);
        self.buf.write().write(0, self.cur_block_id * 128 + index, &page_data);
        self.update_cur_block_id();
    }

    pub fn update_cur_block_id(&mut self) {
        if self.cur_block_id == self.block_id + self.block_num as u32 - 1 {
            self.cur_block_id = self.block_id;
        } else {
            self.cur_block_id += 1;
        }
        let mut loop_num = 0;
        while self.block_table.get(&self.cur_block_id).is_some() && *self.block_table.get(&self.cur_block_id).unwrap() {
            self.cur_block_id += 1;
            loop_num += 1;
            if loop_num == self.block_num {
                panic!();
            }
        }
    }

    // pub fn merge(&mut self) {
    //     // 这里有问题
    //     let mut a = None;
    //     let mut b = None;
    //     for entry in self.files.iter() {
    //         if a.is_none() {
    //             a = Some(entry.0);
    //         }
    //         if b.is_none() {
    //             b = Some(entry.0);
    //         }
    //         break;
    //     }
    //     let a_block = self.files.get(&a.unwrap()).unwrap();
    //     let b_block = self.files.get(&b.unwrap()).unwrap();
    //     let mut a_data = file_iter::FileIter::new(a_block.0, a_block.1, Arc::clone(&self.buf));
    //     let mut b_data = file_iter::FileIter::new(b_block.0, b_block.1, Arc::clone(&self.buf));
    //     let mut page_data: [u8; 4096] = [0; 4096];
    //     let mut index = 0;
    //     let mut size = 0;
    //     let mut a_key: Option<Vec<u8>> = None;
    //     let mut b_key: Option<Vec<u8>> = None;
    //     let mut a_value: Option<Vec<u8>> = None;
    //     let mut b_value: Option<Vec<u8>> = None;
    //     loop {
    //         if a_key.is_none() && a_data.has_next() {
    //             let res = a_data.next().unwrap();
    //             a_key = Some(res.key);
    //             a_value = Some(res.value);
    //         }
    //         if b_key.is_none() && b_data.has_next() {
    //             let res = b_data.next().unwrap();
    //             b_key = Some(res.key);
    //             b_value = Some(res.value);
    //         }
    //         if a_key.is_none() && b_key.is_none() && a_data.has_next() == false && b_data.has_next() == false {
    //             break;
    //         }
    //         let k: Vec<u8>;
    //         let v: Vec<u8>;
    //         if !a_key.is_none() && !b_key.is_none() {
    //             let cmp = a_key.clone().unwrap().cmp(&b_key.clone().unwrap());
    //             if cmp == Ordering::Equal {
    //                 k = b_key.unwrap();
    //                 v = b_value.unwrap();
    //                 a_key = None;
    //                 a_value = None;
    //                 b_key = None;
    //                 b_value = None;
    //             } else if cmp == Ordering::Greater {
    //                 k = b_key.unwrap();
    //                 v = b_value.unwrap();

    //                 b_key = None;
    //                 b_value = None;
    //             } else {
    //                 k = a_key.unwrap();
    //                 v = a_value.unwrap();

    //                 a_key = None;
    //                 a_value = None;
    //             }
    //         } else if !a_key.is_none() {
    //             k = a_key.unwrap();
    //             v = a_value.unwrap();
    //             a_key = None;
    //             a_value = None;
    //         } else {
    //             k = b_key.unwrap();
    //             v = b_value.unwrap();
    //             b_key = None;
    //             b_value = None;
    //         }
    //         let mut raw_entry = raw_entry::Entry::new(k, v);
    //         let data = raw_entry.encode_entry();
    //         if size + data.len() > 4096 {
    //             if index == 128 {
    //                 self.update_cur_block_id();
    //                 index = 0;
    //             }
    //             self.buf.write().write(0, self.cur_block_id * 128 + index as u32, &page_data);
    //             index += 1;
    //             size = 0;
    //             page_data = [0; 4096];
    //         }
    //         page_data[size..size+data.len()].copy_from_slice(&data);
    //         size += data.len();
    //     }
    //     let mut eof_entry = raw_entry::Entry::new(raw_entry::EOF.as_bytes().to_vec(), raw_entry::EOF.as_bytes().to_vec());
    //     let data = eof_entry.encode_entry();
    //     page_data[size..size+data.len()].copy_from_slice(&data);
    //     self.buf.write().write(0, self.cur_block_id * 128 + index as u32, &page_data);
    //     self.update_cur_block_id();
    // }

    pub fn clear(&mut self) {
        self.cur_block_id = 0;
        self.sstable_max_id = 0;
        self.sstable_num = 0;
        self.files = BTreeMap::new();
        self.file_table = HashMap::new();
        self.block_table = HashMap::new();
    }
}