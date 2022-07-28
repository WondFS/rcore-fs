pub mod bit;
pub mod pit;
pub mod super_block;

#[cfg(test)]
mod tests {
    use crate::kv::kv_manager;
    use crate::util::array::array;
    use super::bit;
    use super::pit;

    #[test]
    fn test_bit() {
        let mut bit = bit::BIT::new();
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
        let iter = bit::DataRegion::new(&data, 0);
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

    #[test]
    fn test_pit() {
        let mut pit = pit::PIT::new();
        let mut data = array::Array1::<[u8; 4096]>::new(128, [0; 4096]);
        let mut temp = data.get(0);
        temp[0] = 0x77;
        temp[1] = 0x77;
        temp[2] = 0xee;
        temp[3] = 0xee;
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
        let iter = pit::DataRegion::new(&data, pit::PITStrategy::Serial);
        for (index, ino) in iter {
            if ino != 0 {
                pit.init_page(index, ino);
            }
        }
        assert_eq!(kv_manager::KVManager::transfer(&pit.encode()), data);
        assert_eq!(pit.need_sync(), false);
        pit.set_page(200, 100);
        assert_eq!(pit.get_page(200), 100);
        assert_eq!(pit.need_sync(), true);
    }
}