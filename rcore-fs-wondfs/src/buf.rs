use crate::util::lru_cache;
use crate::driver::disk_manager;

pub struct BufCache {
    pub capacity: usize,
    pub cache: lru_cache::LRUCache<Buf>,
    pub disk_manager: disk_manager::DiskManager,
}

impl BufCache {
    pub fn new() -> BufCache {
        let capacity = 1024;
        BufCache {
            capacity: capacity as usize,
            cache: lru_cache::LRUCache::new(capacity as usize),
            disk_manager: disk_manager::DiskManager::new(true, None),
        }
    }
}

impl BufCache {
    pub fn read(&mut self, _: u8, address: u32) -> [u8; 4096] {
        let data = self.get_data(address);
        if data.is_some() {
            return data.unwrap();
        }
        let data = self.disk_manager.disk_read(address);
        self.put_data(address, data);
        self.get_data(address).unwrap()
    }

    pub fn write(&mut self, _: u8, address: u32, data: &[u8; 4096]) {
        self.put_data(address, *data);
        self.disk_manager.disk_write(address, data);
    }

    pub fn erase(&mut self, _: u8, block_no: u32) {
        let start_address = block_no * 128;
        let end_address = (block_no + 1) * 128;
        for address in start_address..end_address {
            self.remove_data(address);
        }
        self.disk_manager.disk_erase(block_no);
    }
}

impl BufCache {
    fn get_data(&mut self, address: u32) -> Option<[u8; 4096]> {
        let data = self.cache.get(address);
        if data.is_some() {
            return Some(data.as_deref().unwrap().data);
        }
        None
    }

    fn put_data(&mut self, address: u32, data: [u8; 4096]) {
        let buf = Buf::new(address, data);
        self.cache.put(address, buf);
    }

    fn remove_data(&mut self, address: u32) {
        self.cache.remove(address);
    }
}

#[derive(Clone, Copy)]
pub struct Buf {
    address: u32,
    data: [u8; 4096],
}

impl Buf {
    fn new(address: u32, data: [u8; 4096]) -> Buf {
        Buf {
            address,
            data,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn basics() {
        let mut cache = BufCache::new();
        let data = [1; 4096];        
        cache.put_data(100, data);
        assert_eq!(cache.get_data(100).unwrap(), [1; 4096]);
        cache.remove_data(100);
        assert_eq!(cache.get_data(100), None);
        cache.write(0, 100, &data);
        let data = cache.read(0, 100);
        assert_eq!(data, [1; 4096]);
        cache.erase(0, 0);
        let data = cache.read(0, 100);
        assert_eq!(data, [0; 4096]);
    }
}