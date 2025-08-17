use crate::noita::types::component::{ComponentBuffer, ComponentData};
use crate::noita::types::{
    Component, ComponentTypeManager, Inventory2Component, StdMap, StdString, StdVec, Vec2,
};
use std::{mem, slice};
impl EntityManager {
    pub fn create(&mut self) -> &'static mut Entity {
        self.max_entity_id += 1;
        let ent = Entity {
            id: self.max_entity_id,
            entry: 0,
            filename_index: 0,
            kill_flag: false,
            padding: [0; 3],
            unknown1: 0,
            name: StdString::default(),
            unknown2: 0,
            tags: BitSet::default(),
            transform: Transform::default(),
            children: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
        };
        let ent = Box::leak(Box::new(ent));
        if let Some(entry) = self.free_ids.pop() {
            ent.entry = entry;
            self.entities[entry] = ent;
        } else {
            ent.entry = self.entities.len();
            self.entities.push(ent);
        }
        ent
    }
    pub fn set_all_components(&mut self, ent: &mut Entity, enabled: bool) {
        self.iter_component_buffers_mut().for_each(|c| {
            c.iter_components_mut(ent.entry)
                .for_each(|c| c.enabled = enabled)
        })
    }
    pub fn set_components<C: Component>(
        &mut self,
        component_type_manager: &mut ComponentTypeManager,
        ent: &mut Entity,
        enabled: bool,
    ) {
        component_type_manager
            .get_mut::<C>(self)
            .iter_components_mut(ent.entry)
            .for_each(|c| c.enabled = enabled)
    }
    pub fn set_all_components_with_tag(
        &mut self,
        tag_manager: &TagManager<u8>,
        ent: &mut Entity,
        tag: &StdString,
        enabled: bool,
    ) {
        self.iter_component_buffers_mut().for_each(|c| {
            c.iter_components_mut(ent.entry)
                .filter(|c| c.tags.has_tag(tag_manager, tag))
                .for_each(|c| c.enabled = enabled)
        })
    }
    pub fn set_components_with_tag<C: Component>(
        &mut self,
        component_type_manager: &mut ComponentTypeManager,
        tag_manager: &TagManager<u8>,
        ent: &mut Entity,
        tag: &StdString,
        enabled: bool,
    ) {
        component_type_manager
            .get_mut::<C>(self)
            .iter_components_mut(ent.entry)
            .filter(|c| c.tags.has_tag(tag_manager, tag))
            .for_each(|c| c.enabled = enabled)
    }
    pub fn get_entities_with_tag(
        &self,
        tag: &StdString,
        tag_manager: &TagManager<u16>,
    ) -> impl DoubleEndedIterator<Item = &'static Entity> {
        let n = *tag_manager.tag_indices.get(tag).unwrap();
        self.entity_buckets
            .get(n as usize)
            .unwrap()
            .as_ref()
            .iter()
            .filter_map(|e| unsafe { e.as_ref() })
    }
    pub fn get_entities_with_tag_mut(
        &mut self,
        tag_manager: &TagManager<u16>,
        tag: &StdString,
    ) -> impl DoubleEndedIterator<Item = &'static mut Entity> {
        let n = *tag_manager.tag_indices.get(tag).unwrap();
        self.entity_buckets
            .get_mut(n as usize)
            .unwrap()
            .as_mut()
            .iter_mut()
            .filter_map(|e| unsafe { e.as_mut() })
    }
    pub fn get_entity_with_name(&self, name: StdString) -> Option<&'static Entity> {
        self.entities
            .as_ref()
            .iter()
            .filter_map(|a| unsafe { a.as_ref() })
            .find(|e| e.name == name)
    }
    pub fn get_entity_with_name_mut(&mut self, name: StdString) -> Option<&'static mut Entity> {
        self.entities
            .as_ref()
            .iter()
            .filter_map(|a| unsafe { a.as_mut() })
            .find(|e| e.name == name)
    }
    pub fn get_entity(&self, id: usize) -> Option<&'static Entity> {
        self.entities
            .as_ref()
            .iter()
            .filter_map(|c| unsafe { c.as_ref() })
            .find(|ent| ent.id == id)
    }
    pub fn get_entity_mut(&mut self, id: usize) -> Option<&'static mut Entity> {
        self.entities
            .as_mut()
            .iter_mut()
            .filter_map(|c| unsafe { c.as_mut() })
            .find(|ent| ent.id == id)
    }
    pub fn iter_entities_with_tag(
        &self,
        tag_manager: &TagManager<u16>,
        tag: &StdString,
    ) -> impl DoubleEndedIterator<Item = &'static Entity> {
        unsafe {
            if let Some(n) = tag_manager.tag_indices.get(tag).copied()
                && let Some(v) = self.entity_buckets.get(n as usize)
            {
                v.as_ref()
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
    ) -> impl DoubleEndedIterator<Item = &'static mut Entity> {
        unsafe {
            if let Some(n) = tag_manager.tag_indices.get(tag).copied()
                && let Some(v) = self.entity_buckets.get_mut(n as usize)
            {
                v.as_mut()
            } else {
                &mut []
            }
            .iter_mut()
            .filter_map(|e| e.as_mut())
        }
    }
    pub fn iter_entities(&self) -> impl DoubleEndedIterator<Item = &'static Entity> {
        self.entities
            .as_ref()
            .iter()
            .filter_map(|c| unsafe { c.as_ref() })
    }
    pub fn iter_entities_mut(&mut self) -> impl DoubleEndedIterator<Item = &'static mut Entity> {
        self.entities
            .as_mut()
            .iter_mut()
            .filter_map(|c| unsafe { c.as_mut() })
    }
    pub fn iter_component_buffers(
        &self,
    ) -> impl DoubleEndedIterator<Item = &'static ComponentBuffer> {
        self.component_buffers
            .as_ref()
            .iter()
            .filter_map(|c| unsafe { c.as_ref() })
    }
    pub fn iter_component_buffers_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &'static mut ComponentBuffer> {
        self.component_buffers
            .as_mut()
            .iter_mut()
            .filter_map(|c| unsafe { c.as_mut() })
    }
    pub fn iter_components<C: Component + 'static>(
        &self,
        entry: usize,
        component_type_manager: &ComponentTypeManager,
    ) -> impl DoubleEndedIterator<Item = &'static C> {
        let index = component_type_manager
            .component_buffer_indices
            .get(C::STD_NAME)
            .copied()
            .unwrap();
        let mgr = self.component_buffers.get(index).unwrap();
        unsafe { mgr.as_ref() }
            .unwrap()
            .iter_components(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn iter_components_mut<C: Component + 'static>(
        &mut self,
        entry: usize,
        component_type_manager: &mut ComponentTypeManager,
    ) -> impl DoubleEndedIterator<Item = &'static mut C> {
        component_type_manager
            .get_mut::<C>(self)
            .iter_components_mut(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn iter_enabled_components<C: Component + 'static>(
        &self,
        entry: usize,
        component_type_manager: &ComponentTypeManager,
    ) -> impl DoubleEndedIterator<Item = &'static C> {
        component_type_manager
            .get::<C>(self)
            .iter_enabled_components(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn iter_enabled_components_mut<C: Component + 'static>(
        &mut self,
        entry: usize,
        component_type_manager: &mut ComponentTypeManager,
    ) -> impl DoubleEndedIterator<Item = &'static mut C> {
        component_type_manager
            .get_mut::<C>(self)
            .iter_enabled_components_mut(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn iter_disabled_components<C: Component + 'static>(
        &self,
        entry: usize,
        component_type_manager: &ComponentTypeManager,
    ) -> impl DoubleEndedIterator<Item = &'static C> {
        component_type_manager
            .get::<C>(self)
            .iter_disabled_components(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn iter_disabled_components_mut<C: Component + 'static>(
        &mut self,
        entry: usize,
        component_type_manager: &mut ComponentTypeManager,
    ) -> impl DoubleEndedIterator<Item = &'static mut C> {
        component_type_manager
            .get_mut::<C>(self)
            .iter_disabled_components_mut(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    #[allow(clippy::mut_from_ref)]
    pub fn create_component<C: Component + 'static>(
        &self,
        entity: &mut Entity,
        max_component: &mut usize,
        component_type_manager: &mut ComponentTypeManager,
    ) -> &mut C {
        let index = component_type_manager
            .component_buffer_indices
            .get(C::STD_NAME)
            .copied()
            .unwrap();
        let mgr = self.component_buffers.get(index).unwrap();
        let com = unsafe { mgr.as_mut() }
            .unwrap()
            .create::<C>(entity, *max_component, index);
        *max_component += 1;
        com
    }
    pub fn get_component_buffer<'a, C: Component + 'static>(
        &self,
        component_type_manager: &'a ComponentTypeManager,
    ) -> &'a ComponentBuffer {
        //TODO this needs to deal with when it does not exist
        component_type_manager.get::<C>(self)
    }
    pub fn get_component_buffer_mut<'a, C: Component + 'static>(
        &mut self,
        component_type_manager: &'a mut ComponentTypeManager,
    ) -> &'a mut ComponentBuffer {
        //TODO this needs to deal with when it does not exist
        component_type_manager.get_mut::<C>(self)
    }
    pub fn get_first_component<C: Component + 'static>(
        &self,
        entry: usize,
        component_type_manager: &ComponentTypeManager,
    ) -> Option<&'static C> {
        component_type_manager
            .get::<C>(self)
            .get_first(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn get_first_component_mut<C: Component + 'static>(
        &mut self,
        entry: usize,
        component_type_manager: &mut ComponentTypeManager,
    ) -> Option<&'static mut C> {
        component_type_manager
            .get_mut::<C>(self)
            .get_first_mut(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn get_first_enabled_component<C: Component + 'static>(
        &self,
        entry: usize,
        component_type_manager: &ComponentTypeManager,
    ) -> Option<&'static C> {
        component_type_manager
            .get::<C>(self)
            .get_first_enabled(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn get_first_enabled_component_mut<C: Component + 'static>(
        &mut self,
        entry: usize,
        component_type_manager: &mut ComponentTypeManager,
    ) -> Option<&'static mut C> {
        component_type_manager
            .get_mut::<C>(self)
            .get_first_enabled_mut(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn get_first_disabled_component<C: Component + 'static>(
        &self,
        entry: usize,
        component_type_manager: &ComponentTypeManager,
    ) -> Option<&'static C> {
        component_type_manager
            .get::<C>(self)
            .get_first_disabled(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn get_first_disabled_component_mut<C: Component + 'static>(
        &mut self,
        entry: usize,
        component_type_manager: &mut ComponentTypeManager,
    ) -> Option<&'static mut C> {
        component_type_manager
            .get_mut::<C>(self)
            .get_first_disabled_mut(entry)
            .map(|c| unsafe { mem::transmute(c) })
    }
    pub fn iter_every_component(&self) -> impl DoubleEndedIterator<Item = &'static ComponentData> {
        self.iter_component_buffers()
            .flat_map(move |c| c.iter_every_component())
    }
    pub fn iter_every_component_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &'static mut ComponentData> {
        self.iter_component_buffers_mut()
            .flat_map(move |c| c.iter_every_component_mut())
    }
    pub fn iter_all_components(
        &self,
        entry: usize,
    ) -> impl DoubleEndedIterator<Item = &'static ComponentData> {
        self.iter_component_buffers()
            .flat_map(move |c| c.iter_components(entry))
    }
    pub fn iter_all_components_mut(
        &mut self,
        entry: usize,
    ) -> impl DoubleEndedIterator<Item = &'static mut ComponentData> {
        self.iter_component_buffers_mut()
            .flat_map(move |c| c.iter_components_mut(entry))
    }
    pub fn iter_in_radius(
        &self,
        pos: Vec2,
        radius: f32,
    ) -> impl DoubleEndedIterator<Item = &'static Entity> {
        self.entities
            .as_ref()
            .iter()
            .filter_map(|e| unsafe { e.as_ref() })
            .filter(move |e| pos.abs2(&e.transform.pos) < radius * radius)
    }
    pub fn iter_in_radius_with_tag(
        &self,
        pos: Vec2,
        radius: f32,
        tag: &StdString,
        tag_manager: &TagManager<u16>,
    ) -> impl DoubleEndedIterator<Item = &'static Entity> {
        if let Some(tag) = tag_manager.tag_indices.get(tag).copied()
            && let Some(ents) = self.entity_buckets.get(tag as usize)
        {
            ents.as_ref()
        } else {
            &[]
        }
        .iter()
        .filter_map(|e| unsafe { e.as_ref() })
        .filter(move |e| pos.abs2(&e.transform.pos) < radius * radius)
    }
    pub fn iter_in_radius_mut(
        &mut self,
        pos: Vec2,
        radius: f32,
    ) -> impl DoubleEndedIterator<Item = &'static mut Entity> {
        self.entities
            .as_mut()
            .iter_mut()
            .filter_map(|e| unsafe { e.as_mut() })
            .filter(move |e| pos.abs2(&e.transform.pos) < radius * radius)
    }
    pub fn iter_in_radius_with_tag_mut(
        &mut self,
        pos: Vec2,
        radius: f32,
        tag: &StdString,
        tag_manager: &TagManager<u16>,
    ) -> impl DoubleEndedIterator<Item = &'static mut Entity> {
        if let Some(tag) = tag_manager.tag_indices.get(tag).copied()
            && let Some(ents) = self.entity_buckets.get_mut(tag as usize)
        {
            ents.as_mut()
        } else {
            &mut []
        }
        .iter_mut()
        .filter_map(|e| unsafe { e.as_mut() })
        .filter(move |e| pos.abs2(&e.transform.pos) < radius * radius)
    }
    pub fn get_with_name(&self, name: StdString) -> Option<&'static Entity> {
        self.entities.as_ref().iter().find_map(|e| {
            unsafe { e.as_ref() }.and_then(|e| if e.name == name { Some(e) } else { None })
        })
    }
    pub fn get_closest(&self, pos: Vec2) -> Option<&'static Entity> {
        self.entities
            .as_ref()
            .iter()
            .filter_map(|e| unsafe { e.as_ref().map(|e| (pos.abs2(&e.transform.pos), e)) })
            .min_by(|(a, _), (b, _)| a.total_cmp(b))
            .map(|(_, e)| e)
    }
    pub fn get_closest_with_tag(
        &self,
        pos: Vec2,
        tag: &StdString,
        tag_manager: &TagManager<u16>,
    ) -> Option<&'static Entity> {
        tag_manager.tag_indices.get(tag).copied().and_then(|tag| {
            self.entity_buckets.get(tag as usize).and_then(|b| {
                b.as_ref()
                    .iter()
                    .filter_map(|e| unsafe { e.as_ref().map(|e| (pos.abs2(&e.transform.pos), e)) })
                    .min_by(|(a, _), (b, _)| a.total_cmp(b))
                    .map(|(_, e)| e)
            })
        })
    }
    pub fn get_with_name_mut(&mut self, name: StdString) -> Option<&'static mut Entity> {
        self.entities.as_mut().iter_mut().find_map(|e| {
            unsafe { e.as_mut() }.and_then(|e| if e.name == name { Some(e) } else { None })
        })
    }
    pub fn get_closest_mut(&mut self, pos: Vec2) -> Option<&'static mut Entity> {
        self.entities
            .as_mut()
            .iter_mut()
            .filter_map(|e| unsafe { e.as_mut().map(|e| (pos.abs2(&e.transform.pos), e)) })
            .min_by(|(a, _), (b, _)| a.total_cmp(b))
            .map(|(_, e)| e)
    }
    pub fn get_closest_with_tag_mut(
        &mut self,
        pos: Vec2,
        tag: &StdString,
        tag_manager: &TagManager<u16>,
    ) -> Option<&'static mut Entity> {
        tag_manager.tag_indices.get(tag).copied().and_then(|tag| {
            self.entity_buckets.get_mut(tag as usize).and_then(|b| {
                b.as_mut()
                    .iter_mut()
                    .filter_map(|e| unsafe { e.as_mut().map(|e| (pos.abs2(&e.transform.pos), e)) })
                    .min_by(|(a, _), (b, _)| a.total_cmp(b))
                    .map(|(_, e)| e)
            })
        })
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct BitSet<const N: usize>(pub [isize; N]);
impl BitSet<16> {
    pub fn get(&self, n: u16) -> bool {
        let out_index = n / 32;
        let in_index = n % 32;
        self.0[out_index as usize] & (1 << in_index) != 0
    }
    pub fn set(&mut self, n: u16, value: bool) {
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
    pub fn has_tag(&'static self, tag_manager: &TagManager<u16>, tag: &StdString) -> bool {
        if let Some(n) = tag_manager.tag_indices.get(tag) {
            self.get(*n)
        } else {
            false
        }
    }
    pub fn get_tags(
        &'static self,
        tag_manager: &TagManager<u16>,
    ) -> impl Iterator<Item = &'static StdString> {
        tag_manager
            .tag_indices
            .iter()
            .filter_map(|(a, b)| if self.get(*b) { Some(a) } else { None })
    }
}
impl<const N: usize> Default for BitSet<N> {
    fn default() -> Self {
        Self([0; N])
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct Entity {
    pub id: usize,
    pub entry: usize,
    pub filename_index: usize,
    pub kill_flag: bool,
    padding: [u8; 3],
    unknown1: isize,
    pub name: StdString,
    unknown2: isize,
    pub tags: BitSet<16>,
    pub transform: Transform,
    pub children: *mut StdVec<*mut Entity>,
    pub parent: *mut Entity,
}
#[repr(C)]
#[derive(Debug, Default)]
pub struct Transform {
    pub pos: Vec2,
    pub angle: Vec2,
    pub rot90: Vec2,
    pub scale: Vec2,
}

impl Entity {
    pub fn kill(&mut self) {
        self.kill_flag = true;
        self.iter_children_mut().for_each(|e| e.kill());
    }
    pub fn kill_safe(&mut self, inventory: &mut Inventory) {
        if inventory.wand_pickup == self {
            inventory.wand_pickup = std::ptr::null_mut();
            inventory.pickup_state = 0;
        }
        self.kill();
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
    pub fn add_tag(
        &'static mut self,
        tag_manager: &mut TagManager<u16>,
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
    pub max_entity_id: usize,
    pub free_ids: StdVec<usize>,
    pub entities: StdVec<*mut Entity>,
    pub entity_buckets: StdVec<StdVec<*mut Entity>>,
    pub component_buffers: StdVec<*mut ComponentBuffer>,
}
impl Default for EntityManager {
    fn default() -> Self {
        Self {
            vtable: &EntityManagerVTable {},
            max_entity_id: 0,
            free_ids: Default::default(),
            entities: Default::default(),
            entity_buckets: Default::default(),
            component_buffers: Default::default(),
        }
    }
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
#[derive(Debug, Default)]
pub enum GameEffect {
    #[default]
    None = 0,
}
#[repr(C)]
#[derive(Debug)]
pub struct Inventory {
    pub entity: *mut Entity,
    pub inventory_quick: *mut Entity,
    pub inventory_full: *mut Entity,
    pub held_item_id: usize,
    pub switch_item_id: isize,
    pub inventory_component: *mut Inventory2Component,
    unk7b1: bool,
    pub item_placed: bool,
    unk7b3: bool,
    padding: u8,
    pub item_in_pickup_range: bool,
    padding2: u8,
    padding3: u8,
    padding4: u8,
    pub is_in_inventory: bool,
    unk9b2: bool,
    pub is_dragging: bool,
    padding5: u8,
    unk10: StdVec<isize>,
    pub pickup_state: usize,
    pub wand_pickup: *mut Entity,
    pub animation_state: usize,
    unk15: StdVec<[isize; 18]>,
}
