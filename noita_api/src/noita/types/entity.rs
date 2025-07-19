use crate::noita::types::component::{Component, ComponentManager};
use crate::noita::types::{StdString, StdVec};
use std::slice;
impl EntityManager {
    pub fn get_entity(&self, id: isize) -> Option<&'static Entity> {
        unsafe {
            let o = self
                .entities
                .as_ref()
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
            let o = self
                .entities
                .as_ref()
                .iter()
                .find_map(|c| c.as_ref().map(|c| c.id - c.entry))
                .unwrap_or(id);
            let start = self.entities.start.offset(id - o);
            let list = slice::from_raw_parts(start, self.entities.len() - (id - o) as usize);
            list.iter().find_map(|c| c.as_mut().filter(|c| c.id == id))
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
    ) -> impl Iterator<Item = &'static Component> {
        self.iter_component_managers()
            .flat_map(move |c| c.iter_components(ent))
    }
    pub fn iter_all_components_mut(
        &mut self,
        ent: &'static Entity,
    ) -> impl Iterator<Item = &'static mut Component> {
        self.iter_component_managers_mut()
            .flat_map(move |c| c.iter_components_mut(ent))
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
    tags: [isize; 16],
    pub x: f32,
    pub y: f32,
    pub angle_a: f32,
    pub angle: f32,
    pub rot90_a: f32,
    pub rot90: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub children: *mut StdVec<*mut Entity>,
    pub parent: *mut Entity,
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
    pub vtable: *const EntityManagerVTable,
    pub next_entity_id: usize,
    pub free_ids: StdVec<usize>,
    pub entities: StdVec<*mut Entity>,
    pub entity_buckets: StdVec<StdVec<*mut Entity>>,
    pub component_managers: StdVec<*mut ComponentManager>,
}
pub struct EntityManagerVTable {
    //TODO
}
