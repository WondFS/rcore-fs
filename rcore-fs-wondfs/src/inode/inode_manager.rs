extern crate alloc;
use alloc::sync::Arc;
use crate::fs::filesystem;
use crate::kv::kv::InodeMetadata;
use super::inode::*;

pub type InodeLink = Arc<Inode>;

pub struct InodeManager {
    pub capacity: usize,
    pub inode_buffer: Vec<InodeLink>,
    pub fs: Arc<filesystem::WondFS>,
}

impl InodeManager {
    pub fn new(fs: Arc<filesystem::WondFS>) -> InodeManager {
        let mut buf = vec![];
        let capacity = 30;
        for _ in 0..capacity {
            buf.push(Arc::new(Inode::new(Arc::clone(&fs))));
        }
        InodeManager {
            fs,
            capacity,
            inode_buffer: buf,
        }
    }

    pub fn get_capacity(&self) -> u32 {
        self.capacity as u32
    }
}

impl InodeManager {
    pub fn i_alloc(&mut self) -> Option<InodeLink> {
        let mut empty_index = -1;
        for (index, ip) in self.inode_buffer.iter().enumerate() {
            if empty_index == -1 && ip.stat.read().ref_cnt == 0 {
                empty_index = index as i32;
            }
        }
        if empty_index == -1 {
            panic!("InodeManager: alloc no spare cache to store");
        }
        let mut inode_metadata = InodeMetadata {
            file_type: InodeFileType::File.into(),
            ino: 0,
            size: 0,
            n_link: 1,
            last_accessed: 0,
            last_modified: 0,
            last_metadata_changed: 0,
        };
        let ino = self.fs.kv.write().allocate_indoe(&mut inode_metadata);
        let inode_stat = InodeStat {
            ino,
            file_type: InodeFileType::File,
            size: 0,
            ref_cnt: 1,
            n_link: 1,
            last_accessed: 0,
            last_modified: 0,
            last_metadata_changed: 0,
        };
        let inode = Inode::new(Arc::clone(&self.fs));
        *inode.stat.write() = inode_stat;
        inode.validate();
        let link = Arc::new(inode);
        self.inode_buffer[empty_index as usize] = Arc::clone(&link);
        Some(link)
    }

    pub fn i_get(&mut self, ino: u32) -> Option<InodeLink> {
        let mut empty_index = -1;
        for (index, ip) in self.inode_buffer.iter().enumerate() {
            if ip.stat.read().ref_cnt > 0 && ip.stat.read().ino == ino {
                ip.stat.write().ref_cnt += 1;
                return Some(Arc::clone(ip));
            }
            if empty_index == -1 && ip.stat.read().ref_cnt == 0 {
                empty_index = index as i32;
            }
        }
        if empty_index == -1 {
            panic!("InodeManager: get no spare cache to store");
        }
        let metadata = self.fs.kv.write().get_inode_metadata(ino);
        if metadata.is_none() {
            return None;
        }
        let metadata = metadata.unwrap();
        let mut inode_stat = InodeStat {
            ino,
            file_type: InodeFileType::File,
            size: metadata.size,
            ref_cnt: 1,
            n_link: metadata.n_link,
            last_accessed: metadata.last_accessed,
            last_modified: metadata.last_modified,
            last_metadata_changed: metadata.last_metadata_changed,
        };
        if metadata.file_type == 1 {
            inode_stat.file_type = InodeFileType::Directory;
        }
        let inode = Inode::new(Arc::clone(&self.fs));
        *inode.stat.write() = inode_stat;
        inode.validate();
        let link = Arc::new(inode);
        self.inode_buffer[empty_index as usize] = Arc::clone(&link);
        Some(link)
    }

    pub fn i_put(&mut self, inode: InodeLink) {
        if inode.stat.read().ref_cnt > 0 {
            inode.stat.write().ref_cnt -= 1;
        }
        if inode.stat.read().ref_cnt == 0 {
            inode.invalidate();
        }
    }
}