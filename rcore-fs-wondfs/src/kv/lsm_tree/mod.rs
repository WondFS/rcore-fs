pub mod lsm_tree;
pub mod raw_entry;
pub mod entry;
pub mod memtable;
pub mod file_iter;
pub mod block_iter;
pub mod sstable_manager;

#[cfg(test)]
mod test {
    use super::*;
    use crate::buf;
    extern crate alloc;
    use spin::RwLock;
    use alloc::sync::Arc;

    #[test]
    fn basics() {
        let buf = buf::BufCache::new();
        let mut kv = lsm_tree::LSMTree::new(Arc::new(RwLock::new(buf)));
        kv.put(&"a".as_bytes().to_vec(), &"b".as_bytes().to_vec());
        kv.put(&"ssa".as_bytes().to_vec(), &"ada".as_bytes().to_vec());
        assert_eq!(kv.get(&"a".as_bytes().to_vec()).unwrap(), "b".as_bytes().to_vec());
        assert_eq!(kv.get(&"ssa".as_bytes().to_vec()).unwrap(), "ada".as_bytes().to_vec());
        kv.put(&"a".as_bytes().to_vec(), &"bc".as_bytes().to_vec());
        assert_eq!(kv.get(&"a".as_bytes().to_vec()).unwrap(), "bc".as_bytes().to_vec());
        for i in 0..100 {
            kv.put(&i.to_string().as_bytes().to_vec(), &"test".as_bytes().to_vec());
        }
        for i in 0..100 {
            assert_eq!(kv.get(&i.to_string().as_bytes().to_vec()).unwrap(), "test".as_bytes().to_vec());
        }
    }
 } 