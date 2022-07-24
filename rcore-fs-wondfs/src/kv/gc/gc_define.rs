pub const HOTAGEKEY: u32 = 60 * 60 * 24;
pub const COLDAGEKEY: u32 = 60 * 60 * 24 * 14;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PageUsedStatus {
    Clean,
    Dirty,
    Busy(u32),
}

pub enum GCStrategy {
    Forward,
    BackgroundSimple,
    BackgroundCold,
}

pub struct GCEventGroup {
    pub events: Vec<GCEvent>,
}

impl GCEventGroup {
    pub fn new() -> GCEventGroup {
        GCEventGroup {
            events: vec![],
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GCEvent {
    Erase(EraseGCEvent),
    Move(MoveGCEvent),
    None,
}

impl GCEvent {
    pub fn get_index(&self) -> u32 {
        let mut index = 0;
        match self {
            GCEvent::Erase(event) => index = event.index,
            GCEvent::Move(event) => index = event.index,
            GCEvent::None => ()
        }
        index
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct EraseGCEvent {
    pub index: u32,
    pub block_no: u32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MoveGCEvent {
    pub index: u32,
    pub ino: u32,
    pub size: u32,
    pub o_address: u32,
    pub d_address: u32,
}