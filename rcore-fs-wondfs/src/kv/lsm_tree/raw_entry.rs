pub static TOMBSTONE: &str = "TOMBSTONE";
pub static EOF: &str = "EOF";

#[derive(Clone)]
pub struct Entry {
    pub crc32: u32,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub key_size: u32,
    pub value_size: u32,
}

impl Entry {
    pub fn new(key: Vec<u8>, value: Vec<u8>) -> Entry {
        Entry {
            crc32: 0,
            key_size: key.len() as u32,
            value_size: value.len() as u32,
            key,
            value,
        }
    }

    pub fn encode_entry(&mut self) -> Vec<u8> {
        let mut data = vec![];
        let crc32 = 0;
        data.append(&mut Entry::encode_u32(crc32));
        data.append(&mut Entry::encode_u32(self.key_size));
        data.append(&mut Entry::encode_u32(self.value_size));
        data.append(&mut self.key);
        data.append(&mut self.value);
        data
    }

    pub fn encode_u32(data: u32) -> Vec<u8> {
        let mut ret = [0; 4];
        ret[0] = (data >> 24) as u8;
        ret[1] = (data >> 16) as u8;
        ret[2] = (data >> 8) as u8;
        ret[3] = data as u8;
        ret.to_vec()
    }
}