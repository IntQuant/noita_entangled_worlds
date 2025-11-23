use crate::ExtState;
use bimap::BiHashMap;
use noita_api::addr_grabber::Globals;
use std::collections::HashMap;
pub fn check_globals(_: &mut ExtState) -> eyre::Result<()> {
    let vftables = include_str!("../vftables.txt");
    let mut name_map = BiHashMap::new();
    let mut size_map = HashMap::new();
    for line in vftables.lines() {
        let mut split = line.split(' ');
        let Some(name) = split.next() else {
            continue;
        };
        let mut name = name.to_string();
        let Some(addr) = split.next() else {
            continue;
        };
        let addr = usize::from_str_radix(addr, 16)?;
        let Some(size) = split.next() else {
            continue;
        };
        let size = size.parse::<usize>().unwrap_or(0);
        let mut i = 1;
        while name_map.contains_right(&name) {
            if i != 1 {
                name.pop();
            }
            name = format!("{name}{i}");
            i += 1;
        }
        name_map.insert(addr, name);
        size_map.insert(addr, size);
    }
    let mut map = HashMap::new();
    for (addr, size) in size_map.into_iter() {
        let name = name_map.remove_by_left(&addr).unwrap().1;
        map.insert(addr, (name, size));
    }
    let globals = Globals::default();
    /*noita_api::print!(
        "\n{}",
        check_global(globals.entity_manager.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.world_seed.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.new_game_count.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.global_stats.cast(), &map, &mut Vec::new())
    );*/
    Elem::from_addr(
        unsafe {
            globals
                .game_global
                .cast::<*const usize>()
                .as_ref()
                .copied()
                .unwrap()
        },
        &map,
        &mut Vec::new(),
        "GameGlobal",
        416,
        0,
    )
    .print(0, 0);
    /*noita_api::print!(
        "\n{}",
        check_global(globals.entity_tag_manager.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.component_type_manager.cast(), &map, &mut Vec::new(),)
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.component_tag_manager.cast(), &map, &mut Vec::new(),)
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.translation_manager.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.platform.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.filenames.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.inventory.cast(), &map, &mut Vec::new())
    );
    noita_api::print!("\n{}", check_global(globals.mods.cast(), &map, &mut Vec::new()));
    noita_api::print!(
        "\n{}",
        check_global(globals.max_component.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.component_manager.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.world_state.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.world_state_component.cast(), &map, &mut Vec::new(),)
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.event_manager.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.death_match.cast(), &map, &mut Vec::new())
    );
    noita_api::print!(
        "\n{}",
        check_global(globals.debug_settings.cast(), &map, &mut Vec::new())
    );*/
    Ok(())
}
fn check_global(
    reference: *const usize,
    map: &HashMap<usize, (String, usize)>,
    addrs: &mut Vec<*const usize>,
    entry: usize,
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
            Elem::from_addr(reference, map, addrs, name, *size, entry)
        } else if in_range(table)
            && let Some(inner) = (table as *const usize)
                .cast::<*const usize>()
                .as_ref()
                .copied()
        {
            Elem::Ref(Box::new(check_global(inner, map, addrs, entry)))
        } else {
            Elem::Usize(entry)
        }
    }
}
pub enum Elem {
    Ref(Box<Elem>),
    Struct(Struct, usize),
    #[allow(unused)]
    Usize(usize),
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
        for i in 1..size / 4 {
            let len = addrs.len();
            s.fields
                .push(check_global(unsafe { reference.add(i) }, map, addrs, i));
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
                //noita_api::print!("{}[{e}]{}usize", " ".repeat(n), "&".repeat(count))
            }
            Elem::Recursive(k, e) => {
                noita_api::print!("{}[{e}]{}recursive<{k}>", " ".repeat(n), "&".repeat(count))
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
        if self.fields.is_empty() {
            noita_api::print!(
                "{}[{entry}]{}{}<{}>",
                " ".repeat(n),
                "&".repeat(count),
                self.name,
                self.size
            )
        } else {
            noita_api::print!(
                "{}[{entry}]{}{}<{}>(",
                " ".repeat(n),
                "&".repeat(count),
                self.name,
                self.size,
            )
        }
        for f in self.fields.iter() {
            f.print(n + 1, 0);
        }
    }
}
use windows::Win32::System::Memory::*;
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
    true
}
