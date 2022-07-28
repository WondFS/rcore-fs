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
    fn test_block_iter() {
        let buf = buf::BufCache::new();
        block_iter::BlockIter::new(0, Arc::new(RwLock::new(buf)));
    }

    #[test]
    fn test_file_iter() {
        let buf = buf::BufCache::new();
        file_iter::FileIter::new(0, 3, Arc::new(RwLock::new(buf)));
    }

    #[test]
    fn test_entry() {
        let entry = entry::Entry::new(vec![1, 2, 3], vec![4, 5, 6]);
        assert_eq!(entry.get_size(), 6);
    }

    #[test]
    fn test_raw_entry() {
        let mut entry = raw_entry::Entry::new(vec![1, 2, 3], vec![4, 5, 6]);
        let data = entry.encode_entry();
        assert_eq!(data.len(), 18);
    }

    #[test]
    fn test_memtable() {
        let mut memtable = memtable::Memtable::new(100);
        for i in 0..5 {
            memtable.put(&vec![1, 2, i], &vec![3, 4, i]);
        }
        assert_eq!(memtable.get_size(), 90);
        assert_eq!(memtable.get(&vec![1, 2, 0]).unwrap(), vec![3, 4, 0]);
        memtable.put(&vec![1, 2, 0], &vec![3, 4, 5, 6]);
        assert_eq!(memtable.get_size(), 91);
        assert_eq!(memtable.get(&vec![1, 2, 0]).unwrap(), vec![3, 4, 5, 6]);
        assert_eq!(memtable.can_put(10), false);
        let entries = memtable.flush();
        for (index, entry) in entries.iter().enumerate().skip(1) {
            assert_eq!(entry.key, vec![1, 2, index as u8]);
            assert_eq!(entry.value, vec![3, 4, index as u8]);
        }
    }

    #[test]
    fn test_sstable() {
        let buf = buf::BufCache::new();
        let mut manager = sstable_manager::SSTableManager::new(0, 10, Arc::new(RwLock::new(buf)));
        manager.build();
        let mut memtable = memtable::Memtable::new(4096 * 128);
        for i in 0..12345 as usize {
            let mut ret = [0; 4];
            ret[0] = (i >> 24) as u8;
            ret[1] = (i >> 16) as u8;
            ret[2] = (i >> 8) as u8;
            ret[3] = i as u8;
            let ret = ret.to_vec();
            let mut key = vec![1, 2, 3, 4];
            key.extend_from_slice(&ret);
            let mut value = vec![3, 4];
            value.extend_from_slice(&ret);
            memtable.put(&key, &value);
        }
        assert_eq!(memtable.get_size(), 320970);
        let entries = memtable.flush();
        manager.flush(&entries);
        assert_eq!(manager.cur_block_id, 1);
        for i in 0..12345 as usize {
            let mut ret = [0; 4];
            ret[0] = (i >> 24) as u8;
            ret[1] = (i >> 16) as u8;
            ret[2] = (i >> 8) as u8;
            ret[3] = i as u8;
            let ret = ret.to_vec();
            let mut key = vec![1, 2, 3, 4];
            key.extend_from_slice(&ret);
            let mut value = vec![3, 4];
            value.extend_from_slice(&ret);
            assert_eq!(manager.get(&key).unwrap(), value);
        }
        manager.clear();
        manager.build();
        for i in 0..12345 as usize {
            let mut ret = [0; 4];
            ret[0] = (i >> 24) as u8;
            ret[1] = (i >> 16) as u8;
            ret[2] = (i >> 8) as u8;
            ret[3] = i as u8;
            let ret = ret.to_vec();
            let mut key = vec![1, 2, 3, 4];
            key.extend_from_slice(&ret);
            let mut value = vec![3, 4];
            value.extend_from_slice(&ret);
            assert_eq!(manager.get(&key).unwrap(), value);
        }
        for i in 0..12345 as usize {
            let mut ret = [0; 4];
            ret[0] = (i >> 24) as u8;
            ret[1] = (i >> 16) as u8;
            ret[2] = (i >> 8) as u8;
            ret[3] = i as u8;
            let ret = ret.to_vec();
            let mut key = vec![100, 100, 100, 100];
            key.extend_from_slice(&ret);
            let mut value = vec![222, 222];
            value.extend_from_slice(&ret);
            memtable.put(&key, &value);
        }
        assert_eq!(memtable.get_size(), 320970);
        let entries = memtable.flush();
        manager.flush(&entries);
        assert_eq!(manager.cur_block_id, 2);
        assert_eq!(manager.files.len(), 2);
        for i in 0..12345 as usize {
            let mut ret = [0; 4];
            ret[0] = (i >> 24) as u8;
            ret[1] = (i >> 16) as u8;
            ret[2] = (i >> 8) as u8;
            ret[3] = i as u8;
            let ret = ret.to_vec();
            let mut key = vec![100, 100, 100, 100];
            key.extend_from_slice(&ret);
            let mut value = vec![222, 222];
            value.extend_from_slice(&ret);
            assert_eq!(manager.get(&key).unwrap(), value);
        }
    }

    #[test]
    fn test_lsm_tree() {
        let buf = buf::BufCache::new();
        let mut kv = lsm_tree::LSMTree::new(Arc::new(RwLock::new(buf)));
        kv.put(&"a".as_bytes().to_vec(), &"b".as_bytes().to_vec());
        kv.put(&"ssa".as_bytes().to_vec(), &"ada".as_bytes().to_vec());
        assert_eq!(kv.get(&"a".as_bytes().to_vec()).unwrap(), "b".as_bytes().to_vec());
        assert_eq!(kv.get(&"ssa".as_bytes().to_vec()).unwrap(), "ada".as_bytes().to_vec());
        kv.delete(&"a".as_bytes().to_vec());
        assert_eq!(kv.get(&"a".as_bytes().to_vec()), None);
        for i in 0..100 {
            let mut ret = [0; 4];
            ret[0] = (i >> 24) as u8;
            ret[1] = (i >> 16) as u8;
            ret[2] = (i >> 8) as u8;
            ret[3] = i as u8;
            let ret = ret.to_vec();
            let mut key = vec![1, 2, 3, 4];
            key.extend_from_slice(&ret);
            kv.put(&key, &"test".as_bytes().to_vec());
        }
        for i in 0..100 {
            let mut ret = [0; 4];
            ret[0] = (i >> 24) as u8;
            ret[1] = (i >> 16) as u8;
            ret[2] = (i >> 8) as u8;
            ret[3] = i as u8;
            let ret = ret.to_vec();
            let mut key = vec![1, 2, 3, 4];
            key.extend_from_slice(&ret);
            assert_eq!(kv.get(&key).unwrap(), "test".as_bytes().to_vec());
        }
        for i in 0..82345 as usize {
            let mut ret = [0; 4];
            ret[0] = (i >> 24) as u8;
            ret[1] = (i >> 16) as u8;
            ret[2] = (i >> 8) as u8;
            ret[3] = i as u8;
            let ret = ret.to_vec();
            let mut key = vec![1, 2, 3, 4];
            key.extend_from_slice(&ret);
            let mut value = vec![3, 4];
            value.extend_from_slice(&ret);
            kv.put(&key, &value);
        }
        for i in 0..82345 as usize {
            let mut ret = [0; 4];
            ret[0] = (i >> 24) as u8;
            ret[1] = (i >> 16) as u8;
            ret[2] = (i >> 8) as u8;
            ret[3] = i as u8;
            let ret = ret.to_vec();
            let mut key = vec![1, 2, 3, 4];
            key.extend_from_slice(&ret);
            let mut value = vec![3, 4];
            value.extend_from_slice(&ret);
            assert_eq!(kv.get(&key).unwrap(), value);
        }
    }
 } 