use crate::noita::types::{StdMap, StdString};
//reference stored at 0x01204b30 or 0x01206fac
#[repr(C)]
#[derive(Debug)]
pub struct TagManager {
    unk1: [isize; 3],
    pub tags: StdMap<StdString, usize>,
    //TODO unk
}
