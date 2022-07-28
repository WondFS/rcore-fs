pub mod gc_define;
pub mod gc_manager;
pub mod block_table;

#[cfg(test)]
mod test {
    use super::gc_define::*;
    use super::gc_manager::*;
    use super::block_table::*;

    #[test]
    fn test_block_table() {
        let mut table = BlockTable::new(32);
        table.set_page(0, PageUsedStatus::Busy(0));
        table.set_page(1, PageUsedStatus::Busy(0));
        table.set_page(2, PageUsedStatus::Busy(0));
        table.set_page(3, PageUsedStatus::Busy(0));
        assert_eq!(table.table[0].block_no, 0);
        assert_eq!(table.table[0].reserved_offset, 4);
        assert_eq!(table.table[0].reserved_size, 124);
        table.erase_block(0);
        assert_eq!(table.table[0].reserved_offset, 0);
        assert_eq!(table.table[0].reserved_size, 128);
    }

    #[test]
    fn test_gc_manager() {
        let mut manager = GCManager::new();
        assert_eq!(manager.find_write_pos(5), Some(0));
        manager.set_page(0, PageUsedStatus::Busy(0));
        manager.set_page(1, PageUsedStatus::Busy(0));
        manager.set_page(2, PageUsedStatus::Busy(0));
        manager.set_page(3, PageUsedStatus::Busy(0));
        manager.set_page(4, PageUsedStatus::Busy(0));
        assert_eq!(manager.get_page(0), PageUsedStatus::Busy(0));
        assert_eq!(manager.find_write_pos(128), Some(128));
        let event = manager.new_gc_event(GCStrategy::Forward);
        assert_eq!(event.events[0], GCEvent::Move(MoveGCEvent{ index: 0, ino: 0, size: 5, o_address: 0, d_address: 128 }));
        assert_eq!(event.events[1], GCEvent::Erase(EraseGCEvent{ index: 1, block_no: 0 }));
    }
}