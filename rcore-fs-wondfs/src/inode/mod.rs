pub mod inode;
pub mod inode_impl;
pub mod inode_manager;

extern crate alloc;
use spin::RwLock;
use alloc::sync::Arc;

pub struct bb {
    pub aa: u32,
}

pub struct test {
    pub aa: Arc<RwLock<u32>>,
    pub bb: Arc<RwLock<bb>>,
}

impl test {
    pub fn new() -> test {
        test {
            aa: Arc::new(RwLock::new(10)),
            bb: Arc::new(RwLock::new(bb { aa: 100 })),
        }
    }

    pub fn www(&self) {
        let mut aa = self.aa.write();
        *aa = 0;
        self.bb.write().aa = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        let test = test::new();
        assert_eq!(*test.aa.read(), 10);
        assert_eq!(test.bb.read().aa, 100);
        test.www();
        assert_eq!(*test.aa.read(), 0);
        assert_eq!(test.bb.read().aa, 0);
    }
}