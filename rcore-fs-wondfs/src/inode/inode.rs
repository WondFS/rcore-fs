extern crate alloc;
use spin::RwLock;
use alloc::sync::Arc;
use crate::fs::filesystem;
use crate::kv::kv::InodeMetadata;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InodeFileType {
    File,
    Directory,
}

impl From<InodeFileType> for u8 {
    fn from(kind: InodeFileType) -> Self {
        match kind {
            InodeFileType::File => 0,
            InodeFileType::Directory => 1,
        }
    }
}

#[derive(Copy, Clone)]
pub struct InodeStat {
    pub file_type: InodeFileType,
    pub ino: u32,
    pub size: u32,
    pub ref_cnt: u8,
    pub n_link: u8,
    pub last_accessed: u32,
    pub last_modified: u32,
    pub last_metadata_changed: u32,
}

impl InodeStat {
    pub fn new() -> InodeStat {
        InodeStat {
            file_type: InodeFileType::File,
            ino: 0,
            size: 0,
            ref_cnt: 0,
            n_link: 0,
            last_accessed: 0,
            last_modified: 0,
            last_metadata_changed: 0,
        }
    }
}

pub struct Inode {
    pub valid: RwLock<bool>,
    pub fs: Arc<filesystem::WondFS>,
    pub stat: RwLock<InodeStat>,
}

impl Inode {
    pub fn new(fs: Arc<filesystem::WondFS>) -> Inode {
        Inode {
            fs,
            valid: RwLock::new(false),
            stat: RwLock::new(InodeStat::new()),
        }
    }

    pub fn read_all(&self, buf: &mut Vec<u8>) -> usize {
        self.read(0, self.stat.read().size as usize, buf)
    }

    pub fn read(&self, offset: usize, len: usize, buf: &mut Vec<u8>) -> usize {
        assert!(*self.valid.read());
        buf.clear();
        let data = self.fs.kv.write().get_inode_data(self.stat.read().ino, offset, len);
        buf.copy_from_slice(&data.unwrap());
        buf.len()
    }

    pub fn write(&self, offset: usize, len: usize, buf: &Vec<u8>) {
        assert!(*self.valid.read());
        let size = self.fs.kv.write().set_inode_data(self.stat.read().ino, offset, len, buf);
        self.stat.write().size = size as u32;
    }

    pub fn truncate(&self, offset: usize, len: usize) {
        assert!(*self.valid.read());
        let size = self.fs.kv.write().delete_inode_data(self.stat.read().ino, offset, len);
        self.stat.write().size = size as u32;
    }

    pub fn delete(&self) {
        assert!(*self.valid.read());
        self.fs.kv.write().delete_inode(self.stat.read().ino);
        *self.valid.write() = false;
    }

    pub fn get_stat(&self) -> InodeStat {
        assert!(*self.valid.read());
        *self.stat.read()
    }

    pub fn modify_stat(&self, stat: InodeStat) {
        assert!(*self.valid.read());
        self.stat.write().file_type = stat.file_type;
        self.stat.write().size = stat.size;
        self.stat.write().ref_cnt = stat.ref_cnt;
        self.stat.write().n_link = stat.n_link;
        self.stat.write().last_accessed = stat.last_accessed;
        self.stat.write().last_modified = stat.last_modified;
        self.stat.write().last_metadata_changed = stat.last_metadata_changed;
        let metadata = InodeMetadata {
            file_type: stat.file_type.into(),
            ino: self.stat.read().ino,
            size: self.stat.read().size,
            n_link: self.stat.read().n_link,
            last_accessed: self.stat.read().last_accessed,
            last_modified: self.stat.read().last_modified,
            last_metadata_changed: self.stat.read().last_metadata_changed,
        };
        self.fs.kv.write().set_inode_metadata(self.stat.read().ino, &metadata);
    }

    pub fn invalidate(&self) {
        *self.valid.write() = false;
    }

    pub fn validate(&self) {
        *self.valid.write() = true;
    }

    pub fn nlinks_inc(&self) {
        assert!(*self.valid.read());
        self.stat.write().n_link += 1;
        let metadata = InodeMetadata {
            file_type: self.stat.read().file_type.into(),
            ino: self.stat.read().ino,
            size: self.stat.read().size,
            n_link: self.stat.read().n_link,
            last_accessed: self.stat.read().last_accessed,
            last_modified: self.stat.read().last_modified,
            last_metadata_changed: self.stat.read().last_metadata_changed,
        };
        self.fs.kv.write().set_inode_metadata(self.stat.read().ino, &metadata);
    }

    pub fn nlinks_dec(&self) {
        assert!(*self.valid.read());
        assert!(self.stat.read().n_link > 0);
        self.stat.write().n_link -= 1;
        if self.stat.read().n_link == 0 {
            self.delete();
        }
        let metadata = InodeMetadata {
            file_type: self.stat.read().file_type.into(),
            ino: self.stat.read().ino,
            size: self.stat.read().size,
            n_link: self.stat.read().n_link,
            last_accessed: self.stat.read().last_accessed,
            last_modified: self.stat.read().last_modified,
            last_metadata_changed: self.stat.read().last_metadata_changed,
        };
        self.fs.kv.write().set_inode_metadata(self.stat.read().ino, &metadata);
    }
}