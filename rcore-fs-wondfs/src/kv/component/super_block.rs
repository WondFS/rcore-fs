use crate::util::array::array;

const MAGICNUMBER: u32 = 0x3bf7444d;

pub struct SuperStat {
    magic_code: u32,
    block_num: u32,
    super_block_num: u32,
    bit_block_num: u32,
    pit_block_num: u32,
    journal_block_num: u32,
    kv_block_num: u32,
    main_area_block_num: u32,
    reserved_block_num: u32,
    page_size: u32,
    page_num_per_block: u32,
}

impl SuperStat {
    pub fn new() -> SuperStat {
        SuperStat {
            magic_code: 0,
            block_num: 32,
            super_block_num: 1,
            bit_block_num: 2,
            pit_block_num: 2,
            journal_block_num: 1,
            kv_block_num: 4,
            main_area_block_num: 18,
            reserved_block_num: 4,
            page_size: 4096,
            page_num_per_block: 128,
        }
    }

    pub fn build(&mut self, data: &array::Array1::<[u8; 4096]>) {
        let page = data.get(0);
        let byte1 = (page[0] as u32) << 24;
        let byte2 = (page[1] as u32) << 16;
        let byte3 = (page[2] as u32) << 8;
        let byte4 = page[3] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.magic_code = value;
        let byte1 = (page[4] as u32) << 24;
        let byte2 = (page[5] as u32) << 16;
        let byte3 = (page[6] as u32) << 8;
        let byte4 = page[7] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.block_num = value;
        let byte1 = (page[8] as u32) << 24;
        let byte2 = (page[9] as u32) << 16;
        let byte3 = (page[10] as u32) << 8;
        let byte4 = page[11] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.super_block_num = value;
        let byte1 = (page[12] as u32) << 24;
        let byte2 = (page[13] as u32) << 16;
        let byte3 = (page[14] as u32) << 8;
        let byte4 = page[15] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.bit_block_num = value;
        let byte1 = (page[16] as u32) << 24;
        let byte2 = (page[17] as u32) << 16;
        let byte3 = (page[18] as u32) << 8;
        let byte4 = page[19] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.pit_block_num = value;
        let byte1 = (page[20] as u32) << 24;
        let byte2 = (page[21] as u32) << 16;
        let byte3 = (page[22] as u32) << 8;
        let byte4 = page[23] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.journal_block_num = value;
        let byte1 = (page[24] as u32) << 24;
        let byte2 = (page[25] as u32) << 16;
        let byte3 = (page[26] as u32) << 8;
        let byte4 = page[27] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.kv_block_num = value;
        let byte1 = (page[28] as u32) << 24;
        let byte2 = (page[29] as u32) << 16;
        let byte3 = (page[30] as u32) << 8;
        let byte4 = page[31] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.main_area_block_num = value;
        let byte1 = (page[32] as u32) << 24;
        let byte2 = (page[33] as u32) << 16;
        let byte3 = (page[34] as u32) << 8;
        let byte4 = page[35] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.reserved_block_num = value;
        let byte1 = (page[36] as u32) << 24;
        let byte2 = (page[37] as u32) << 16;
        let byte3 = (page[38] as u32) << 8;
        let byte4 = page[39] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.page_size = value;
        let byte1 = (page[40] as u32) << 24;
        let byte2 = (page[41] as u32) << 16;
        let byte3 = (page[42] as u32) << 8;
        let byte4 = page[43] as u32;
        let value = byte1 + byte2 + byte3 + byte4;
        self.page_num_per_block = value;
        if self.magic_code != MAGICNUMBER {
            panic!("SuperStat: build error");
        }
    }

    pub fn get_bit_offset(&self) -> u32 {
        self.super_block_num
    }

    pub fn get_bit_size(&self) -> u32 {
        self.bit_block_num
    }

    pub fn get_pit_offset(&self) -> u32 {
        self.super_block_num + self.bit_block_num
    }

    pub fn get_pit_size(&self) -> u32 {
        self.pit_block_num
    }

    pub fn get_journal_offset(&self) -> u32 {
        self.super_block_num + self.bit_block_num + self.pit_block_num
    }

    pub fn get_journal_size(&self) -> u32 {
        self.journal_block_num
    }

    pub fn get_kv_offset(&self) -> u32 {
        self.super_block_num + self.bit_block_num + self.pit_block_num + self.journal_block_num
    }

    pub fn get_kv_size(&self) -> u32 {
        self.kv_block_num
    }

    pub fn get_main_offset(&self) -> u32 {
        self.super_block_num + self.bit_block_num + self.pit_block_num + self.journal_block_num + self.kv_block_num
    }

    pub fn get_main_size(&self) -> u32 {
        self.main_area_block_num
    }

    pub fn get_reserved_offset(&self) -> u32 {
        self.super_block_num + self.bit_block_num + self.pit_block_num + self.journal_block_num + self.kv_block_num + self.main_area_block_num
    }

    pub fn get_reserved_size(&self) -> u32 {
        self.reserved_block_num
    }

    pub fn get_page_size(&self) -> u32 {
        self.page_size
    }

    pub fn get_page_num_per_block(&self) -> u32 {
        self.page_num_per_block
    }

    pub fn get_block_num(&self) -> u32 {
        self.block_num
    }
}