use crate::noita::types::{
    BitSet, CString, Component, EntityManager, StdMap, StdString, StdVec, TagManager,
};
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
impl ComponentTypeManager {
    pub fn get<C: Component>(&self, entity_manager: &EntityManager) -> &ComponentManager {
        let index = self
            .component_manager_indices
            .get(C::STD_NAME)
            .copied()
            .unwrap();
        let mgr = entity_manager.component_managers.get(index).unwrap();
        unsafe { mgr.as_ref() }.unwrap()
    }
    pub fn get_mut<C: Component>(
        &mut self,
        entity_manager: &mut EntityManager,
    ) -> &mut ComponentManager {
        let index = self
            .component_manager_indices
            .get(C::STD_NAME)
            .copied()
            .unwrap();
        let mgr = entity_manager.component_managers.get(index).unwrap();
        unsafe { mgr.as_mut() }.unwrap()
    }
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
    pub fn iter_components(&self, entry: usize) -> ComponentIter {
        if let Some(off) = self.entity_entry.get(entry) {
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
    pub fn iter_components_mut(&mut self, entry: usize) -> ComponentIterMut {
        if let Some(off) = self.entity_entry.get(entry) {
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
    pub fn iter_components_with_tag(
        &self,
        tag_manager: &TagManager<u8>,
        entry: usize,
        tag: &StdString,
    ) -> impl Iterator<Item = &'static ComponentData> {
        self.iter_components(entry)
            .filter(|c| c.tags.has_tag(tag_manager, tag))
    }
    pub fn iter_components_with_tag_mut(
        &mut self,
        tag_manager: &TagManager<u8>,
        entry: usize,
        tag: &StdString,
    ) -> impl Iterator<Item = &'static mut ComponentData> {
        self.iter_components_mut(entry)
            .filter(|c| c.tags.has_tag(tag_manager, tag))
    }
    pub fn iter_enabled_components(
        &self,
        entry: usize,
    ) -> impl Iterator<Item = &'static ComponentData> {
        self.iter_components(entry).filter(|c| c.enabled)
    }
    pub fn iter_disabled_components(
        &self,
        entry: usize,
    ) -> impl Iterator<Item = &'static ComponentData> {
        self.iter_components(entry).filter(|c| !c.enabled)
    }
    pub fn iter_enabled_components_mut(
        &mut self,
        entry: usize,
    ) -> impl Iterator<Item = &'static mut ComponentData> {
        self.iter_components_mut(entry).filter(|c| c.enabled)
    }
    pub fn iter_disabled_components_mut(
        &mut self,
        entry: usize,
    ) -> impl Iterator<Item = &'static mut ComponentData> {
        self.iter_components_mut(entry).filter(|c| !c.enabled)
    }
    pub fn get_first(&self, entry: usize) -> Option<&'static ComponentData> {
        self.iter_components(entry).next()
    }
    pub fn get_first_mut(&mut self, entry: usize) -> Option<&'static mut ComponentData> {
        self.iter_components_mut(entry).next()
    }
    pub fn get_first_enabled(&self, entry: usize) -> Option<&'static ComponentData> {
        self.iter_enabled_components(entry).next()
    }
    pub fn get_first_disabled(&self, entry: usize) -> Option<&'static ComponentData> {
        self.iter_disabled_components(entry).next()
    }
    pub fn get_first_enabled_mut(&mut self, entry: usize) -> Option<&'static mut ComponentData> {
        self.iter_enabled_components_mut(entry).next()
    }
    pub fn get_first_disabled_mut(&mut self, entry: usize) -> Option<&'static mut ComponentData> {
        self.iter_disabled_components_mut(entry).next()
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
impl BitSet<8> {
    pub fn get(&self, n: u8) -> bool {
        let out_index = n / 32;
        let in_index = n % 32;
        self.0[out_index as usize] & (1 << in_index) != 0
    }
    pub fn set(&mut self, n: u8, value: bool) {
        let out_index = n / 32;
        let in_index = n % 32;
        if value {
            self.0[out_index as usize] |= 1 << in_index
        } else {
            self.0[out_index as usize] &= !(1 << in_index)
        }
    }
    pub fn count(&self) -> usize {
        let mut n = 0;
        for s in self.0 {
            n += s.count_ones()
        }
        n as usize
    }
    pub fn has_tag(&self, tag_manager: &TagManager<u8>, tag: &StdString) -> bool {
        if let Some(n) = tag_manager.tag_indices.get(tag) {
            self.get(*n)
        } else {
            false
        }
    }
    pub fn add_tag(&mut self, tag_manager: &TagManager<u8>, tag: &StdString) {
        if let Some(n) = tag_manager.tag_indices.get(tag) {
            self.set(*n, true)
        }
        //TODO
    }
    pub fn remove_tag(&mut self, tag_manager: &TagManager<u8>, tag: &StdString) {
        if let Some(n) = tag_manager.tag_indices.get(tag) {
            self.set(*n, false)
        }
    }
    pub fn get_tags(
        &self,
        tag_manager: &TagManager<u8>,
    ) -> impl Iterator<Item = &'static StdString> {
        tag_manager
            .tag_indices
            .iter()
            .filter_map(|(a, b)| if self.get(*b) { Some(a) } else { None })
    }
}
