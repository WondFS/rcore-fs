use rcore_fs::vfs;
use super::filesystem::*;
use super::consts::*;

impl vfs::FileSystem for WondFS {
    fn sync(&self) -> vfs::Result<()> {
        Ok(())
    }

    fn root_inode(&self) -> std::sync::Arc<dyn vfs::INode> {
        self.inode_manager.as_ref().unwrap().write().i_get(ROOT_INO).unwrap()
    }

    fn info(&self) -> vfs::FsInfo {
        let sb = &self.kv.read().manager.super_stat;
        vfs::FsInfo {
            bsize: PAGESIZE,
            frsize: PAGESIZE,
            blocks: (sb.get_block_num() * sb.get_page_num_per_block()) as usize,
            bfree: (sb.get_block_num() * sb.get_page_num_per_block()) as usize,  // inaccurate
            bavail: (sb.get_block_num() * sb.get_page_num_per_block()) as usize, // inaccurate
            files: (sb.get_block_num() * sb.get_page_num_per_block()) as usize,  // inaccurate
            ffree: (sb.get_block_num() * sb.get_page_num_per_block()) as usize,  // inaccurate
            namemax: MAX_FNAME_LEN,
        }
    }
}