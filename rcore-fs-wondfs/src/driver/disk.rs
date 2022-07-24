extern crate alloc;
use alloc::sync::Arc;
use rcore_fs::{
    vfs,
    dev::Device,
};

trait DeviceExt: Device {
    fn read_page(&self, id: usize, offset: usize, buf: &mut [u8]) -> vfs::Result<()> {
        debug_assert!(offset + buf.len() <= 4096);
        match self.read_at(id * 4096 + offset, buf) {
            Ok(len)  if len == buf.len() => Ok(()),
            _ => panic!("Device: cannot read block {} offset {} from device", id, offset),
        }
    }

    fn write_page(&self, id: usize, offset: usize, buf: &[u8]) -> vfs::Result<()> {
        debug_assert!(offset + buf.len() <= 4096);
        match self.write_at(id * 4096 + offset, buf) {
            Ok(len) if len == buf.len() => Ok(()),
            _ => panic!("Device: cannot write block {} offset {} to device", id, offset),
        }
    }
}

impl DeviceExt for dyn Device {}

pub struct DiskDriver {
    device: Arc<dyn Device>,
}

impl DiskDriver {
    pub fn new(device: Arc<dyn Device>) -> DiskDriver {
        DiskDriver {
            device
        }
    }
}

impl DiskDriver {
    pub fn disk_read(&self, address: u32) -> [u8; 4096] {
        let mut buf = [0; 4096];
        self.device.read_at(address as usize, &mut buf).ok();
        buf
    }

    pub fn disk_write(&mut self, address: u32, data: &[u8; 4096]) {
        let mut o_data = [0; 4096];
        self.device.read_page(address as usize, 0, &mut o_data).ok();
        if o_data != [0; 4096] {
            panic!("Disk: write at not clean address");
        }
        self.device.write_page(address as usize, 0, data).ok();
    }

    pub fn disk_erase(&mut self, block_no: u32) {
        let start_index = block_no * 128;
        let end_index = (block_no + 1) * 128;
        for index in start_index..end_index {
            self.device.write_page(index as usize, 0, &[0; 4096]).ok();
        }
    }
}