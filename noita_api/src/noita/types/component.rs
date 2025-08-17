use crate::noita::types::{
    BitSet, CString, Component, Entity, EntityManager, StdMap, StdString, StdVec, TagManager,
};
use std::ptr;
#[repr(C)]
#[derive(Debug)]
pub struct ComponentData {
    pub vtable: &'static ComponentVTable,
    pub local_id: usize,
    pub type_name: CString,
    pub type_id: usize,
    pub id: usize,
    pub enabled: bool,
    unk2: [u8; 3],
    pub tags: BitSet<8>,
    unk3: StdVec<usize>,
    unk4: usize,
}
impl Default for ComponentData {
    fn default() -> Self {
        Self {
            vtable: &ComponentVTable {},
            local_id: 0,
            type_name: CString(ptr::null()),
            type_id: 0,
            id: 0,
            enabled: false,
            unk2: [0; 3],
            tags: Default::default(),
            unk3: StdVec::null(),
            unk4: 0,
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
pub struct ComponentBufferVTable {
    //TODO should be a union
}
#[repr(C)]
#[derive(Debug, Default)]
pub struct ComponentTypeManager {
    pub next_id: usize,
    pub component_buffer_indices: StdMap<StdString, usize>,
}
impl ComponentTypeManager {
    pub fn get<C: Component>(&self, entity_manager: &EntityManager) -> &ComponentBuffer {
        let index = self
            .component_buffer_indices
            .get(C::STD_NAME)
            .copied()
            .unwrap();
        let mgr = entity_manager.component_buffers.get(index).unwrap();
        unsafe { mgr.as_ref() }.unwrap()
    }
    pub fn get_mut<C: Component>(
        &mut self,
        entity_manager: &mut EntityManager,
    ) -> &mut ComponentBuffer {
        let index = self
            .component_buffer_indices
            .get(C::STD_NAME)
            .copied()
            .unwrap();
        let mgr = entity_manager.component_buffers.get(index).unwrap();
        unsafe { mgr.as_mut() }.unwrap()
    }
}
#[test]
fn test_com_create() {
    let mut em = EntityManager::default();
    let cm = &mut ComponentTypeManager::default();
    let id = &mut 0;
    {
        let mut com_buffer = ComponentBuffer::default();
        let com = &mut com_buffer as *mut _;
        em.component_buffers.push(com);
        let mut node = crate::noita::types::StdMapNode::default();
        node.key = StdString::from_str("WalletComponent");
        node.value = 0usize;
        unsafe { cm.component_buffer_indices.root.as_mut().unwrap() }.parent = &mut node as *mut _;
    }
    let ent = em.create();
    {
        em.create_component::<crate::noita::types::WalletComponent>(ent, id, cm);
        println!(
            "{:?}",
            em.get_component_buffer::<crate::noita::types::WalletComponent>(cm)
        );
        em.create_component::<crate::noita::types::WalletComponent>(ent, id, cm);
        println!(
            "{:?}",
            em.get_component_buffer::<crate::noita::types::WalletComponent>(cm)
        );
        em.create_component::<crate::noita::types::WalletComponent>(ent, id, cm);
        println!(
            "{:?}",
            em.get_component_buffer::<crate::noita::types::WalletComponent>(cm)
        );
        em.create_component::<crate::noita::types::WalletComponent>(ent, id, cm);
        println!(
            "{:?}",
            em.get_component_buffer::<crate::noita::types::WalletComponent>(cm)
        );
    }
    let mut coms = em.iter_components::<crate::noita::types::WalletComponent>(ent.entry, cm);
    println!(
        "{:?}",
        em.get_component_buffer::<crate::noita::types::WalletComponent>(cm)
            .component_list
    );
    println!(
        "{:?}",
        em.get_component_buffer::<crate::noita::types::WalletComponent>(cm)
            .component_list
            .get(0)
    );
    println!("{:?}", coms.next());
    println!("{:?}", coms.next());
    println!("{:?}", coms.next());
    println!("{:?}", coms.next());
    println!("{:?}", coms.next());
    println!("{:?}", coms.next());
}
#[repr(C)]
#[derive(Debug)]
pub struct ComponentBuffer {
    pub vtable: &'static ComponentBufferVTable,
    pub end: usize,
    unk: [isize; 2],
    pub entity_entry: StdVec<usize>,
    pub entities: StdVec<*mut Entity>,
    pub prev: StdVec<usize>,
    pub next: StdVec<usize>,
    pub component_list: StdVec<*mut ComponentData>,
}
impl Default for ComponentBuffer {
    fn default() -> Self {
        Self {
            vtable: &ComponentBufferVTable {},
            end: (-1isize).cast_unsigned(),
            unk: [0, 0],
            entity_entry: Default::default(),
            entities: Default::default(),
            prev: Default::default(),
            next: Default::default(),
            component_list: Default::default(),
        }
    }
}
impl ComponentBuffer {
    pub fn create<C: Component>(
        &mut self,
        entity: &mut Entity,
        id: usize,
        type_id: usize,
    ) -> &'static mut C {
        let com = C::default(ComponentData {
            vtable: C::VTABLE,
            local_id: self.component_list.len(),
            type_name: C::C_NAME,
            type_id,
            id,
            enabled: false,
            unk2: [0; 3],
            tags: Default::default(),
            unk3: StdVec::null(),
            unk4: 0,
        });
        let com = Box::leak(Box::new(com));
        let index = self.component_list.len();
        self.component_list.push((com as *mut C).cast());
        if self.entities.len() > index {
            self.entities[index] = entity;
        } else {
            while self.entities.len() < index {
                self.entities.push(ptr::null_mut())
            }
            self.entities.push(entity);
        }
        while self.entity_entry.len() <= entity.entry {
            self.entity_entry.push(self.end)
        }
        let mut off;
        let mut last = self.end;
        if let Some(e) = self.entity_entry.get(entity.entry).copied()
            && e != self.end
        {
            off = e;
            while let Some(next) = self.next.get(off).copied() {
                last = off;
                if next == self.end {
                    break;
                }
                off = next;
            }
            while self.next.len() <= index {
                self.next.push(self.end)
            }
            self.next[off] = index;
            while self.prev.len() <= index {
                self.prev.push(self.end)
            }
            self.prev[index] = last;
        } else {
            off = index;
            self.entity_entry[entity.entry] = off;
            while self.next.len() <= index {
                self.next.push(self.end)
            }
            self.next[off] = self.end;
        }
        com
    }
    pub fn iter_components(&self, entry: usize) -> ComponentIter {
        if let Some(off) = self.entity_entry.get(entry) {
            ComponentIter {
                component_list: self.component_list.copy(),
                off: *off,
                next: self.next.copy(),
                prev: self.prev.copy(),
                end: self.end,
            }
        } else {
            ComponentIter {
                component_list: StdVec::null(),
                off: 0,
                next: StdVec::null(),
                prev: StdVec::null(),
                end: 0,
            }
        }
    }
    pub fn iter_components_mut(&mut self, entry: usize) -> ComponentIterMut {
        if let Some(off) = self.entity_entry.get(entry) {
            ComponentIterMut {
                component_list: self.component_list.copy(),
                off: *off,
                next: self.next.copy(),
                prev: self.prev.copy(),
                end: self.end,
            }
        } else {
            ComponentIterMut {
                component_list: StdVec::null(),
                off: 0,
                next: StdVec::null(),
                prev: StdVec::null(),
                end: 0,
            }
        }
    }
    pub fn iter_every_component(&self) -> impl DoubleEndedIterator<Item = &'static ComponentData> {
        self.component_list
            .as_ref()
            .iter()
            .filter_map(|c| unsafe { c.as_ref() })
    }
    pub fn iter_every_component_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &'static mut ComponentData> {
        self.component_list
            .as_mut()
            .iter_mut()
            .filter_map(|c| unsafe { c.as_mut() })
    }
    pub fn iter_components_with_tag(
        &self,
        tag_manager: &TagManager<u8>,
        entry: usize,
        tag: &StdString,
    ) -> impl DoubleEndedIterator<Item = &'static ComponentData> {
        self.iter_components(entry)
            .filter(|c| c.tags.has_tag(tag_manager, tag))
    }
    pub fn iter_components_with_tag_mut(
        &mut self,
        tag_manager: &TagManager<u8>,
        entry: usize,
        tag: &StdString,
    ) -> impl DoubleEndedIterator<Item = &'static mut ComponentData> {
        self.iter_components_mut(entry)
            .filter(|c| c.tags.has_tag(tag_manager, tag))
    }
    pub fn iter_enabled_components(
        &self,
        entry: usize,
    ) -> impl DoubleEndedIterator<Item = &'static ComponentData> {
        self.iter_components(entry).filter(|c| c.enabled)
    }
    pub fn iter_disabled_components(
        &self,
        entry: usize,
    ) -> impl DoubleEndedIterator<Item = &'static ComponentData> {
        self.iter_components(entry).filter(|c| !c.enabled)
    }
    pub fn iter_enabled_components_mut(
        &mut self,
        entry: usize,
    ) -> impl DoubleEndedIterator<Item = &'static mut ComponentData> {
        self.iter_components_mut(entry).filter(|c| c.enabled)
    }
    pub fn iter_disabled_components_mut(
        &mut self,
        entry: usize,
    ) -> impl DoubleEndedIterator<Item = &'static mut ComponentData> {
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
    next: StdVec<usize>,
    prev: StdVec<usize>,
}

impl Iterator for ComponentIter {
    type Item = &'static ComponentData;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.off == self.end {
                return None;
            }
            let com = self.component_list.get(self.off)?.as_ref();
            self.off = *self.next.get(self.off)?;
            com
        }
    }
}

impl DoubleEndedIterator for ComponentIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.off == self.end {
                return None;
            }
            let com = self.component_list.get(self.off)?.as_ref();
            self.off = *self.prev.get(self.off)?;
            com
        }
    }
}
#[derive(Debug)]
pub struct ComponentIterMut {
    component_list: StdVec<*mut ComponentData>,
    off: usize,
    end: usize,
    next: StdVec<usize>,
    prev: StdVec<usize>,
}

impl Iterator for ComponentIterMut {
    type Item = &'static mut ComponentData;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.off == self.end {
                return None;
            }
            let com = self.component_list.get(self.off)?.as_mut();
            self.off = *self.next.get(self.off)?;
            com
        }
    }
}
impl DoubleEndedIterator for ComponentIterMut {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.off == self.end {
                return None;
            }
            let com = self.component_list.get(self.off)?.as_mut();
            self.off = *self.prev.get(self.off)?;
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
    pub fn add_tag(&mut self, tag_manager: &mut TagManager<u8>, tag: &StdString) {
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
#[repr(C)]
#[derive(Debug, Default)]
pub struct ComponentSystemManager {
    pub list: StdVec<&'static ComponentSystem>,
}
#[repr(C)]
#[derive(Debug)]
pub struct ComponentSystem {
    pub vtable: &'static ComponentSystemVTable,
    pub unk: [*const usize; 2],
    pub name: StdString,
}
#[repr(C)]
#[derive(Debug)]
pub struct ComponentSystemVTable {}
