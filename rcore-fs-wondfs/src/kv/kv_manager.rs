use std::sync::Arc;
use std::cell::RefCell;
use std::cmp::max;
use crate::buf;
use super::lsm_tree::lsm_tree;
use serde::{Serialize, Deserialize};

pub enum KVOperationsObject {
    MetaObject,
    DataObject,
    ExtraObject,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DataObjectValueEntry {
    pub len: usize,
    pub offset: usize,
    pub page_pointer: u32,
}

#[derive(Serialize, Deserialize)]
pub struct DataObjectValue {
    pub size: usize,
    pub entries: Vec<DataObjectValueEntry>,
}

pub struct KVManager {
    pub buf: Arc<RefCell<buf::BufCache>>,
    pub lsm_tree: lsm_tree::LSMTree,
}

impl KVManager {
    pub fn new() -> KVManager {
        let buf = Arc::new(RefCell::new(buf::BufCache::new()));
        KVManager {
            lsm_tree: lsm_tree::LSMTree::new(Arc::clone(&buf)),
            buf,
        }
    }

    pub fn get(&mut self, key: &String, off: usize, len: usize) -> Option<Vec<u8>> {
        let operation_type = KVManager::parse_key(key);
        match operation_type {
            KVOperationsObject::MetaObject => {
                let value = self.lsm_tree.get(&key.as_bytes().to_vec());
                if value.is_none() {
                    return None;
                }
                if len != 0 {
                    Some(value.unwrap()[off..off+len].to_vec())
                } else {
                    value
                }
            },
            KVOperationsObject::DataObject => {
                let value = self.lsm_tree.get(&key.as_bytes().to_vec());
                if value.is_none() {
                    return None;
                }
                let mut data_object: DataObjectValue = serde_json::from_slice(&value.unwrap()).unwrap();
                if len != 0 {
                    Some(self.read_data_object_all(&mut data_object))
                } else {
                    if off + len > data_object.size {
                        return Some(self.read_data_object_all(&mut data_object)[off..].to_vec());
                    }
                    KVManager::sort_data_object(&mut data_object);
                    let mut index = 0;
                    for (i, entry) in data_object.entries.iter().enumerate() {
                        if off < entry.offset {
                            index = i - 1;
                            break;
                        }
                    }
                    let mut remain_num = len;
                    let mut result = vec![];
                    while remain_num != 0 {
                        let mut data = self.read_data_object_entry(&data_object.entries[index]);
                        result.append(&mut data);
                        remain_num -= data.len();
                        index += 1;
                    }
                    Some(result)
                }
            },
            KVOperationsObject::ExtraObject => {
                let value = self.lsm_tree.get(&key.as_bytes().to_vec());
                if value.is_none() {
                    return None;
                }
                if len != 0 {
                    Some(value.unwrap()[off..off+len].to_vec())
                } else {
                    value
                }
            },
        }
    }

    pub fn set(&mut self, key: &String, off: usize, len: usize, value: &Vec<u8>) {
        let operation_type = KVManager::parse_key(key);
        match operation_type {
            KVOperationsObject::MetaObject => {
                let pre_value = self.lsm_tree.get(&key.as_bytes().to_vec());
                if pre_value.is_none() {
                    self.lsm_tree.put(&key.as_bytes().to_vec(), value);
                }
                if len != 0 {
                    let mut pre_value = pre_value.unwrap();
                    if pre_value.len() >= off + len {
                        pre_value[off..off+len].copy_from_slice(value);
                    } else {
                        pre_value.truncate(off);
                        pre_value.append(&mut value.clone());
                    }
                    self.lsm_tree.put(&key.as_bytes().to_vec(), &pre_value);
                } else {
                    self.lsm_tree.put(&key.as_bytes().to_vec(), value);
                }
            },
            KVOperationsObject::DataObject => {
                let value = self.lsm_tree.get(&key.as_bytes().to_vec());
                if value.is_none() {
                    // 更新value
                }
                let mut data_object: DataObjectValue = serde_json::from_slice(&value.unwrap()).unwrap();
            },
            KVOperationsObject::ExtraObject => {
                let pre_value = self.lsm_tree.get(&key.as_bytes().to_vec());
                if pre_value.is_none() {
                    self.lsm_tree.put(&key.as_bytes().to_vec(), value);
                }
                if len != 0 {
                    let mut pre_value = pre_value.unwrap();
                    if pre_value.len() >= off + len {
                        pre_value[off..off+len].copy_from_slice(value);
                    } else {
                        pre_value.truncate(off);
                        pre_value.append(&mut value.clone());
                    }
                    self.lsm_tree.put(&key.as_bytes().to_vec(), &pre_value);
                } else {
                    self.lsm_tree.put(&key.as_bytes().to_vec(), value);
                }
            },
        }
    }

    pub fn delete(&mut self, key: &String, off: usize, len: usize, value: &Vec<u8>) {
        let operation_type = KVManager::parse_key(key);
        match operation_type {
            KVOperationsObject::MetaObject => {
                let pre_value = self.lsm_tree.get(&key.as_bytes().to_vec());
                if pre_value.is_none() {
                    return;
                }
                if len != 0 {
                    let mut pre_value = pre_value.unwrap();
                    if pre_value.len() >= off + len {
                        pre_value[off..off+len].copy_from_slice(value);
                    } else {
                        pre_value.truncate(off);
                        pre_value.append(&mut value.clone());
                    }
                    self.lsm_tree.put(&key.as_bytes().to_vec(), &pre_value);
                } else {
                    self.lsm_tree.delete(&key.as_bytes().to_vec());
                }
            },
            KVOperationsObject::DataObject => {

            },
            KVOperationsObject::ExtraObject => {
                let pre_value = self.lsm_tree.get(&key.as_bytes().to_vec());
                if pre_value.is_none() {
                    return;
                }
                if len != 0 {
                    let mut pre_value = pre_value.unwrap();
                    if pre_value.len() >= off + len {
                        pre_value[off..off+len].copy_from_slice(value);
                    } else {
                        pre_value.truncate(off);
                        pre_value.append(&mut value.clone());
                    }
                    self.lsm_tree.put(&key.as_bytes().to_vec(), &pre_value);
                } else {
                    self.lsm_tree.delete(&key.as_bytes().to_vec());
                }
            },
        }
    }

    pub fn parse_key(key: &String) -> KVOperationsObject {
        match &key[0..2] {
            "m:" => KVOperationsObject::MetaObject,
            "d:" => KVOperationsObject::DataObject,
            "e:" => KVOperationsObject::ExtraObject,
            _ => panic!(),
        }
    }
}

impl KVManager {
    pub fn set_data_object(&mut self, object: &mut DataObjectValue, off: usize, len: usize, value: &Vec<u8>) {
        KVManager::sort_data_object(object);
        let mut index = 0;
        let mut flag = false;
        let new_entry = DataObjectValueEntry {
            len,
            offset: off,
            page_pointer: 0,
        };
        // let mut second_entry = None;
        // let mut second_o_entry = None;
        let mut second_index = 0;
        if off > object.size {
            return;
        }
        for entry in object.entries.iter() {
            if entry.offset + entry.len <= new_entry.offset {
                index += 1;
                continue;
            } else if entry.offset >= new_entry.offset + new_entry.len {
                break;
            }
            let valid_prev = max(0, new_entry.offset as i32 - entry.offset as i32) as usize;
            let valid_suffix = max(0, entry.offset as i32 + entry.len as i32 - new_entry.offset as i32 - new_entry.len as i32) as usize;
            if valid_prev == 0 {
                // todo
            } else {
                // todo
            }
            index += 1;
            if !flag {
                // todo
            }
            if valid_suffix > 0 {
                // todo
            }
        }
        if !flag {
            // todo
        }
        // if second_entry.is_some() {
        //     // todo
        // }
    }

    pub fn delete_data_object(&mut self, object: &mut DataObjectValue) {
        
    }
}

impl KVManager {
    pub fn read_data_object_all(&mut self, object: &mut DataObjectValue) -> Vec<u8> {
        KVManager::sort_data_object(object);
        let mut result = vec![];
        for entry in object.entries.iter() {
            result.append(&mut self.read_data_object_entry(entry));
        }
        result
    }

    pub fn read_data_object_entry(&mut self, entry: &DataObjectValueEntry) -> Vec<u8> {
        let mut data = vec![];
        for i in 0..(entry.len-1/4096)+1 {
            let page_data = self.buf.borrow_mut().read(0, entry.page_pointer + i as u32);
            data.append(&mut page_data.to_vec());
        }
        data[..entry.len].to_vec()
    }

    pub fn sort_data_object(object: &mut DataObjectValue) {
        let len = object.entries.len();
        for i in 0..len {
            for j in 0..len - 1 - i {
                let index_1 = object.entries[j].offset;
                let index_2 = object.entries[j+1].offset;
                if index_1 > index_2 {
                    let temp = object.entries[j];
                    object.entries[j] = object.entries[j+1];
                    object.entries[j+1] = temp;
                }
            }
        }
    }

}

impl KVManager {
    pub fn find_write_pos(&mut self, size: u32) -> u32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        
    }
}