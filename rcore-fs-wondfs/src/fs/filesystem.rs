extern crate alloc;
use spin::RwLock;
use alloc::sync::{Arc, Weak};
use rcore_fs::{dev::Device, vfs};
use crate::kv::kv::KV;
use crate::inode::inode_manager::InodeManager;
use crate::inode::inode;
use crate::common::directory;

pub struct WondFS {
    pub device: Arc<dyn Device>,
    pub is_virtual: bool,
    pub kv: Arc<RwLock<KV>>,
    pub self_ptr: Weak<WondFS>,
    pub inode_manager: Option<Arc<RwLock<InodeManager>>>,
}

impl WondFS {
    pub fn open(device: Arc<dyn Device>) -> vfs::Result<Arc<Self>> {
        Ok(WondFS {
            device,
            is_virtual: false,
            kv: Arc::new(RwLock::new(KV::new())),
            self_ptr: Weak::default(),
            inode_manager: None,
        }.wrap())
    }

    pub fn wrap(self) -> Arc<Self> {
        let fs = Arc::new(self);
        let weak = Arc::downgrade(&fs);
        let ptr = Arc::into_raw(fs) as *mut Self;
        unsafe {
            (*ptr).self_ptr = weak;
        }
        unsafe { Arc::from_raw(ptr) }
    }

    pub fn init_inode_manager(&mut self) {
        self.inode_manager = Some(Arc::new(RwLock::new(InodeManager::new(self.self_ptr.upgrade().unwrap()))));
    }
}

impl WondFS {
    pub fn new_inode_file(&self) -> vfs::Result<Arc<inode::Inode>> {
        let inode = self.inode_manager.as_ref().unwrap().write().i_alloc().ok_or(vfs::FsError::NoDeviceSpace)?;
        Ok(inode)
    }

    pub fn new_inode_dir(&self, parent: u32) -> vfs::Result<Arc<inode::Inode>> {
        let inode = self.inode_manager.as_ref().unwrap().write().i_alloc().ok_or(vfs::FsError::NoDeviceSpace)?;
        let mut stat = inode.get_stat();
        stat.file_type = inode::InodeFileType::Directory;
        inode.modify_stat(stat);
        directory::dir_link(&inode, inode.stat.read().ino, ".".to_string());
        directory::dir_link(&inode, parent, "..".to_string());
        Ok(inode)
    }

    pub fn get_inode(&self, ino: u32) -> vfs::Result<Arc<inode::Inode>> {
        Ok(self.inode_manager.as_ref().unwrap().write().i_get(ino).unwrap())
    }
}