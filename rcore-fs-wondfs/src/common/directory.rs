use crate::inode::inode;

pub fn dir_lookup(inode: &inode::Inode, name: String) -> Option<(u32, usize)> {
    if inode.stat.read().file_type != inode::InodeFileType::Directory {
        return None;
    }
    let mut buf = vec![];
    if inode.read_all(&mut buf) == 0 {
        return None;
    }
    let iter = DirectoryParser::new(&buf);
    for (i, entry) in iter.enumerate() {
        if entry.ino == 0 {
            continue;
        }
        if entry.file_name == name {
            return Some((entry.ino, i));
        }
    }
    None
}

pub fn dir_link(inode: &inode::Inode, ino: u32, name: String) {
    if dir_lookup(&inode, name.clone()).is_some() {
        return;
    }
    let mut buf = vec![];
    if inode.read_all(&mut buf) == 0 {
        return;
    }
    let iter = DirectoryParser::new(&buf);
    let mut index = 0;
    let per_size = iter.per_size;
    for entry in iter {
        if entry.ino == 0 {
            break;
        }
        index += 1;
    }
    let entry = DirectoryInodeEntry {
        file_name: name,
        ino,
    };
    let buf = DirectoryParser::encode(&entry).unwrap();
    inode.write(index * per_size, per_size, &buf)
}

pub fn dir_unlink(inode: &inode::Inode, ino: u32, name: String) {
    if !dir_lookup(&inode, name.clone()).is_some() {
        return;
    }
    let mut buf = vec![];
    if inode.read_all(&mut buf) == 0 {
        return;
    }
    let iter = DirectoryParser::new(&buf);
    let mut index = 0;
    let per_size = iter.per_size;
    let len = iter.len;
    for entry in iter {
        if entry.ino == ino && entry.file_name == name {
            break;
        }
        index += 1;
    }
    if index == len {
        return;
    }
    inode.truncate(index * per_size, per_size)
}

#[derive(PartialEq, Debug)]
pub struct DirectoryInodeEntry {
    pub file_name: String,
    pub ino: u32,
}

pub struct DirectoryParser {
    pub count: usize,
    pub data: Vec<u8>,
    pub len: usize,
    pub per_size: usize,
}

impl DirectoryParser {
    pub fn new(data: &Vec<u8>) -> DirectoryParser {
        if data.len() % 14 != 0 {
            panic!("DirectoryParser: new not matched size");
        }
        DirectoryParser {
            count: 0,
            data: data.clone(),
            len: data.len(),
            per_size: 14,
        }
    }
    
    pub fn decode(buf: &Vec<u8>) -> Option<DirectoryInodeEntry> {
        if buf.len() != 14 {
            panic!("DirectoryParser: decode not matched size");
        }
        let mut ino = 0;
        ino += (buf[0] as u32) << 24;
        ino += (buf[1] as u32) << 16;
        ino += (buf[2] as u32) << 8;
        ino += buf[3] as u32;
        let mut len = 0;
        for byte in buf[4..14].iter() {
            if *byte != 0 {
                len += 1;
            }
        }
        if len == 0 {
            panic!("Directory: decode not available name");
        }
        Some(DirectoryInodeEntry {
            ino,
            file_name: std::str::from_utf8(&buf[4..4+len as usize]).unwrap().to_string(),
        })
    }
    
    pub fn encode(entry: &DirectoryInodeEntry) -> Option<Vec<u8>> {
        let mut res = vec![];
        let ino = entry.ino;
        res.push((ino >> 24) as u8);
        res.push((ino >> 16) as u8);
        res.push((ino >> 8) as u8);
        res.push(ino as u8);
        let mut name = entry.file_name.clone().into_bytes();
        for _ in name.len()..10 {
            name.push(0);
        }
        for i in 0..10 {
            res.push(name[i]);
        }
        Some(res)
    }
}

impl Iterator for DirectoryParser {
    type Item = DirectoryInodeEntry;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.len {
            let entry = DirectoryParser::decode(&self.data[self.count..self.count+14].to_vec()).unwrap();
            self.count += 14;
            Some(entry)
        } else {
            None
        }
    }
}