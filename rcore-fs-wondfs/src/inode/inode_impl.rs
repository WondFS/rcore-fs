extern crate alloc;
use alloc::sync::Arc;
use core::any::Any;
use rcore_fs::vfs;
use super::inode::*;
use crate::common::directory::*;

impl From<InodeFileType> for vfs::FileType {
    fn from(t: InodeFileType) -> Self {
        match t {
            InodeFileType::File => vfs::FileType::File,
            InodeFileType::Directory => vfs::FileType::Dir,
        }
    }
}

impl vfs::INode for Inode {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> vfs::Result<usize> {
        match self.stat.read().file_type {
            InodeFileType::File => {
                let mut data = vec![0; buf.len()];
                let num = self.read(offset, buf.len(), &mut data);
                buf.copy_from_slice(&data);
                Ok(num)
            },
            _ => Err(vfs::FsError::NotFile),
        }
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> vfs::Result<usize> {
        match self.stat.read().file_type {
            InodeFileType::File => {
                let mut data = vec![];
                data.copy_from_slice(buf);
                self.write(offset, buf.len(), &data);
                Ok(buf.len())
            }
            _ => Err(vfs::FsError::NotFile),
        }
    }

    fn poll(&self) -> vfs::Result<vfs::PollStatus> {
        Ok(vfs::PollStatus {
            read: true,
            write: true,
            error: false,
        })
    }

    fn metadata(&self) -> vfs::Result<vfs::Metadata> {
        Ok(vfs::Metadata {
            inode: self.stat.read().ino as usize,
            size: self.stat.read().size as usize,
            blk_size: 4096,
            atime: vfs::Timespec { sec: self.stat.read().last_accessed as i64, nsec: 0},
            mtime: vfs::Timespec { sec: self.stat.read().last_modified as i64, nsec: 0},
            ctime: vfs::Timespec { sec: self.stat.read().last_metadata_changed as i64, nsec: 0},
            type_: vfs::FileType::from(self.stat.read().file_type),
            mode: 0o777,
            nlinks: self.stat.read().n_link as usize,
            dev: 0,    // inaccurate
            blocks: 1, // inaccurate
            uid: 0,    // inaccurate
            gid: 0,    // inaccurate
            rdev:0,    // inaccurate
        })
    }

    fn set_metadata(&self, metadata: &vfs::Metadata) -> vfs::Result<()> {
        let mut stat = self.get_stat();
        stat.last_accessed = metadata.atime.sec as u32;
        stat.last_modified = metadata.mtime.sec as u32;
        stat.last_metadata_changed = metadata.ctime.sec as u32;
        self.modify_stat(stat);
        Ok(())
    }

    fn sync_all(&self) -> vfs::Result<()> {
        Ok(())
    }

    fn sync_data(&self) -> vfs::Result<()> {
        self.sync_all()
    }

    fn resize(&self, len: usize) -> vfs::Result<()> {
        if self.stat.read().file_type != InodeFileType::File {
            return Err(vfs::FsError::NotFile);
        }
        let size = self.stat.read().size;
        self.truncate(size as usize - len, len);
        Ok(())
    }

    fn create2(
        &self,
        name: &str,
        type_: vfs::FileType,
        _mode: u32,
        _: usize,
    ) -> vfs::Result<Arc<dyn vfs::INode>> {
        let info = self.metadata()?;
        if info.type_ != vfs::FileType::Dir {
            return Err(vfs::FsError::NotDir);
        }
        if info.nlinks == 0 {
            return Err(vfs::FsError::DirRemoved);
        }
        if dir_lookup(&self, name.to_string()).is_some() {
            return Err(vfs::FsError::EntryExist);
        }
        let inode = match type_ {
            vfs::FileType::File => self.fs.new_inode_file()?,
            vfs::FileType::Dir => self.fs.new_inode_dir(info.inode as u32)?,
            _ => return Err(vfs::FsError::InvalidParam),
        };
        dir_link(&self, inode.stat.read().ino, name.to_string());
        self.nlinks_inc();
        if type_ == vfs::FileType::Dir {
            inode.nlinks_inc();
            self.nlinks_inc();
        }
        Ok(inode)
    }

    fn link(&self, name: &str, other: &Arc<dyn vfs::INode>) -> vfs::Result<()> {
        let info = self.metadata()?;
        if info.type_ != vfs::FileType::Dir {
            return Err(vfs::FsError::NotDir);
        }
        if info.nlinks == 0 {
            return Err(vfs::FsError::DirRemoved);
        }
        if dir_lookup(&self, name.to_string()).is_some() {
            return Err(vfs::FsError::EntryExist);
        }
        let child = other.downcast_ref::<Inode>().ok_or(vfs::FsError::NotSameFs)?;
        if !Arc::ptr_eq(&self.fs, &child.fs) {
            return Err(vfs::FsError::NotSameFs);
        }
        if child.metadata()?.type_ == vfs::FileType::Dir {
            return Err(vfs::FsError::IsDir);
        }
        dir_link(&self, child.stat.read().ino, name.to_string());
        child.nlinks_inc();
        Ok(())
    }

    fn unlink(&self, name: &str) -> vfs::Result<()> {
        let info = self.metadata()?;
        if info.type_ != vfs::FileType::Dir {
            return Err(vfs::FsError::NotDir);
        }
        if info.nlinks == 0 {
            return Err(vfs::FsError::DirRemoved);
        }
        if name == "." {
            return Err(vfs::FsError::IsDir);
        }
        if name == ".." {
            return Err(vfs::FsError::IsDir);
        }
        let entry = dir_lookup(&self, name.to_string()).ok_or(vfs::FsError::EntryNotFound)?;
        let inode = self.fs.get_inode(entry.0).ok().unwrap();
        if inode.stat.read().file_type == InodeFileType::Directory {
            if inode.stat.read().size > 28 {
                return Err(vfs::FsError::DirNotEmpty);
            }
        }
        inode.nlinks_dec();
        if inode.stat.read().file_type == InodeFileType::Directory {
            inode.nlinks_dec();
            self.nlinks_dec();
        }
        dir_unlink(&self, inode.stat.read().ino, name.to_string());
        Ok(())
    }

    fn move_(&self, old_name: &str, target: &Arc<dyn vfs::INode>, new_name: &str) -> vfs::Result<()> {
        let info = self.metadata()?;
        if info.type_ != vfs::FileType::Dir {
            return Err(vfs::FsError::NotDir);
        }
        if info.nlinks == 0 {
            return Err(vfs::FsError::DirRemoved);
        }
        if old_name == "." {
            return Err(vfs::FsError::IsDir);
        }
        if old_name == ".." {
            return Err(vfs::FsError::IsDir);
        }
        let dest = target.downcast_ref::<Inode>().ok_or(vfs::FsError::NotSameFs)?;
        let dest_info = dest.metadata()?;
        if !Arc::ptr_eq(&self.fs, &dest.fs) {
            return Err(vfs::FsError::NotSameFs);
        }
        if dest_info.type_ != vfs::FileType::Dir {
            return Err(vfs::FsError::NotDir);
        }
        if dest_info.nlinks == 0 {
            return Err(vfs::FsError::DirRemoved);
        }
        if let Some((ino, _)) = dir_lookup(dest, new_name.to_string()) {
            dir_unlink(dest, ino, new_name.to_string());
            let inode = self.fs.get_inode(ino).ok().unwrap();
            inode.nlinks_dec();
        }
        let (ino, _) = dir_lookup(&self, old_name.to_string()).ok_or(vfs::FsError::EntryNotFound)?;
        if info.inode == dest_info.inode {
            dir_unlink(&self, ino, old_name.to_string());
            dir_link(&self, ino, new_name.to_string());
        } else {
            dir_unlink(&self, ino, old_name.to_string());
            dir_link(dest, ino, new_name.to_string());
            let inode = self.fs.get_inode(ino).ok().unwrap();
            if inode.stat.read().file_type == InodeFileType::Directory {
                self.nlinks_dec();
                dest.nlinks_inc();
            }
        }
        Ok(())
    }

    fn find(&self, name: &str) -> vfs::Result<Arc<dyn vfs::INode>> {
        let info = self.metadata()?;
        if info.type_ != vfs::FileType::Dir {
            return Err(vfs::FsError::NotDir);
        }
        let (ino, _) = dir_lookup(&self, name.to_string()).ok_or(vfs::FsError::EntryNotFound)?;
        Ok(self.fs.get_inode(ino).ok().unwrap())
    }

    fn get_entry(&self, id: usize) -> vfs::Result<String> {
        if self.stat.read().file_type != InodeFileType::Directory {
            return Err(vfs::FsError::NotDir);
        }
        if id >= self.stat.read().size as usize / 14 {
            return Err(vfs::FsError::EntryNotFound);
        };
        let mut data = vec![];
        self.read_all(&mut data);
        let iter = DirectoryParser::new(&data);
        for entry in iter.skip(id) {
            return Ok(entry.file_name);
        }
        Err(vfs::FsError::EntryNotFound)
    }

    fn get_entry_with_metadata(&self, id: usize) -> vfs::Result<(vfs::Metadata, String)> {
        if self.stat.read().file_type != InodeFileType::Directory {
            return Err(vfs::FsError::NotDir);
        }
        if id >= self.stat.read().size as usize / 14 {
            return Err(vfs::FsError::EntryNotFound);
        };
        let mut data = vec![];
        self.read_all(&mut data);
        let iter = DirectoryParser::new(&data);
        for entry in iter.skip(id) {
            return Ok((
                self.fs.get_inode(entry.ino).ok().unwrap().metadata()?,
                entry.file_name,
            ));
        }
        Err(vfs::FsError::EntryNotFound)
    }

    fn io_control(&self, _cmd: u32, _data: usize) -> vfs::Result<usize> {
        Err(vfs::FsError::IOCTLError)
    }

    fn mmap(&self, _area: vfs::MMapArea) -> vfs::Result<()> {
        Err(vfs::FsError::NotSupported)
    }

    fn fs(&self) -> Arc<dyn vfs::FileSystem> {
        self.fs.clone()
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}