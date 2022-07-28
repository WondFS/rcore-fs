use std::collections::HashMap;
use crate::util::array::array;

const MAGIC_NUMBER: u32 = 0x5555dddd;

#[derive(Copy, Clone)]
pub struct BITSegement {
    pub used_map: u128,
    pub last_erase_time: u32,
    pub erase_count: u32,
    pub average_age: u32,
    pub reserved: [u8; 4],
}

pub struct BIT {
    pub table: HashMap<u32, BITSegement>,
    pub sync: bool,
    pub is_op: bool,
}

impl BIT {
    pub fn new() -> BIT {
        BIT {
            table: HashMap::new(),
            sync: false,
            is_op: false,
        }
    }

    pub fn init_bit_segment(&mut self, block_no: u32, segment: BITSegement) {
        if self.table.contains_key(&block_no) {
            panic!("BIT: init block has exist");
        }
        self.table.insert(block_no, segment);
    }

    pub fn get_bit_segment(&self, block_no: u32) -> BITSegement {
        if !self.table.contains_key(&block_no) {
            panic!("BIT: get bit segment not that block");
        }
        self.table.get(&block_no).unwrap().to_owned()
    }

    pub fn set_bit_segment(&mut self, block_no: u32, segment: BITSegement) {
        if !self.table.contains_key(&block_no) {
            panic!("BIT: set bit segment not that block");
        }
        *self.table.get_mut(&block_no).unwrap() = segment;
        self.sync = true;
    }

    pub fn get_page(&self, address: u32) -> bool {
        let block_no = address / 128;
        let offset = address % 128;
        if !self.table.contains_key(&block_no) {
            panic!("BIT: get page not that page");
        }
        let bitmap = self.table.get(&block_no).unwrap().used_map;
        ((bitmap >> (127 - offset)) & 1) == 1 
    }

    pub fn set_page(&mut self, address: u32, status: bool) {
        let block_no = address / 128;
        let offset = address % 128;
        if !self.table.contains_key(&block_no) {
            panic!("BIT: set page not that page");
        }
        let mut bitmap = self.table.get(&block_no).unwrap().used_map;
        let tag: u128;
        match status {
            true => tag = 1,
            false => tag = 0,
        }
        bitmap = bitmap | (tag << (127 - offset));
        self.table.get_mut(&block_no).unwrap().used_map = bitmap;
        self.sync = true;
    }

    pub fn get_last_erase_time(&self, block_no: u32) -> u32 {
        if !self.table.contains_key(&block_no) {
            panic!("BIT: get last erase time not that block");
        }
        self.table.get(&block_no).unwrap().last_erase_time
    }

    pub fn set_last_erase_time(&mut self, block_no: u32, time: u32) {
        if !self.table.contains_key(&block_no) {
            panic!("BIT: set last erase time not that block");
        }
        self.table.get_mut(&block_no).unwrap().last_erase_time = time;
        self.sync = true;
    }

    pub fn get_erase_count(&self, block_no: u32) -> u32 {
        if !self.table.contains_key(&block_no) {
            panic!("BIT: get erase count not that block");
        }
        self.table.get(&block_no).unwrap().erase_count
    }

    pub fn set_erase_count(&mut self, block_no: u32, count: u32) {
        if !self.table.contains_key(&block_no) {
            panic!("BIT: set erase count not that block");
        }
        self.table.get_mut(&block_no).unwrap().erase_count = count;
        self.sync = true;
    }

    pub fn get_average_age(&self, block_no: u32) -> u32 {
        if !self.table.contains_key(&block_no) {
            panic!("BIT: get average age not that block");
        }
        self.table.get(&block_no).unwrap().average_age
    }

    pub fn set_average_age(&mut self, block_no: u32, age: u32) {
        if !self.table.contains_key(&block_no) {
            panic!("BIT: set average age not that block");
        }
        self.table.get_mut(&block_no).unwrap().average_age = age;
        self.sync = true;
    }
    
    pub fn get_block(&self, block_no: u32) -> Option<[bool; 128]> {
        let mut res = [false; 128];
        let start_index = block_no * 128;
        let end_index = (block_no + 1) * 128;
        for (index, i) in (start_index..end_index).enumerate() {
            res[index] = self.get_page(i);
        }
        Some(res)
    }

    pub fn set_block(&mut self, block_no: u32, status: [bool; 128]) {
        let start_index = block_no * 128;
        let end_index = (block_no + 1) * 128;
        for (index, i) in (start_index..end_index).enumerate() {
            self.set_page(i, status[index]);
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

impl BIT {
    pub fn encode(&self) -> array::Array1<u8> {
        let mut data = array::Array1::<u8>::new(128 * 4096, 0);
        data.set(0, 0x55);
        data.set(1, 0x55);
        data.set(2, 0xdd);
        data.set(3, 0xdd);
        for (block_no, segment) in &self.table {
            let start_index = 32 + block_no * 32;
            let bit_map = segment.used_map;
            let last_erase_time = segment.last_erase_time;
            let erase_count = segment.erase_count;
            let average_age = segment.average_age;
            let byte_1 = (bit_map >> 120) as u8;
            let byte_2 = (bit_map >> 112) as u8;
            let byte_3 = (bit_map >> 104) as u8;
            let byte_4 = (bit_map >> 96) as u8;
            let byte_5 = (bit_map >> 88) as u8;
            let byte_6 = (bit_map >> 80) as u8;
            let byte_7 = (bit_map >> 72) as u8;
            let byte_8 = (bit_map >> 64) as u8;
            let byte_9 = (bit_map >> 56) as u8;
            let byte_10 = (bit_map >> 48) as u8;
            let byte_11 = (bit_map >> 40) as u8;
            let byte_12 = (bit_map >> 32) as u8;
            let byte_13 = (bit_map >> 24) as u8;
            let byte_14 = (bit_map >> 16) as u8;
            let byte_15 = (bit_map >> 8) as u8;
            let byte_16 = bit_map as u8;
            data.set(start_index, byte_1);
            data.set(start_index + 1, byte_2);
            data.set(start_index + 2, byte_3);
            data.set(start_index + 3, byte_4);
            data.set(start_index + 4, byte_5);
            data.set(start_index + 5, byte_6);
            data.set(start_index + 6, byte_7);
            data.set(start_index + 7, byte_8);
            data.set(start_index + 8, byte_9);
            data.set(start_index + 9, byte_10);
            data.set(start_index + 10, byte_11);
            data.set(start_index + 11, byte_12);
            data.set(start_index + 12, byte_13);
            data.set(start_index + 13, byte_14);
            data.set(start_index + 14, byte_15);
            data.set(start_index + 15, byte_16);
            let byte_1 = (last_erase_time >> 24) as u8;
            let byte_2 = (last_erase_time >> 16) as u8;
            let byte_3 = (last_erase_time >> 8) as u8;
            let byte_4 = last_erase_time as u8;
            data.set(start_index + 16, byte_1);
            data.set(start_index + 17, byte_2);
            data.set(start_index + 18, byte_3);
            data.set(start_index + 19, byte_4);
            let byte_1 = (erase_count >> 24) as u8;
            let byte_2 = (erase_count >> 16) as u8;
            let byte_3 = (erase_count >> 8) as u8;
            let byte_4 = erase_count as u8;
            data.set(start_index + 20, byte_1);
            data.set(start_index + 21, byte_2);
            data.set(start_index + 22, byte_3);
            data.set(start_index + 23, byte_4);
            let byte_1 = (average_age >> 24) as u8;
            let byte_2 = (average_age >> 16) as u8;
            let byte_3 = (average_age >> 8) as u8;
            let byte_4 = average_age as u8;
            data.set(start_index + 24, byte_1);
            data.set(start_index + 25, byte_2);
            data.set(start_index + 26, byte_3);
            data.set(start_index + 27, byte_4);
            for i in 0..4 {
                data.set(start_index + 28 + i as u32, segment.reserved[i]);
            }
        }
        data
    }
}

pub struct DataRegion<'a> {
    count: u32,
    index: u32,
    num: u32,
    data: &'a array::Array1<[u8; 4096]>,
}

impl DataRegion<'_> {
    pub fn new(data: &array::Array1::<[u8; 4096]>, num: u32) -> DataRegion {
        if data.len() != 128 {
            panic!("DataRegion: new not matched size");
        }
        DataRegion {
            count: 32,
            index: 0,
            data,
            num
        }
    }
}

impl Iterator for DataRegion<'_> {
    type Item = (u32, BITSegement);
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 128 * 4096 && (self.index < self.num || self.num == 0) {
            let byte_1 = (self.data.get(self.count / 4096)[(self.count % 4096) as usize] as u128) << 120;
            let byte_2 = (self.data.get((self.count + 1) / 4096)[((self.count + 1) % 4096) as usize] as u128) << 112;
            let byte_3 = (self.data.get((self.count + 2) / 4096)[((self.count + 2) % 4096) as usize] as u128) << 104;
            let byte_4 = (self.data.get((self.count + 3) / 4096)[((self.count + 3) % 4096) as usize] as u128) << 96;
            let byte_5 = (self.data.get((self.count + 4) / 4096)[((self.count + 4) % 4096) as usize] as u128) << 88;
            let byte_6 = (self.data.get((self.count + 5) / 4096)[((self.count + 5) % 4096) as usize] as u128) << 80;
            let byte_7 = (self.data.get((self.count + 6) / 4096)[((self.count + 6) % 4096) as usize] as u128) << 72;
            let byte_8 = (self.data.get((self.count + 7) / 4096)[((self.count + 7) % 4096) as usize] as u128) << 64;
            let byte_9 = (self.data.get((self.count + 8) / 4096)[((self.count + 8) % 4096) as usize] as u128) << 56;
            let byte_10 = (self.data.get((self.count + 9) / 4096)[((self.count + 9) % 4096) as usize] as u128) << 48;
            let byte_11 = (self.data.get((self.count + 10) / 4096)[((self.count + 10) % 4096) as usize] as u128) << 40;
            let byte_12 = (self.data.get((self.count + 11) / 4096)[((self.count + 11) % 4096) as usize] as u128) << 32;
            let byte_13 = (self.data.get((self.count + 12) / 4096)[((self.count + 12) % 4096) as usize] as u128) << 24;
            let byte_14 = (self.data.get((self.count + 13) / 4096)[((self.count + 13) % 4096) as usize] as u128) << 16;
            let byte_15 = (self.data.get((self.count + 14) / 4096)[((self.count + 14) % 4096) as usize] as u128) << 8;
            let byte_16 = self.data.get((self.count + 15) / 4096)[((self.count + 15) % 4096) as usize] as u128;
            let bitmap = byte_1 + byte_2 + byte_3 + byte_4 + byte_5 + byte_6 + byte_7 + byte_8 + byte_9 + byte_10 + byte_11 + byte_12 + byte_13 + byte_14 + byte_15 + byte_16;
            let byte_1 = (self.data.get((self.count + 16) / 4096)[((self.count + 16) % 4096) as usize] as u32) << 24;
            let byte_2 = (self.data.get((self.count + 17) / 4096)[((self.count + 17) % 4096) as usize] as u32) << 16;
            let byte_3 = (self.data.get((self.count + 18) / 4096)[((self.count + 18) % 4096) as usize] as u32) << 8;
            let byte_4 = self.data.get((self.count + 19) / 4096)[((self.count + 19) % 4096) as usize] as u32;
            let last_erase_time = byte_1 + byte_2 + byte_3 + byte_4;
            let byte_1 = (self.data.get((self.count + 20) / 4096)[((self.count + 20) % 4096) as usize] as u32) << 24;
            let byte_2 = (self.data.get((self.count + 21) / 4096)[((self.count + 21) % 4096) as usize] as u32) << 16;
            let byte_3 = (self.data.get((self.count + 22) / 4096)[((self.count + 22) % 4096) as usize] as u32) << 8;
            let byte_4 = self.data.get((self.count + 23) / 4096)[((self.count + 23) % 4096) as usize] as u32;
            let erase_count = byte_1 + byte_2 + byte_3 + byte_4;
            let byte_1 = (self.data.get((self.count + 24) / 4096)[((self.count + 24) % 4096) as usize] as u32) << 24;
            let byte_2 = (self.data.get((self.count + 25) / 4096)[((self.count + 25) % 4096) as usize] as u32) << 16;
            let byte_3 = (self.data.get((self.count + 26) / 4096)[((self.count + 26) % 4096) as usize] as u32) << 8;
            let byte_4 = self.data.get((self.count + 27) / 4096)[((self.count + 27) % 4096) as usize] as u32;
            let average_age = byte_1 + byte_2 + byte_3 + byte_4;
            let mut reserved = [0; 4];
            for i in 0..4 {
                reserved[i] = self.data.get((self.count + 28 + i as u32) / 4096)[((self.count + 28 + i as u32) % 4096) as usize]
            }
            self.count += 32;
            self.index += 1;
            Some((self.index - 1, BITSegement {
                used_map: bitmap,
                last_erase_time,
                erase_count,
                reserved,
                average_age,
            }))
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
        let mut bit = BIT::new();
        let mut data = array::Array1::<[u8; 4096]>::new(128, [0; 4096]);
        let mut temp = data.get(0);
        temp[0] = 0x55;
        temp[1] = 0x55;
        temp[2] = 0xdd;
        temp[3] = 0xdd;
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
        let iter = DataRegion::new(&data, 0);
        for (block_no, segment) in iter {
            bit.init_bit_segment(block_no, segment);
        }
        assert_eq!(kv_manager::KVManager::transfer(&bit.encode()), data);
        assert_eq!(bit.need_sync(), false);
        bit.set_page(200, true);
        assert_eq!(bit.get_page(200), true);
        assert_eq!(bit.need_sync(), true);
        let data = [true; 128];
        bit.set_block(10, data);
        assert_eq!(bit.get_block(10).unwrap(), data);
    }
}