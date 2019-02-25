use std::env;
use std::fs;
use std::io::{Read, Write, Result};
use std::path::Path;
use std::mem::uninitialized;
use std::sync::Arc;
use simple_filesystem::*;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let cmd = &args[1];
    let dir_path = Path::new(&args[2]);
    let img_path = Path::new(&args[3]);
    match cmd.as_str() {
        "zip" => zip(dir_path, img_path),
        "unzip" => unzip(dir_path, img_path),
        _ => {
            println!("USAGE: <zip|unzip> <PATH> <IMG>");
            panic!("Invalid command: {}", cmd);
        }
    }
}

fn zip(path: &Path, img_path: &Path) -> Result<()> {
    const MAX_SPACE: usize = 0x1000 * 0x1000 * 8; // 128MB (4K bitmap)
    let img = fs::OpenOptions::new().read(true).write(true).create(true).open(img_path)?;
    let sfs = SimpleFileSystem::create(Box::new(img), MAX_SPACE);
    let inode = sfs.root_inode();
    zip_dir(path, inode)?;
    sfs.sync().expect("Failed to sync");
    Ok(())
}

fn zip_dir(path: &Path, inode: Arc<INode>) -> Result<()> {
    let dir = fs::read_dir(path).expect("Failed to open dir");
    for entry in dir {
        let entry = entry?;
        let name_ = entry.file_name();
        let name = name_.to_str().unwrap();
        let type_ = entry.file_type()?;
        if type_.is_file() {
            let inode = inode.create(name, FileType::File).expect("Failed to create INode");
            let mut file = fs::File::open(entry.path())?;
            inode.resize(file.metadata().unwrap().len() as usize).expect("Failed to resize INode");
            let mut buf: [u8; 4096] = unsafe { uninitialized() };
            let mut offset = 0usize;
            let mut len = 4096;
            while len == 4096 {
                len = file.read(&mut buf)?;
                inode.write_at(offset, &buf).expect("Failed to write image");
                offset += len;
            }
        } else if type_.is_dir() {
            let inode = inode.create(name, FileType::Dir).expect("Failed to create INode");
            zip_dir(entry.path().as_path(), inode)?;
        }
    }
    Ok(())
}

fn unzip(path: &Path, img_path: &Path) -> Result<()> {
    let img = fs::File::open(img_path)?;
    let sfs = SimpleFileSystem::open(Box::new(img)).expect("Failed to open sfs");
    let inode = sfs.root_inode();
    fs::create_dir(&path)?;
    unzip_dir(path, inode)
}

fn unzip_dir(path: &Path, inode: Arc<INode>) -> Result<()> {
    let files = inode.list().expect("Failed to list files from INode");
    for name in files.iter().skip(2) {
        let inode = inode.lookup(name.as_str()).expect("Failed to lookup");
        let mut path = path.to_path_buf();
        path.push(name);
        let info = inode.info().expect("Failed to get file info");
        match info.type_ {
            FileType::File => {
                let mut file = fs::File::create(&path)?;
                let mut buf: [u8; 4096] = unsafe { uninitialized() };
                let mut offset = 0usize;
                let mut len = 4096;
                while len == 4096 {
                    len = inode.read_at(offset, buf.as_mut()).expect("Failed to read from INode");
                    file.write(&buf[..len])?;
                    offset += len;
                }
            }
            FileType::Dir => {
                fs::create_dir(&path)?;
                unzip_dir(path.as_path(), inode)?;
            }
        }
    }
    Ok(())
}