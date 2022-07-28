extern crate alloc;
use spin::RwLock;
use alloc::sync::Arc;
use crate::buf;
use crate::util::array::array;
use super::gc::gc_manager;
use super::gc::gc_define::*;
use super::component::bit;
use super::component::pit;
use super::component::super_block;
use super::lsm_tree::lsm_tree;
use super::kv_manager::*;

impl KVManager {
    pub fn new() -> KVManager {
        let buf = Arc::new(RwLock::new(buf::BufCache::new()));
        KVManager {
            lsm_tree: lsm_tree::LSMTree::new(Arc::clone(&buf)),
            buf,
            bit: bit::BIT::new(),
            pit: pit::PIT::new(),
            gc: gc_manager::GCManager::new(),
            super_stat: super_block::SuperStat::new(),
        }
    }

    pub fn mount(&mut self) {
        self.read_bit();
        self.read_pit();
    }
}

impl KVManager {
    pub fn find_write_pos(&mut self, size: usize) -> u32 {
        let mut res;
        loop {
            res = self.gc.find_write_pos(size);
            if res.is_some() {
                break;
            }
            self.forward_gc();
        }
        res.unwrap()
    }

    pub fn forward_gc(&mut self) {

    }

    pub fn set_page(&mut self, address: u32, status: PageUsedStatus) {
        self.gc.set_page(address, status);
    }

    pub fn set_block_info(&mut self, block_no: u32, segment: bit::BITSegement) {
        self.gc.set_block_info(block_no, (segment.last_erase_time, segment.erase_count, segment.average_age));
    }

    pub fn erase_block_in_block_table(&mut self, block_no: u32) {
        self.gc.erase_block(block_no);
    }
}

impl KVManager {
    pub fn read_bit(&mut self) {
        let mut data_1 = self.read_block(1, false);
        let data_2 = self.read_block(2, false);
        let mut flag = false;
        for i in 0..4 {
            if data_2.get(0)[i] & 0b1111_1111 != 0 {
                flag = true;
                break;
            }
        }
        if flag {
            self.erase_block(1, false);
            self.write_block(1, &data_2, false);
            self.erase_block(2, false);
            data_1 = data_2;
        }
        self.set_bit(&data_1);
    }

    pub fn set_bit(&mut self, data: &array::Array1::<[u8; 4096]>) {
        let iter = bit::DataRegion::new(&data, 18);
        for (block_no, segment) in iter {
            let bit_map = segment.used_map;
            self.bit.init_bit_segment(block_no, segment);
            self.set_block_info(block_no, segment);
            let start_index = block_no * 128;
            for i in 0..128 {
                let index = start_index + i;
                if (bit_map >> (127 - i)) & 1 == 1 {
                    self.set_page(index, PageUsedStatus::Dirty);
                } else {
                    self.set_page(index, PageUsedStatus::Clean);
                }
            }
        }
    }

    pub fn update_bit(&mut self, address: u32, status: bool) {
        self.bit.set_page(address, status);
        match status {
            true => self.set_page(address, PageUsedStatus::Dirty),
            false => self.set_page(address, PageUsedStatus::Clean),
        }
        self.sync_bit();
    }

    pub fn set_last_erase_time(&mut self, block_no: u32, time: u32) {
        self.bit.set_last_erase_time(block_no, time);
        self.sync_bit();
    }

    pub fn set_erase_count(&mut self, block_no: u32, count: u32) {
        self.bit.set_erase_count(block_no, count);
        self.sync_bit();
    }

    pub fn set_average_age(&mut self, block_no: u32, age: u32) {
        self.bit.set_average_age(block_no, age);
        self.sync_bit();
    }

    pub fn sync_bit(&mut self) {
        if self.bit.need_sync() {
            let data = self.bit.encode();
            let data = KVManager::transfer(&data);
            self.write_block(2, &data, false);
            self.erase_block(1, false);
            self.write_block(1, &data, false);
            self.erase_block(2, false);
            self.bit.sync();
        }
    }

    pub fn bit_begin_op(&mut self) {
        self.bit.begin_op();
    }

    pub fn bit_end_op(&mut self) {
        self.bit.end_op();
        self.sync_bit();
    }
}

impl KVManager {
    pub fn read_pit(&mut self) {
        let mut data_1 = self.read_block(3, false);
        let data_2 = self.read_block(4, false);
        let mut flag = false;
        for i in 0..4 {
            if data_2.get(0)[i] & 0b1111_1111 != 0 {
                flag = true;
                break;
            }
        }
        if flag {
            self.erase_block(3, false);
            self.write_block(3, &data_2, false);
            self.erase_block(4, false);
            data_1 = data_2;
        }
        self.set_pit(&data_1);
    }

    pub fn set_pit(&mut self, data: &array::Array1::<[u8; 4096]>) {
        let mut startegy = pit::PITStrategy::None;
        if data.get(0)[0] == 0x77 && data.get(0)[1] == 0x77 && data.get(0)[2] == 0xdd && data.get(0)[3] == 0xdd {
            startegy = pit::PITStrategy::Map;
        }
        if data.get(0)[119] == 0x77 && data.get(0)[120] == 0x77 && data.get(0)[121] == 0xee && data.get(0)[122] == 0xee {
            startegy = pit::PITStrategy::Serial;
        }
        let iter = pit::DataRegion::new(&data, startegy);
        for (index, ino) in iter {
            if ino != 0 {
                self.pit.init_page(index, ino);
                self.set_page(index, PageUsedStatus::Busy(ino));
            }
        }
        self.pit.set_page_num(self.super_stat.get_main_size() * self.super_stat.get_page_num_per_block());
    }

    pub fn update_pit(&mut self, address: u32, status: u32) {
        self.pit.set_page(address, status);
        self.set_page(address, PageUsedStatus::Busy(status));
        self.sync_pit();
    }

    pub fn dirty_pit(&mut self, address: u32) {
        self.pit.delete_page(address);
        self.set_page(address, PageUsedStatus::Dirty);
        self.sync_pit();
    }

    pub fn clean_pit(&mut self, address: u32) {
        self.pit.clean_page(address);
        self.set_page(address, PageUsedStatus::Clean);
        self.sync_pit();
    }
    
    pub fn sync_pit(&mut self) {
        if self.pit.need_sync() {
            let data = self.pit.encode();
            let data = KVManager::transfer(&data);
            self.write_block(4, &data, false);
            self.erase_block(3, false);
            self.write_block(3, &data, false);
            self.erase_block(4, false);
            self.pit.sync();
        }
    }

    pub fn pit_begin_op(&mut self) {
        self.pit.begin_op();
    }

    pub fn pit_end_op(&mut self) {
        self.pit.end_op();
        self.sync_pit();
    }
}

impl KVManager {
    pub fn read_page(&mut self, address: u32, is_main: bool) -> [u8; 4096] {
        if is_main {
            self.buf.write().read(0, address + 15 * 128)
        } else {
            self.buf.write().read(0, address)
        }
    }

    pub fn read_block(&mut self, block_no: u32, is_main: bool) -> array::Array1::<[u8; 4096]> {
        let address = block_no * 128;
        let mut data = array::Array1::<[u8; 4096]>::new(128, [0; 4096]);
        for index in 0..128 {
            let page = self.read_page(address + index, is_main);
            data.set(index, page);
        }
        data
    }

    pub fn write_page(&mut self, address: u32, data: &[u8; 4096], is_main: bool) {
        if is_main {
            self.buf.write().write(0, address + 15 * 128, &data);
        } else {
            self.buf.write().write(0, address, &data);
        }
    }

    pub fn write_block(&mut self, block_no: u32, data: &array::Array1::<[u8; 4096]>, is_main: bool) {
        if data.len() != 128 {
            panic!("CoreManager: write block not matched size");
        }
        let address = block_no * 128;
        for (index, data) in data.iter().enumerate() {
            self.write_page(address + index as u32, &data, is_main);
        }
    }

    pub fn erase_block(&mut self, block_no: u32, is_main: bool) {
        if is_main {
            self.buf.write().erase(0, block_no + 15);
        } else {
            self.buf.write().erase(0, block_no);
        }
    }
}

impl KVManager {
    pub fn transfer(data: &array::Array1::<u8>) -> array::Array1::<[u8; 4096]> {
        if data.len() != 128 * 4096 {
            panic!("CoreManager: transfer not available size");
        }
        let mut res = array::Array1::<[u8; 4096]>::new(128, [0; 4096]);
        for index in 0..128 {
            let start_index = index * 4096;
            let mut page = [0; 4096];
            for i in 0..4096 {
                page[i] = data.get((start_index + i) as u32);
            }
            res.set(index as u32, page);
        }
        res
    }
}