use std::time::SystemTime;
use super::gc_define::*;

pub struct BlockTable {
    pub size: u32,
    pub table: Vec<BlockInfo>,
}

impl BlockTable {
    pub fn new(size: u32) -> BlockTable {
        let mut table = vec![];
        let mut map = vec![];
        for _ in 0..128 {
            map.push(PageUsedStatus::Clean);
        }
        for i in 0..size {
            let block = BlockInfo {
                size: 128,
                block_no: i,
                reserved_size: 128,
                reserved_offset: 0,
                erase_count: 0,
                last_erase_time: 0,
                average_age: 0,
                used_map: map.clone(),
                dirty_num: 0,
                clean_num: 128,
                used_num: 0,
            };
            table.push(block);
        }
        BlockTable {
            size,
            table,
        }
    }
}

impl BlockTable {
    pub fn get_block_info(&self, block_no: u32) -> &BlockInfo {
        if block_no >= self.size {
            panic!("BlockTable: get block at too big block_no");
        }
        &self.table[block_no as usize]
    }

    pub fn set_block_info(&mut self, block_no: u32, info: BlockInfo) {
        if block_no >= self.size {
            panic!("BlockTable: set block at too big block_no");
        }
        self.table[block_no as usize] = info;
    }

    pub fn get_page(&self, address: u32) -> PageUsedStatus {
        let block_no = address / 128;
        if block_no >= self.size {
            panic!("BlockTable: get page at too big address");
        }
        self.table[block_no as usize].get_page(address)
    }

    pub fn set_page(&mut self, address: u32, status: PageUsedStatus) {
        let block_no = address / 128;
        if block_no >= self.size {
            panic!("BlockTable: set page at too big address");
        }
        self.table[block_no as usize].set_page(address, status);
    }

    pub fn erase_block(&mut self, block_no: u32) {
        self.table[block_no as usize].erase();
    }

    pub fn set_erase_count(&mut self, block_no: u32, erase_count: u32) {
        if block_no >= self.size {
            panic!("BlockTable: set at too big block_no");
        }
        self.table[block_no as usize].erase_count = erase_count;
    }

    pub fn set_last_erase_time(&mut self, block_no: u32, last_erase_time: u32) {
        if block_no >= self.size {
            panic!("BlockTable: set at too big block_no");
        }
        self.table[block_no as usize].last_erase_time = last_erase_time;
    }

    pub fn set_average_age(&mut self, block_no: u32, average_age: u32) {
        if block_no >= self.size {
            panic!("BlockTable: set at too big block_no");
        }
        self.table[block_no as usize].average_age = average_age;
    }
}

pub struct BlockInfo {
    pub size: u32,
    pub block_no: u32,
    pub reserved_size: u32,
    pub reserved_offset: u32,
    pub erase_count: u32,
    pub last_erase_time: u32,
    pub average_age: u32,
    dirty_num: u32,
    clean_num: u32,
    used_num: u32,
    used_map: Vec<PageUsedStatus>,
}

impl BlockInfo {
    pub fn set_page(&mut self, address: u32, status: PageUsedStatus) {
        let offset = address as i32 - self.block_no as i32 * 128;
        if offset < 0 || offset > 127 {
            panic!("BlockInfo: set page at not valid address");
        }
        let origin_status = self.used_map[offset as usize];
        match origin_status {
            PageUsedStatus::Clean => self.clean_num -= 1,
            PageUsedStatus::Dirty => self.dirty_num -= 1,
            PageUsedStatus::Busy(_) => self.used_num -= 1,
        }
        match status {
            PageUsedStatus::Clean => self.clean_num += 1,
            PageUsedStatus::Dirty => {
                self.dirty_num += 1;
            },
            PageUsedStatus::Busy(_) => {
                self.used_num += 1;
                self.reserved_offset += 1;
                self.reserved_size -= 1;
            },
        }
        self.used_map[offset as usize] = status;
    }

    pub fn get_page(&self, address: u32) -> PageUsedStatus {
        let offset = address as i32 - self.block_no as i32 * 128;
        if offset < 0 || offset > 127 {
            panic!("BlockInfo: get page at not valid address");
        }
        self.used_map[offset as usize]
    }

    pub fn erase(&mut self) {
        self.average_age = 0;
        self.reserved_offset = 0;
        self.reserved_size = 128;
        self.used_map.clear();
        self.dirty_num = 0;
        self.used_num = 0;
        self.clean_num = 128;
        for _ in 0..128 {
            self.used_map.push(PageUsedStatus::Clean);
        }
        self.erase_count += 1;
        self.last_erase_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).ok().unwrap().as_secs() as u32;
    }

    pub fn get_utilize_ratio(&self) -> f32 {
        (self.clean_num + self.used_num) as f32 / self.dirty_num as f32
    }
}