use crate::noita::types::{StdMap, StdString, StdVec};
//reference stored at 0x01204b30 or 0x01206fac
#[repr(C)]
#[derive(Debug)]
pub struct TagManager {
    pub tags: StdVec<StdString>,
    pub tag_indices: StdMap<StdString, [u8; 4]>,
    pub max_tag_count: usize,
    pub name: StdString,
}
