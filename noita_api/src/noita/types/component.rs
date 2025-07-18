use crate::noita::types::CString;
use crate::noita::types::entity::Entity;
#[repr(C)]
#[derive(Debug)]
pub struct Component {
    pub vtable: &'static ComponentVTable,
    unk1: isize,
    pub type_name: CString,
    pub type_id: isize,
    pub id: isize,
    pub enabled: bool,
    unk2: [u8; 3],
    pub tags: [isize; 8],
    unk3: [isize; 3],
    //pub data: D,
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
pub struct ComponentManager {
    pub vtable: *const ComponentManagerVTable,
    pub end: isize,
    unk: [isize; 2],
    pub entity_entry: *mut isize,
    unk2: [isize; 8],
    pub next: *mut isize,
    unk3: isize,
    unk4: isize,
    pub component_list: *mut *mut Component,
    //TODO Unknown
}
impl ComponentManager {
    pub fn iter_components(&self, ent: &'static Entity) -> ComponentIter {
        unsafe {
            if let Some(off) = self.entity_entry.offset(ent.entry).as_ref() {
                ComponentIter {
                    component_list: self.component_list as *const *const Component,
                    off: *off,
                    next: self.next,
                    end: self.end,
                }
            } else {
                ComponentIter {
                    component_list: std::ptr::null_mut(),
                    off: 0,
                    next: std::ptr::null_mut(),
                    end: 0,
                }
            }
        }
    }
    pub fn iter_components_mut(&mut self, ent: &'static Entity) -> ComponentIterMut {
        unsafe {
            if let Some(off) = self.entity_entry.offset(ent.entry).as_ref() {
                ComponentIterMut {
                    component_list: self.component_list,
                    off: *off,
                    next: self.next,
                    end: self.end,
                }
            } else {
                ComponentIterMut {
                    component_list: std::ptr::null_mut(),
                    off: 0,
                    next: std::ptr::null_mut(),
                    end: 0,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ComponentIter {
    component_list: *const *const Component,
    off: isize,
    end: isize,
    next: *const isize,
}

impl Iterator for ComponentIter {
    type Item = &'static Component;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.off == self.end {
                return None;
            }
            let com = self.component_list.offset(self.off).as_ref()?.as_ref();
            if let Some(n) = self.next.offset(self.off).as_ref() {
                self.off = *n
            } else {
                self.off = self.end
            }
            com
        }
    }
}
#[derive(Debug)]
pub struct ComponentIterMut {
    component_list: *const *mut Component,
    off: isize,
    end: isize,
    next: *const isize,
}

impl Iterator for ComponentIterMut {
    type Item = &'static mut Component;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.off == self.end {
                return None;
            }
            let com = self.component_list.offset(self.off).as_ref()?.as_mut();
            if let Some(n) = self.next.offset(self.off).as_ref() {
                self.off = *n
            } else {
                self.off = self.end
            }
            com
        }
    }
}
