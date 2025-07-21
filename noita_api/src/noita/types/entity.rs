use crate::noita::types::component::{ComponentData, ComponentManager};
use crate::noita::types::{StdMap, StdString, StdVec, Vec2};
use std::slice;
impl EntityManager {
    pub fn get_entity_with_tag(
        &self,
        tag_manager: &TagManager<u16>,
        tag: &StdString,
    ) -> Option<&'static Entity> {
        unsafe {
            let n = *tag_manager.tag_indices.get(tag)?;
            self.entity_buckets.get(n as usize)?.get(0)?.as_ref()
        }
    }
    pub fn get_entity_with_tag_mut(
        &mut self,
        tag_manager: &TagManager<u16>,
        tag: &StdString,
    ) -> Option<&'static mut Entity> {
        unsafe {
            let n = *tag_manager.tag_indices.get(tag)?;
            self.entity_buckets.get_mut(n as usize)?.get(0)?.as_mut()
        }
    }
    pub fn get_entity(&self, id: isize) -> Option<&'static Entity> {
        unsafe {
            let o = self.entities.as_ref()[id as usize..]
                .iter()
                .find_map(|c| c.as_ref().map(|c| c.id - c.entry))
                .unwrap_or(id);
            let start = self.entities.start.offset(id - o);
            let list = slice::from_raw_parts(start, self.entities.len() - (id - o) as usize);
            list.iter().find_map(|c| c.as_ref().filter(|c| c.id == id))
        }
    }
    pub fn get_entity_mut(&mut self, id: isize) -> Option<&'static mut Entity> {
        unsafe {
            let o = self.entities.as_ref()[id as usize..]
                .iter()
                .find_map(|c| c.as_ref().map(|c| c.id - c.entry))
                .unwrap_or(id);
            let start = self.entities.start.offset(id - o);
            let list = slice::from_raw_parts(start, self.entities.len() - (id - o) as usize);
            list.iter().find_map(|c| c.as_mut().filter(|c| c.id == id))
        }
    }
    pub fn iter_entities_with_tag(
        &self,
        tag_manager: &TagManager<u16>,
        tag: &StdString,
    ) -> impl Iterator<Item = &'static Entity> {
        unsafe {
            if let Some(n) = tag_manager.tag_indices.get(tag).copied() {
                if let Some(v) = self.entity_buckets.get(n as usize) {
                    v.as_ref()
                } else {
                    &[]
                }
            } else {
                &[]
            }
            .iter()
            .filter_map(|e| e.as_ref())
        }
    }
    pub fn iter_entities_with_tag_mut(
        &mut self,
        tag_manager: &TagManager<u16>,
        tag: &StdString,
    ) -> impl Iterator<Item = &'static mut Entity> {
        unsafe {
            if let Some(n) = tag_manager.tag_indices.get(tag).copied() {
                if let Some(v) = self.entity_buckets.get_mut(n as usize) {
                    v.as_mut()
                } else {
                    &mut []
                }
            } else {
                &mut []
            }
            .iter_mut()
            .filter_map(|e| e.as_mut())
        }
    }
    pub fn iter_entities(&self) -> impl Iterator<Item = &'static Entity> {
        self.entities
            .as_ref()
            .iter()
            .filter_map(|c| unsafe { c.as_ref() })
    }
    pub fn iter_entities_mut(&mut self) -> impl Iterator<Item = &'static mut Entity> {
        self.entities
            .as_mut()
            .iter_mut()
            .filter_map(|c| unsafe { c.as_mut() })
    }
    pub fn iter_component_managers(&self) -> impl Iterator<Item = &'static ComponentManager> {
        self.component_managers
            .as_ref()
            .iter()
            .filter_map(|c| unsafe { c.as_ref() })
    }
    pub fn iter_component_managers_mut(
        &mut self,
    ) -> impl Iterator<Item = &'static mut ComponentManager> {
        self.component_managers
            .as_mut()
            .iter_mut()
            .filter_map(|c| unsafe { c.as_mut() })
    }
    pub fn iter_all_components(
        &self,
        ent: &'static Entity,
    ) -> impl Iterator<Item = &'static ComponentData> {
        self.iter_component_managers()
            .flat_map(move |c| c.iter_components(ent))
    }
    pub fn iter_all_components_mut(
        &mut self,
        ent: &'static Entity,
    ) -> impl Iterator<Item = &'static mut ComponentData> {
        self.iter_component_managers_mut()
            .flat_map(move |c| c.iter_components_mut(ent))
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct BitSet<const N: usize>([isize; N]);
impl BitSet<16> {
    #[inline]
    pub fn get(&self, n: u16) -> bool {
        let out_index = n / 32;
        let in_index = n % 32;
        self.0[out_index as usize] & (1 << in_index) != 0
    }
    #[inline]
    pub fn set(&mut self, n: u16, value: bool) {
        let out_index = n / 32;
        let in_index = n % 32;
        if value {
            self.0[out_index as usize] |= 1 << in_index
        } else {
            self.0[out_index as usize] &= !(1 << in_index)
        }
    }
}
impl BitSet<8> {
    #[inline]
    pub fn get(&self, n: u8) -> bool {
        let out_index = n / 32;
        let in_index = n % 32;
        self.0[out_index as usize] & (1 << in_index) != 0
    }
    #[inline]
    pub fn set(&mut self, n: u8, value: bool) {
        let out_index = n / 32;
        let in_index = n % 32;
        if value {
            self.0[out_index as usize] |= 1 << in_index
        } else {
            self.0[out_index as usize] &= !(1 << in_index)
        }
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct Entity {
    pub id: isize,
    pub entry: isize,
    pub filename_index: usize,
    pub kill_flag: isize,
    unknown1: isize,
    pub name: StdString,
    unknown2: isize,
    pub tags: BitSet<16>,
    pub transform: Transform,
    pub children: *mut StdVec<*mut Entity>,
    pub parent: *mut Entity,
}
#[repr(C)]
#[derive(Debug)]
pub struct Transform {
    pub pos: Vec2,
    pub angle: Vec2,
    pub rot90: Vec2,
    pub scale: Vec2,
}

impl Entity {
    pub fn kill(&mut self) {
        self.kill_flag = 1;
        self.iter_children_mut().for_each(|e| e.kill());
    }
    pub fn iter_children(&self) -> impl DoubleEndedIterator<Item = &'static Entity> {
        unsafe {
            if let Some(child) = self.children.as_ref() {
                let len = child.end.offset_from(child.start);
                slice::from_raw_parts(child.start, len as usize)
            } else {
                &[]
            }
            .iter()
            .filter_map(|e| e.as_ref())
        }
    }
    pub fn iter_children_mut(&mut self) -> impl DoubleEndedIterator<Item = &'static mut Entity> {
        unsafe {
            if let Some(child) = self.children.as_ref() {
                let len = child.end.offset_from(child.start);
                slice::from_raw_parts(child.start, len as usize)
            } else {
                &[]
            }
            .iter()
            .filter_map(|e| e.as_mut())
        }
    }
    pub fn iter_descendants(&'static self) -> impl Iterator<Item = &'static Entity> {
        DescendantIter {
            entitys: self.iter_children().rev().collect(),
        }
    }
    pub fn iter_descendants_mut(&'static mut self) -> impl Iterator<Item = &'static mut Entity> {
        DescendantIterMut {
            entitys: self.iter_children_mut().rev().collect(),
        }
    }
    pub fn parent(&self) -> Option<&'static Entity> {
        unsafe { self.parent.as_ref() }
    }
    pub fn parent_mut(&mut self) -> Option<&'static mut Entity> {
        unsafe { self.parent.as_mut() }
    }
    pub fn iter_ancestors(&'static self) -> impl Iterator<Item = &'static Entity> {
        AncestorIter {
            current: Some(self),
        }
    }
    pub fn iter_ancestors_mut(&'static mut self) -> impl Iterator<Item = &'static mut Entity> {
        AncestorIterMut {
            current: Some(self),
        }
    }
    pub fn root(&'static self) -> &'static Entity {
        if let Some(ent) = self.iter_ancestors().last() {
            ent
        } else {
            self
        }
    }
    pub fn root_mut(&'static mut self) -> &'static mut Entity {
        if self.parent.is_null() {
            self
        } else {
            self.iter_ancestors_mut().last().unwrap()
        }
    }
    pub fn has_tag(&'static self, tag_manager: &TagManager<u16>, tag: &StdString) -> bool {
        if let Some(n) = tag_manager.tag_indices.get(tag) {
            self.tags.get(*n)
        } else {
            false
        }
    }
    pub fn add_tag(
        &'static mut self,
        tag_manager: &TagManager<u16>,
        entity_manager: &mut EntityManager,
        tag: &StdString,
    ) {
        if let Some(n) = tag_manager.tag_indices.get(tag).copied()
            && !self.tags.get(n)
        {
            entity_manager
                .entity_buckets
                .get_mut(n as usize)
                .unwrap()
                .push(self);
            self.tags.set(n, true)
        }
        //TODO add tag if does not exist
    }
    pub fn remove_tag(
        &'static mut self,
        tag_manager: &TagManager<u16>,
        entity_manager: &mut EntityManager,
        tag: &StdString,
    ) {
        if let Some(n) = tag_manager.tag_indices.get(tag).copied()
            && self.tags.get(n)
        {
            let v = entity_manager.entity_buckets.get_mut(n as usize).unwrap();
            let Some(i) = v
                .as_ref()
                .iter()
                .position(|c| unsafe { c.as_ref() }.map(|c| c.id) == Some(self.id))
            else {
                unreachable!()
            };
            v.remove(i);
            self.tags.set(n, false)
        }
    }
}

#[derive(Debug)]
pub struct AncestorIter {
    current: Option<&'static Entity>,
}

impl Iterator for AncestorIter {
    type Item = &'static Entity;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            self.current = current.parent();
            self.current
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct AncestorIterMut {
    current: Option<&'static mut Entity>,
}

impl Iterator for AncestorIterMut {
    type Item = &'static mut Entity;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current.take() {
            self.current = unsafe { current.parent.as_mut() };
            unsafe { current.parent.as_mut() }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct DescendantIter {
    entitys: Vec<&'static Entity>,
}

impl Iterator for DescendantIter {
    type Item = &'static Entity;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ent) = self.entitys.pop() {
            self.entitys.extend(ent.iter_children().rev());
            Some(ent)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct DescendantIterMut {
    entitys: Vec<&'static mut Entity>,
}

impl Iterator for DescendantIterMut {
    type Item = &'static mut Entity;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ent) = self.entitys.pop() {
            self.entitys.extend(ent.iter_children_mut().rev());
            Some(ent)
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct EntityManager {
    pub vtable: &'static EntityManagerVTable,
    pub next_entity_id: usize,
    pub free_ids: StdVec<usize>,
    pub entities: StdVec<*mut Entity>,
    pub entity_buckets: StdVec<StdVec<*mut Entity>>,
    pub component_managers: StdVec<*mut ComponentManager>,
}
#[repr(C)]
#[derive(Debug)]
pub struct EntityManagerVTable {
    //TODO
}
#[repr(C)]
#[derive(Debug)]
pub struct TagManager<T: 'static> {
    pub tags: StdVec<StdString>,
    pub tag_indices: StdMap<StdString, T>,
    pub max_tag_count: usize,
    pub name: StdString,
}

#[repr(C)]
#[derive(Debug)]
pub struct SpriteStainSystem {}
#[repr(C)]
#[derive(Debug)]
pub enum GameEffect {
    None = 0,
}
#[repr(C)]
#[derive(Debug)]
pub struct Inventory {
    unk1: isize,
    unk2: isize,
    unk3: isize,
    held_item: isize,
    unk5: isize,
    unk6: isize,
    unk7: isize,
    item_near: isize,
    unk9: isize,
    unk10: isize,
    unk11: isize,
    unk12: isize,
    unk13: isize,
    wand_pickup: *mut Entity,
    unk15: isize,
    unk16: isize,
}
