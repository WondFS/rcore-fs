pub mod kv;
pub mod gc;
pub mod lsm_tree;
pub mod kv_helper;
pub mod component;
pub mod kv_manager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_manager() {
        let mut kv = kv::KV::new();
        kv.mount();
        let mut metadata = kv::InodeMetadata {
            file_type: 0,
            ino: 0,
            size: 0,
            n_link: 1,
            last_accessed: 0,
            last_modified: 0,
            last_metadata_changed: 0,
        };
        let ino = kv.allocate_indoe(&mut metadata);
        let data = vec![111; 6000];
        kv.set_inode_data(ino, 0, data.len(), &data);
        kv.delete_inode(ino);
        let metadata = kv.get_inode_metadata(ino);
        assert!(metadata.is_none());
        let data = kv.get_inode_data(ino, 0, 0);
        assert!(data.is_none());
    }

    #[test]
    fn test_kv_meta_object() {
        let mut kv = kv::KV::new();
        kv.mount();
        for i in 0..10000 {
            let metadata = kv::InodeMetadata {
                file_type: 0,
                ino: i,
                size: 0,
                n_link: 1,
                last_accessed: 0,
                last_modified: 0,
                last_metadata_changed: 0,
            };
            kv.set_inode_metadata(i, &metadata);
        }
        for i in 0..10000 {
            let metadata = kv.get_inode_metadata(i).unwrap();
            assert_eq!(metadata.ino, i);
        }
    }

    #[test]
    fn test_kv_data_object() {
        let mut kv = kv::KV::new();
        kv.mount();
        let mut metadata = kv::InodeMetadata {
            file_type: 0,
            ino: 0,
            size: 0,
            n_link: 1,
            last_accessed: 0,
            last_modified: 0,
            last_metadata_changed: 0,
        };
        let ino = kv.allocate_indoe(&mut metadata);
        let mut off = 0;
        for _ in 0..10 {
            let data = vec![111; 6000];
            kv.set_inode_data(ino, off, data.len(), &data);
            off += data.len();
        }
        let mut off = 1000;
        for _ in 0..10 {
            let data = vec![222; 2000];
            kv.set_inode_data(ino, off, data.len(), &data);
            off += data.len();
        }
        let data = kv.get_inode_data(ino, 0, 1000).unwrap();
        assert_eq!(data, vec![111; 1000]);
        let data = kv.get_inode_data(ino, 1000, 2000 * 10).unwrap();
        assert_eq!(data, vec![222; 2000 * 10]);
        let data = kv.get_inode_data(ino, 2000 * 10 + 1000, 6000 * 10 - 2000 * 10 - 1000).unwrap();
        assert_eq!(data, vec![111; 6000 * 10 - 2000 * 10 - 1000]);
        let metadata = kv.get_inode_metadata(ino).unwrap();
        assert_eq!(metadata.size, 60000);
        let data = vec![233; 2000];
        let metadata = kv.get_inode_metadata(ino).unwrap();
        kv.set_inode_data(ino, 59000, data.len(), &data);
        assert_eq!(metadata.size, 60000);
        let data = kv.get_inode_data(ino, 59000, 2000).unwrap();
        assert_eq!(data, vec![233; 2000]);
    }

    #[test]
    fn test_kv_data_object_advanced() {
        let mut kv = kv::KV::new();
        kv.mount();
        let mut inos = vec![];
        for _ in 0..5 {
            let mut metadata = kv::InodeMetadata {
                file_type: 0,
                ino: 0,
                size: 0,
                n_link: 1,
                last_accessed: 0,
                last_modified: 0,
                last_metadata_changed: 0,
            };
            let ino = kv.allocate_indoe(&mut metadata);
            inos.push(ino);
            let mut off = 0;
            for _ in 0..4 {
                let data = vec![111; 5000];
                kv.set_inode_data(ino, off, data.len(), &data);
                off += data.len();
            }
            let data = vec![222; 2000];
            kv.set_inode_data(ino, 4000, data.len(), &data);
            kv.set_inode_data(ino, 19000, data.len(), &data);
        }
        for ino in inos {
            let data = kv.get_inode_data(ino, 0, 4000).unwrap();
            assert_eq!(data, vec![111; 4000]);
            let data = kv.get_inode_data(ino, 4000, 2000).unwrap();
            assert_eq!(data, vec![222; 2000]);
            let data = kv.get_inode_data(ino, 6000, 13000).unwrap();
            assert_eq!(data, vec![111; 13000]);
            let data = kv.get_inode_data(ino, 19000, 2000).unwrap();
            assert_eq!(data, vec![222; 2000]);
            let metadata = kv.get_inode_metadata(ino).unwrap();
            assert_eq!(metadata.size, 21000);
        }
    }

    #[test]
    fn test_kv_data_object_delete() {
        let mut kv = kv::KV::new();
        kv.mount();
        let mut metadata = kv::InodeMetadata {
            file_type: 0,
            ino: 0,
            size: 0,
            n_link: 1,
            last_accessed: 0,
            last_modified: 0,
            last_metadata_changed: 0,
        };
        let ino = kv.allocate_indoe(&mut metadata);
        let mut off = 0;
        for _ in 0..5 {
            let data = vec![111; 6000];
            kv.set_inode_data(ino, off, data.len(), &data);
            off += data.len();
        }
        kv.delete_inode_data(ino, 0, 30000);
        let data = kv.get_inode_data(ino, 0, 1000);
        assert_eq!(data, Some(vec![]));
        let metadata = kv.get_inode_metadata(ino).unwrap();
        assert_eq!(metadata.size, 0);
        let mut off = 0;
        for _ in 0..5 {
            let data = vec![111; 6000];
            kv.set_inode_data(ino, off, data.len(), &data);
            off += data.len();
        }
        kv.delete_inode_data(ino, 10000, 20000);
        let metadata = kv.get_inode_metadata(ino).unwrap();
        assert_eq!(metadata.size, 10000);
        let data = kv.get_inode_data(ino, 0, 0).unwrap();
        assert_eq!(data, vec![111; 10000]);
        kv.delete_inode_data(ino, 9000, 1000);
        let data = kv.get_inode_data(ino, 0, 0).unwrap();
        assert_eq!(data, vec![111; 9000]);
    }

    #[test]
    fn test_kv_extra_object() {
        let mut kv = kv::KV::new();
        kv.mount();
        for i in 0..127 as usize {
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
            kv.set_extra_value(String::from_utf8(key).unwrap(), &value);
        }
        for i in 0..127 as usize {
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
            assert_eq!(kv.get_extra_value(String::from_utf8(key).unwrap()).unwrap(), value);
        }
    }

    #[test]
    fn basics() {
        let mut a = vec![1, 2, 3];
        a.push(10);
        println!("{:?}", a);
    }
}