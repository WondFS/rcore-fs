extern crate alloc;
use spin::RwLock;
use alloc::sync::Arc;
use super::raw_entry;
use crate::buf;
use std::cmp::min;

pub struct BlockIter {
    pub iter: usize,
    pub entries: Vec<raw_entry::Entry>,
    pub entry_num: usize,
}

impl BlockIter {
    pub fn new(block_id: u32, read_buf: Arc<RwLock<buf::BufCache>>) -> BlockIter {
        let eof_key = raw_entry::EOF.as_bytes().to_vec();
        let eof_value = raw_entry::EOF.as_bytes().to_vec();
        let mut entries = vec![];
        let mut entry_num = 0;
        let mut is_end = false;
        let mut crc32: Option<u32> = None;
        let mut key_size: Option<u32> = None;
        let mut value_size: Option<u32> = None;
        let mut key: Option<Vec<u8>> = None;
        let mut value: Option<Vec<u8>> = None;
        let mut read_index: usize = 0;
        let mut temp: Vec<u8> = vec![];
        for i in 0..128 {
            if is_end {
                break;
            }
            let page_data = read_buf.write().read(0, block_id+i);
            let mut j = 0;
            while j < 4096 {
                match read_index {
                    0 => {
                        let remain_num = 4 - temp.len();
                        let read_num = min(remain_num, 4096-j);
                        temp.append(&mut page_data[j..j+read_num].to_vec());
                        j += read_num;
                        if temp.len() == 4 {
                            crc32 = Some(BlockIter::decode_u32(&temp));
                            temp.clear();
                            read_index += 1;
                        }
                    },
                    1 => {
                        let remain_num = 4 - temp.len();
                        let read_num = min(remain_num, 4096-j);
                        temp.append(&mut page_data[j..j+read_num].to_vec());
                        j += read_num;
                        if temp.len() == 4 {
                            key_size = Some(BlockIter::decode_u32(&temp));
                            temp.clear();
                            read_index += 1;
                        }
                    },
                    2 => {
                        let remain_num = 4 - temp.len();
                        let read_num = min(remain_num, 4096-j);
                        temp.append(&mut page_data[j..j+read_num].to_vec());
                        j += read_num;
                        if temp.len() == 4 {
                            value_size = Some(BlockIter::decode_u32(&temp));
                            temp.clear();
                            read_index += 1;
                        }
                    },
                    3 => {
                        let remain_num = *key_size.as_ref().unwrap() as usize - temp.len();
                        let read_num = min(remain_num, 4096-j);
                        temp.append(&mut page_data[j..j+read_num].to_vec());
                        j += read_num;
                        if temp.len() == *key_size.as_ref().unwrap() as usize {
                            key = Some(temp.clone());
                            temp.clear();
                            read_index += 1;
                        }
                    },
                    4 => {
                        let remain_num = *value_size.as_ref().unwrap() as usize - temp.len();
                        let read_num = min(remain_num, 4096-j);
                        temp.append(&mut page_data[j..j+read_num].to_vec());
                        j += read_num;
                        if temp.len() == *value_size.as_ref().unwrap() as usize {
                            value = Some(temp.clone());
                            temp.clear();
                            read_index += 1;
                        }
                    },
                    5 => {
                        read_index = 0;
                        if *key.as_ref().unwrap() == eof_key && *value.as_ref().unwrap() == eof_value {
                            is_end = true;
                            break;
                        }
                        if !BlockIter::verify_data(crc32.unwrap(), value.as_ref().unwrap()) {
                            continue;
                        }
                        entry_num += 1;
                        entries.push(raw_entry::Entry::new(key.as_ref().unwrap().to_owned(), value.as_ref().unwrap().to_owned()));
                    },
                    _ => panic!(),
                }
            }
        }
        BlockIter {
            entries,
            entry_num,
            iter: 0,
        }
    }

    pub fn has_next(&self) -> bool {
        self.iter < self.entry_num
    }

    pub fn next(&mut self) -> raw_entry::Entry {
        let ret = self.iter;
        self.iter += 1;
        self.entries[ret].clone()
    }

    pub fn verify_data(_: u32, _: &Vec<u8>) -> bool {
        true
    }

    pub fn decode_u32(data: &Vec<u8>) -> u32 {
        if data.len() != 4 {
            panic!();
        }
        (data[0] as u32) << 24 + (data[1] as u32) << 16 + (data[2] as u32) << 8 + data[3] as u32
    }
}