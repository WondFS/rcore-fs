use std::cmp::Ordering;

pub static TOMBSTONE: &str = "TOMBSTONE";
pub static EOF: &str = "EOF";

#[derive(Eq, Clone)]
pub struct Entry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl Entry {
    pub fn new(key: Vec<u8>, value: Vec<u8>) -> Entry {
        Entry {
            key,
            value,
        }
    }

    pub fn get_size(&self) -> usize {
        self.key.len() + self.value.len()
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}