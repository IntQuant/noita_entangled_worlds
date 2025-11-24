use crate::ExtState;
use noita_api::addr_grabber::Globals;
use noita_api::noita::types::*;
use std::collections::HashMap;
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
    let mut map = HashMap::new();
    for (addr, size) in size_map.into_iter() {
        let name = name_map.remove(&addr).unwrap();
        map.insert(addr, (name, size));
    }
    let globals = Globals::default();
    let print = |addr: *const usize| {
        check_global(addr, &map, &mut Vec::new(), 0, None).print(0, 0);
    };
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
            )
            .print(0, 0);
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
    let ptr = globals.game_global().m_cell_factory as *const CellFactory;
    print_unk!(ptr, CellFactory, false);
    let ptr = globals.game_global().m_textures as *const Textures;
    print_unk!(ptr, Textures, false);
    let ptr = globals.game_global().m_game_world as *const GameWorld;
    print_unk!(ptr, GameWorld, false);
    let ptr = unsafe {
        *(**globals.game_global)
            .m_grid_world
            .chunk_map
            .chunk_array
            .iter()
            .find(|a| !a.is_null())
            .unwrap()
    } as *const Chunk;
    print_unk!(ptr, Chunk, false);
    Ok(())
}
fn check_global(
    reference: *const usize,
    map: &HashMap<usize, (String, usize)>,
    addrs: &mut Vec<*const usize>,
    entry: usize,
    parent: Option<&str>,
) -> Elem {
    if let Some(n) = addrs.iter().position(|n| *n == reference) {
        return Elem::Recursive(addrs.len() - n, entry);
    }
    addrs.push(reference);
    unsafe {
        if !in_range(reference) {
            return Elem::Usize(entry);
        }
        let Some(table) = reference.as_ref() else {
            return Elem::Usize(entry);
        };
        if let Some((name, size)) = map.get(table) {
            if Some(name.as_ref()) == parent {
                Elem::VFTable(entry)
            } else {
                Elem::from_addr(reference, map, addrs, name, *size, entry)
            }
        } else if in_range(table)
            && let Some(inner) = (table as *const usize)
                .cast::<*const usize>()
                .as_ref()
                .copied()
        {
            Elem::Ref(Box::new(check_global(inner, map, addrs, entry, None)))
        } else {
            Elem::Usize(entry)
        }
    }
}
pub enum Elem {
    Ref(Box<Elem>),
    Struct(Struct, usize),
    VFTable(usize),
    #[allow(unused)]
    Usize(usize),
    #[allow(unused)]
    Recursive(usize, usize),
}
impl Elem {
    pub fn from_addr(
        reference: *const usize,
        map: &HashMap<usize, (String, usize)>,
        addrs: &mut Vec<*const usize>,
        name: &str,
        size: usize,
        entry: usize,
    ) -> Self {
        let mut s = Struct::new(name, size);
        let mut i = 0;
        while i < size / 4 {
            let len = addrs.len();
            let e = check_global(unsafe { reference.add(i) }, map, addrs, i, Some(name));
            if let Elem::Struct(_, size) = &e {
                i += (*size).max(1);
            } else {
                i += 1
            }
            s.fields.push(e);
            while len < addrs.len() {
                addrs.pop();
            }
        }
        Elem::Struct(s, entry)
    }
}
#[derive(Default)]
pub struct Struct {
    name: String,
    size: usize,
    fields: Vec<Elem>,
}
impl Elem {
    fn print(&self, n: usize, count: usize) {
        match self {
            Elem::Ref(r) => r.print(n, count + 1),
            Elem::Struct(s, e) => s.print(n, count, *e),
            Elem::Usize(_) => {
                //noita_api::print!("{}[{e}]{}usize", "  ".repeat(n), "&".repeat(count))
            }
            Elem::Recursive(_, _) => {
                //noita_api::print!("{}[{e}]{}recursive<{k}>", "  ".repeat(n), "&".repeat(count))
            }
            Elem::VFTable(e) => {
                noita_api::print!("{}[{e}]{}VFTable", "  ".repeat(n), "&".repeat(count))
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
    fn print(&self, n: usize, count: usize, entry: usize) {
        noita_api::print!(
            "{}[{entry}]{}{}<{}>",
            "  ".repeat(n),
            "&".repeat(count),
            self.name,
            self.size
        );
        for f in self.fields.iter() {
            f.print(n + 1, 0);
        }
    }
}
fn in_range(reference: *const usize) -> bool {
    if reference.is_null() {
        return false;
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
        return false;
    }
    if mbi.State != MEM_COMMIT {
        return false;
    }
    let protect = mbi.Protect;
    if protect == PAGE_NOACCESS || protect == PAGE_GUARD {
        return false;
    }
    //noita_api::print!("{:?} {:?}", reference, mbi);
    true
}
