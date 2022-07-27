use super::kv_manager::KVManager;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct InodeMetadata {
    pub file_type: u8,
    pub ino: u32,
    pub size: u32,
    pub n_link: u8,
    pub last_accessed: u32,
    pub last_modified: u32,
    pub last_metadata_changed: u32,
}

pub struct KV {
    pub manager: KVManager,
    pub max_ino: u32,
}

impl KV {
    pub fn new() -> KV {
        KV {
            manager: KVManager::new(),
            max_ino: 0,
        }
    }

    pub fn allocate_indoe(&mut self, metadata: &mut InodeMetadata) -> u32 {
        self.max_ino += 1;
        metadata.ino = self.max_ino;
        let key = format!("m:{}", self.max_ino);
        let data = serde_json::to_vec(metadata).ok().unwrap();
        self.manager.set(&key, 0, 0, &data, 0);
        self.max_ino
    }

    pub fn delete_inode(&mut self, ino: u32) {
        let meta_key = format!("m:{}", ino);
        let data_key = format!("d:{}", ino);
        self.manager.delete(&meta_key, 0, 0, 0);
        self.manager.delete(&data_key, 0, 0, 0);
    }

    pub fn get_inode_metadata(&mut self, ino: u32) -> Option<InodeMetadata> {
        let key = format!("m:{}", ino);
        let data = self.manager.get(&key, 0, 0).unwrap();
        serde_json::from_slice(&data).ok()
    }

    pub fn set_inode_metadata(&mut self, ino: u32, metadata: &InodeMetadata) {
        let key = format!("m:{}", ino);
        let data = serde_json::to_vec(metadata).ok().unwrap();
        self.manager.set(&key, 0, 0, &data, 0);
    }

    pub fn get_inode_data(&mut self, ino: u32, off: usize, len: usize) -> Option<Vec<u8>> {
        let key = format!("d:{}", ino);
        self.manager.get(&key, off, len)
    }

    pub fn set_inode_data(&mut self, ino: u32, off: usize, len: usize, value: &Vec<u8>) -> usize {
        let mut metadata = self.get_inode_metadata(ino).unwrap();
        let key = format!("d:{}", ino);
        let size = self.manager.set(&key, off, len, value, metadata.ino).unwrap();
        metadata.size = size as u32;
        self.set_inode_metadata(ino, &metadata);
        size
    }

    pub fn delete_inode_data(&mut self, ino: u32, off: usize, len: usize) -> usize {
        let mut metadata = self.get_inode_metadata(ino).unwrap();
        let key = format!("d:{}", ino);
        let size = self.manager.delete(&key, off, len, metadata.ino).unwrap();
        metadata.size = size as u32;
        self.set_inode_metadata(ino, &metadata);
        size
    }

    pub fn get_extra_value(&mut self, key: String) -> Option<Vec<u8>> {
        let key = format!("e:{}", key);
        self.manager.get(&key, 0, 0)
    }

    pub fn set_extra_value(&mut self, key: String, value: &Vec<u8>) {
        let key = format!("e:{}", key);
        self.manager.set(&key, 0, 0, value, 0);
    }
}