use std::collections::HashMap;
use crate::util::array::array;

const MAGIC_NUMBER_1: u32 = 0x7777dddd;
const MAGIC_NUMBER_2: u32 = 0x7777eeee;

#[derive(PartialEq)]
pub enum PITStrategy {
    Map,
    Serial,
    None,
}

pub struct PIT {
    pub page_num: u32,
    pub table: HashMap<u32, u32>,
    pub sync: bool,
    pub is_op: bool,
}

impl PIT {
    pub fn new() -> PIT {
        PIT {
            table: HashMap::new(),
            sync: false,
            is_op: false,
            page_num: 0,
        }
    }

    pub fn set_page_num(&mut self, page_num: u32) {
        self.page_num = page_num;
    }

    pub fn init_page(&mut self, address: u32, status: u32) {
        if self.table.contains_key(&address) {
            panic!("PIT: init page has exist");
        }
        self.table.insert(address, status);
    }

    pub fn get_page(&self, address: u32) -> u32 {
        if !self.table.contains_key(&address) {
            panic!("PIT: get not that page");
        }
        self.table.get(&address).unwrap().clone()
    }

    pub fn set_page(&mut self, address: u32, status: u32) {
        if !self.table.contains_key(&address) {
            self.table.insert(address, status);
            self.sync = true;
            return;
        }
        *self.table.get_mut(&address).unwrap() = status;
        self.sync = true;
    }

    pub fn delete_page(&mut self, address: u32) {
        if !self.table.contains_key(&address) {
            panic!("PIT: delete not that page");
        }
        self.table.remove(&address).unwrap();
        self.sync = true;
    }

    pub fn clean_page(&mut self, address: u32) {
        if self.table.contains_key(&address) {
            self.table.remove(&address).unwrap();
            self.sync = true;
        }
    }

    pub fn need_sync(&self) -> bool {
        if self.is_op {
            return false;
        }
        self.sync
    }

    pub fn sync(&mut self) {
        self.sync = false;
    }

    pub fn begin_op(&mut self) {
        self.is_op = true;
    }

    pub fn end_op(&mut self) {
        self.is_op = false;
    }

}

impl PIT {
    pub fn encode(&self) -> array::Array1::<u8> {
        let strategy = self.choose_strategy();
        if strategy == PITStrategy::Map {
            self.encode_map()
        } else {
            self.encode_serial()
        }
    }
}

impl PIT {
    fn choose_strategy(&self) -> PITStrategy {
        let num = self.table.len();
        let multiples =  num as f32 / self.page_num as f32;
        if multiples < 0.5 {
            PITStrategy::Map
        } else {
            PITStrategy::Serial
        }
    }

    fn encode_serial(&self) -> array::Array1::<u8> {
        let mut res = array::Array1::<u32>::new(128 * 4096 / 4 - 2, 0);
        for (key, value) in &self.table {
            res.set(*key, *value);
        }
        let mut data = array::Array1::<u8>::new(128 * 4096, 0);
        data.set(0, 0x77);
        data.set(1, 0x77);
        data.set(2, 0xee);
        data.set(3, 0xee);
        for (index, value) in res.iter().enumerate() {
            let byte_1 = (value >> 24) as u8;
            let byte_2 = (value >> 16) as u8;
            let byte_3 = (value >> 8) as u8;
            let byte_4 = value as u8;
            let start_index = 8 + index * 4;
            data.set(start_index as u32, byte_1);
            data.set((start_index + 1) as u32, byte_2);
            data.set((start_index + 2) as u32, byte_3);
            data.set((start_index + 3) as u32, byte_4);
        }
        data
    }

    fn encode_map(&self) -> array::Array1::<u8> {
        let mut data = array::Array1::<u8>::new(128 * 4096, 0);
        data.set(0, 0x77);
        data.set(1, 0x77);
        data.set(2, 0xdd);
        data.set(3, 0xdd);
        let mut index = 0;
        for (key, value) in &self.table {
            let start_index = 8 + index * 8;
            let byte_1 = (*key >> 24) as u8;
            let byte_2 = (*key >> 16) as u8;
            let byte_3 = (*key >> 8) as u8;
            let byte_4 = *key as u8;
            data.set(start_index, byte_1);
            data.set(start_index + 1, byte_2);
            data.set(start_index + 2, byte_3);
            data.set(start_index + 3, byte_4);
            let byte_1 = (*value >> 24) as u8;
            let byte_2 = (*value >> 16) as u8;
            let byte_3 = (*value >> 8) as u8;
            let byte_4 = *value as u8;
            data.set(start_index + 4, byte_1);
            data.set(start_index + 5, byte_2);
            data.set(start_index + 6, byte_3);
            data.set(start_index + 7, byte_4);
            index += 1;
        }
        data
    }
}

pub struct DataRegion<'a> {
    count: u32,
    index: u32,
    strategy: PITStrategy,
    data: &'a array::Array1<[u8; 4096]>,
}

impl DataRegion<'_> {
    pub fn new(data: &array::Array1::<[u8; 4096]>, strategy: PITStrategy) -> DataRegion {
        if data.len() != 128 {
            panic!("DataRegion: new not matched size");
        }
        DataRegion {
            count: 8,
            index: 0,
            data,
            strategy,
        }
    }
}

impl Iterator for DataRegion<'_> {
    type Item = (u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 128 * 4096 {
            match self.strategy {
                PITStrategy::Map => {
                    let byte_1 = (self.data.get(self.count / 4096)[(self.count % 4096) as usize] as u32) << 24;
                    let byte_2 = (self.data.get((self.count + 1) / 4096)[((self.count + 1) % 4096) as usize] as u32) << 16;
                    let byte_3 = (self.data.get((self.count + 2) / 4096)[((self.count + 2) % 4096) as usize] as u32) << 8;
                    let byte_4 = self.data.get((self.count + 3) / 4096)[((self.count + 3) % 4096) as usize] as u32;
                    let index = byte_1 + byte_2 + byte_3 + byte_4;
                    let byte_1 = (self.data.get((self.count + 4) / 4096)[((self.count + 4) % 4096) as usize] as u32) << 24;
                    let byte_2 = (self.data.get((self.count + 5) / 4096)[((self.count + 5) % 4096) as usize] as u32) << 16;
                    let byte_3 = (self.data.get((self.count + 6) / 4096)[((self.count + 6) % 4096) as usize] as u32) << 8;
                    let byte_4 = self.data.get((self.count + 7) / 4096)[((self.count + 7) % 4096) as usize] as u32;
                    let ino = byte_1 + byte_2 + byte_3 + byte_4;
                    self.count += 8;
                    if index == 0 && ino == 0 {
                        None
                    } else {
                        Some((index, ino))
                    }
                },
                PITStrategy::Serial => {
                    let byte_1 = (self.data.get(self.count / 4096)[(self.count % 4096) as usize] as u32) << 24;
                    let byte_2 = (self.data.get((self.count + 1) / 4096)[((self.count + 1) % 4096) as usize] as u32) << 16;
                    let byte_3 = (self.data.get((self.count + 2) / 4096)[((self.count + 2) % 4096) as usize] as u32) << 8;
                    let byte_4 = self.data.get((self.count + 3) / 4096)[((self.count + 3) % 4096) as usize] as u32;
                    let ino = byte_1 + byte_2 + byte_3 + byte_4;
                    self.count += 4;
                    self.index += 1;
                    Some((self.index - 1, ino))
                },
                PITStrategy::None => {
                    None
                },
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::kv::kv_manager;
    use crate::util::array::array;
    use super::*;

    #[test]
    fn basics() {
        let mut pit = PIT::new();
        let mut data = array::Array1::<[u8; 4096]>::new(128, [0; 4096]);
        let mut temp = data.get(0);
        temp[0] = 0x77;
        temp[1] = 0x77;
        temp[2] = 0xee;
        temp[3] = 0xee;
        data.set(0, temp);
        let mut temp = data.get(100);
        temp[312] = 234;
        data.set(100, temp);
        let mut temp = data.get(11);
        temp[232] = 67;
        data.set(11, temp);
        let mut temp = data.get(121);
        temp[2332] = 123;
        data.set(121, temp);
        let iter = DataRegion::new(&data, PITStrategy::Serial);
        for (index, ino) in iter {
            if ino != 0 {
                pit.init_page(index, ino);
            }
        }
        assert_eq!(kv_manager::KVManager::transfer(&pit.encode()), data);
        assert_eq!(pit.need_sync(), false);
        pit.set_page(200, 100);
        assert_eq!(pit.get_page(200), 100);
        assert_eq!(pit.need_sync(), true);
    }
}