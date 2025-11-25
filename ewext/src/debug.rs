use crate::ExtState;
use noita_api::addr_grabber::Globals;
use noita_api::noita::types::*;
use std::collections::HashMap;
use std::ops::DerefMut;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::*;
pub fn check_globals(_: &mut ExtState) -> eyre::Result<()> {
    let vftables = include_str!("../vftables.txt");
    let mut name_map = HashMap::new();
    let mut size_map = HashMap::new();
    for line in vftables.lines() {
        let mut split = line.split(' ');
        let Some(name) = split.next() else {
            continue;
        };
        let name = name.to_string();
        let Some(addr) = split.next() else {
            continue;
        };
        let addr = usize::from_str_radix(addr, 16)?;
        let Some(size) = split.next() else {
            continue;
        };
        let size = size.parse::<usize>().unwrap_or(0);
        name_map.insert(addr, name);
        size_map.insert(addr, size);
    }
    let mut map = Vec::new();
    for (addr, size) in size_map.into_iter() {
        let name = name_map.remove(&addr).unwrap();
        map.push((addr, name, size));
    }
    map.sort_by(|a, b| a.0.cmp(&b.0));
    let globals = Globals::default();
    let len = unsafe { GetProcessHeaps(&mut Vec::new()) };
    let mut heaps = vec![HANDLE::default(); len as usize];
    unsafe { GetProcessHeaps(&mut heaps) };
    let mut heapset = Vec::new();
    let mut entry: PROCESS_HEAP_ENTRY = PROCESS_HEAP_ENTRY::default();
    heaps.sort_by(|a, b| a.0.cmp(&b.0));
    for heap in heaps {
        while unsafe { HeapWalk(heap, &mut entry) }.is_ok() {
            if entry.cbData != u32::MAX {
                let ptr = entry.lpData.cast::<usize>().cast_const();
                heapset.push((heap, ptr, unsafe { ptr.add(entry.cbData as usize) }))
            }
        }
    }
    heapset.sort_by(|a, b| a.1.cmp(&b.1));
    #[allow(unused)]
    let print = |addr: *const usize| {
        check_global(addr, &map, &mut Vec::new(), 0, None, &heapset).print(0, 0, None);
    };
    #[allow(unused)]
    macro_rules! maybe_ptr_get {
        (true, $name:tt) => {
            unsafe { $name.cast::<*const usize>().as_ref().copied().unwrap() }
        };
        (false, $name:tt) => {
            $name.cast()
        };
    }
    macro_rules! print_unk {
        ($addr:expr, $t:tt, $is_ref:tt) => {
            Elem::from_addr(
                maybe_ptr_get!($is_ref, $addr),
                &map,
                &mut Vec::new(),
                stringify!($t),
                size_of::<$t>(),
                0,
                &heapset,
                false,
            )
            .print(0, 0, None);
        };
    }
    print(globals.entity_manager.cast());
    print(globals.global_stats.cast());
    print(globals.translation_manager.cast());
    print(globals.platform.cast());
    print(globals.death_match.cast());
    print(globals.debug_settings.cast());
    print_unk!(globals.component_type_manager, ComponentTypeManager, false);
    print_unk!(globals.inventory, Inventory, false);
    print_unk!(globals.game_global, GameGlobal, true);
    print_unk!(globals.component_manager, ComponentSystemManager, false);
    print_unk!(globals.world_state, Entity, true);
    Ok(())
}
fn check_global(
    reference: *const usize,
    map: &[(usize, String, usize)],
    addrs: &mut Vec<*const usize>,
    entry: usize,
    parent: Option<&str>,
    heaps: &[(HANDLE, *const usize, *const usize)],
) -> Elem {
    if let Some(n) = addrs.iter().position(|n| *n == reference) {
        return Elem::Recursive(addrs.len() - n, entry);
    }
    addrs.push(reference);
    unsafe {
        let Some(addr_size) = in_range(reference, heaps) else {
            return Elem::Usize(entry);
        };
        let Some(table) = reference.as_ref() else {
            return Elem::Usize(entry);
        };
        if let Ok(i) = map.binary_search_by(|r| r.0.cmp(table)) {
            let (_, name, size) = &map[i];
            if Some(name.as_ref()) == parent {
                Elem::VFTable(entry)
            } else {
                Elem::from_addr(
                    reference,
                    map,
                    addrs,
                    name,
                    if addr_size != usize::MAX {
                        addr_size
                    } else {
                        *size
                    },
                    entry,
                    heaps,
                    true,
                )
            }
        } else if let Some(size) = in_range(table, heaps)
            && size == 4
            && let Some(inner) = (table as *const usize)
                .cast::<*const usize>()
                .as_ref()
                .copied()
        {
            Elem::Ref(Box::new(check_global(
                inner, map, addrs, entry, None, heaps,
            )))
        } else if addr_size != usize::MAX && parent.is_none() {
            Elem::from_addr(reference, map, addrs, "Unk", addr_size, entry, heaps, false)
        } else {
            Elem::Usize(entry)
        }
    }
}
pub enum Elem {
    Ref(Box<Elem>),
    Struct(Struct, usize),
    VFTable(usize),
    Array(Box<Elem>, usize, usize),
    #[allow(unused)]
    Usize(usize),
    #[allow(unused)]
    Recursive(usize, usize),
}
impl PartialEq for Elem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Elem::Array(r1, n1, _), Elem::Array(r2, n2, _)) => r1 == r2 && n1 == n2,
            (Elem::Ref(r1), Elem::Ref(r2)) => r1 == r2,
            (Elem::Struct(r1, _), Elem::Struct(r2, _)) => r1 == r2,
            (Elem::VFTable(_), Elem::VFTable(_)) => true,
            (Elem::Usize(_), Elem::Usize(_)) => true,
            (Elem::Recursive(r1, _), Elem::Recursive(r2, _)) => r1 == r2,
            _ => false,
        }
    }
}
impl Elem {
    pub fn array_eq(&mut self, other: &Self) -> bool {
        match self {
            Elem::Array(e, _, _) => e.deref_mut() == other,
            e => e == other,
        }
    }
    pub fn entry(&self) -> usize {
        match self {
            Elem::Array(_, _, e) => *e,
            Elem::Ref(r) => r.entry(),
            Elem::Struct(_, e) => *e,
            Elem::VFTable(e) => *e,
            Elem::Usize(e) => *e,
            Elem::Recursive(_, e) => *e,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn from_addr(
        mut reference: *const usize,
        map: &[(usize, String, usize)],
        addrs: &mut Vec<*const usize>,
        name: &str,
        size: usize,
        entry: usize,
        heaps: &[(HANDLE, *const usize, *const usize)],
        skip: bool,
    ) -> Self {
        let mut s = Struct::new(name, size);
        if skip {
            s.fields.push(Elem::VFTable(0));
            reference = unsafe { reference.add(1) };
        }
        let mut i = 0;
        while i < if skip {
            (size / 4).saturating_sub(1)
        } else {
            size / 4
        } {
            let len = addrs.len();
            let e = check_global(
                unsafe { reference.add(i) },
                map,
                addrs,
                if skip { i + 1 } else { i },
                Some(name),
                heaps,
            );
            if let Elem::Struct(_, size) = &e {
                i += (size / 4).max(1);
            } else {
                i += 1
            }
            if let Some(last) = s.fields.last_mut()
                && last.array_eq(&e)
            {
                if let Elem::Array(_, n, _) = last {
                    *n += 1;
                } else {
                    let entry = last.entry();
                    s.fields.pop();
                    s.fields.push(Elem::Array(Box::new(e), 2, entry));
                };
            } else {
                s.fields.push(e);
            }
            while len < addrs.len() {
                addrs.pop();
            }
        }
        Elem::Struct(s, entry)
    }
}
#[derive(Default, PartialEq)]
pub struct Struct {
    name: String,
    size: usize,
    fields: Vec<Elem>,
}
impl Elem {
    fn print(&self, n: usize, count: usize, array: Option<(usize, usize)>) {
        match self {
            Elem::Ref(r) => r.print(n, count + 1, array),
            Elem::Array(r, k, e) => r.print(n, count + 1, Some((*k, *e))),
            Elem::Struct(s, e) => s.print(n, count, *e, array),
            Elem::Usize(e) => {
                noita_api::print!(
                    "{}[{}]{}usize{}",
                    "  ".repeat(n),
                    array.map(|a| a.1).unwrap_or(*e),
                    "&".repeat(count),
                    array.map(|a| format!("[{}]", a.0)).unwrap_or_default()
                )
            }
            Elem::Recursive(k, e) => {
                noita_api::print!(
                    "{}[{}]{}recursive<{k}>{}",
                    "  ".repeat(n),
                    array.map(|a| a.1).unwrap_or(*e),
                    "&".repeat(count),
                    array.map(|a| format!("[{}]", a.0)).unwrap_or_default()
                )
            }
            Elem::VFTable(e) => {
                noita_api::print!(
                    "{}[{}]{}VFTable{}",
                    "  ".repeat(n),
                    array.map(|a| a.1).unwrap_or(*e),
                    "&".repeat(count),
                    array.map(|a| format!("[{}]", a.0)).unwrap_or_default()
                )
            }
        }
    }
}
impl Struct {
    pub fn new(name: &str, size: usize) -> Self {
        Self {
            name: name.to_string(),
            size,
            ..Default::default()
        }
    }
    fn print(&self, n: usize, count: usize, entry: usize, array: Option<(usize, usize)>) {
        noita_api::print!(
            "{}[{}]{}{}<{}>{}",
            "  ".repeat(n),
            array.map(|a| a.1).unwrap_or(entry),
            "&".repeat(count),
            self.name,
            self.size,
            array.map(|a| format!("[{}]", a.0)).unwrap_or_default()
        );
        for f in self.fields.iter() {
            f.print(n + 1, 0, None);
        }
    }
}
fn in_range(
    reference: *const usize,
    heaps: &[(HANDLE, *const usize, *const usize)],
) -> Option<usize> {
    if reference.is_null() {
        return None;
    }
    let mut mbi = MEMORY_BASIC_INFORMATION::default();
    if unsafe {
        VirtualQuery(
            Some(reference.cast()),
            &mut mbi,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    } == 0
    {
        return None;
    }
    if mbi.State != MEM_COMMIT {
        return None;
    }
    let protect = mbi.Protect;
    if protect.contains(PAGE_NOACCESS) || protect.contains(PAGE_GUARD) || protect.0 == 0 {
        return None;
    }
    match heaps.binary_search_by(|r| {
        if reference < r.1 {
            std::cmp::Ordering::Greater
        } else if reference >= r.2 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }) {
        Ok(r) => {
            let r = &heaps[r];
            if r.1 < reference && r.2 > reference {
                Some(unsafe { HeapSize(r.0, HEAP_FLAGS::default(), reference.cast()) })
            } else {
                Some(usize::MAX)
            }
        }
        Err(_) => Some(usize::MAX),
    }
}
