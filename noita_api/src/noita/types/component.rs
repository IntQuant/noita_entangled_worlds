use crate::noita::types::entity::Entity;
use crate::noita::types::{BitSet, CString, StdMap, StdString, StdVec, TagManager};
#[repr(C)]
#[derive(Debug)]
pub struct ComponentData {
    pub vtable: &'static ComponentVTable,
    unk1: isize,
    pub type_name: CString,
    pub type_id: isize,
    pub id: isize,
    pub enabled: bool,
    unk2: [u8; 3],
    pub tags: BitSet<8>,
    unk3: [isize; 4],
}
impl ComponentData {
    pub fn has_tag(&'static self, tag_manager: &TagManager<u8>, tag: &StdString) -> bool {
        if let Some(n) = tag_manager.tag_indices.get(tag) {
            self.tags.get(*n)
        } else {
            false
        }
    }
    pub fn add_tag(&'static mut self, tag_manager: &TagManager<u8>, tag: &StdString) {
        if let Some(n) = tag_manager.tag_indices.get(tag) {
            self.tags.set(*n, true)
        }
        //TODO
    }
    pub fn remove_tag(&'static mut self, tag_manager: &TagManager<u8>, tag: &StdString) {
        if let Some(n) = tag_manager.tag_indices.get(tag) {
            self.tags.set(*n, false)
        }
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct ComponentVTable {
    //TODO should be a union
}
#[repr(C)]
#[derive(Debug)]
pub struct ComponentManagerVTable {
    //TODO should be a union
}
#[repr(C)]
#[derive(Debug)]
pub struct ComponentTypeManager {
    pub next_id: usize,
    pub component_manager_indices: StdMap<StdString, usize>,
}
#[repr(C)]
#[derive(Debug)]
pub struct ComponentManager {
    pub vtable: &'static ComponentManagerVTable,
    pub end: usize,
    unk: [isize; 2],
    pub entity_entry: StdVec<usize>,
    unk2: [isize; 6],
    pub next: *mut usize,
    unk3: [isize; 2],
    pub component_list: StdVec<*mut ComponentData>,
}
impl ComponentManager {
    pub fn iter_components(&self, ent: &'static Entity) -> ComponentIter {
        if let Some(off) = self.entity_entry.get(ent.entry) {
            ComponentIter {
                component_list: self.component_list.copy(),
                off: *off,
                next: self.next,
                end: self.end,
            }
        } else {
            ComponentIter {
                component_list: StdVec {
                    start: std::ptr::null_mut(),
                    end: std::ptr::null_mut(),
                    cap: std::ptr::null_mut(),
                },
                off: 0,
                next: std::ptr::null_mut(),
                end: 0,
            }
        }
    }
    pub fn iter_components_mut(&mut self, ent: &'static Entity) -> ComponentIterMut {
        if let Some(off) = self.entity_entry.get(ent.entry) {
            ComponentIterMut {
                component_list: self.component_list.copy(),
                off: *off,
                next: self.next,
                end: self.end,
            }
        } else {
            ComponentIterMut {
                component_list: StdVec {
                    start: std::ptr::null_mut(),
                    end: std::ptr::null_mut(),
                    cap: std::ptr::null_mut(),
                },
                off: 0,
                next: std::ptr::null_mut(),
                end: 0,
            }
        }
    }
}

#[derive(Debug)]
pub struct ComponentIter {
    component_list: StdVec<*mut ComponentData>,
    off: usize,
    end: usize,
    next: *const usize,
}

impl Iterator for ComponentIter {
    type Item = &'static ComponentData;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.off == self.end {
                return None;
            }
            let com = self.component_list.get(self.off)?.as_ref();
            self.off = self.next.add(self.off).read();
            com
        }
    }
}
#[derive(Debug)]
pub struct ComponentIterMut {
    component_list: StdVec<*mut ComponentData>,
    off: usize,
    end: usize,
    next: *const usize,
}

impl Iterator for ComponentIterMut {
    type Item = &'static mut ComponentData;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.off == self.end {
                return None;
            }
            let com = self.component_list.get(self.off)?.as_mut();
            self.off = self.next.add(self.off).read();
            com
        }
    }
}
