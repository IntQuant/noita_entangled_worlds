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
        let size = size.parse::<usize>().unwrap_or(32);
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
    check_global(globals.entity_manager.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.world_seed.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.new_game_count.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.global_stats.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.game_global.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.entity_tag_manager.cast(), &map, &mut Vec::new(), 0);
    check_global(
        globals.component_type_manager.cast(),
        &map,
        &mut Vec::new(),
        0,
    );
    check_global(
        globals.component_tag_manager.cast(),
        &map,
        &mut Vec::new(),
        0,
    );
    check_global(globals.translation_manager.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.platform.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.filenames.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.inventory.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.mods.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.max_component.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.component_manager.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.world_state.cast(), &map, &mut Vec::new(), 0);
    check_global(
        globals.world_state_component.cast(),
        &map,
        &mut Vec::new(),
        0,
    );
    check_global(globals.event_manager.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.death_match.cast(), &map, &mut Vec::new(), 0);
    check_global(globals.debug_settings.cast(), &map, &mut Vec::new(), 0);
    Ok(())
}
fn check_global(
    reference: *const usize,
    map: &HashMap<usize, (String, usize)>,
    addrs: &mut Vec<*const usize>,
    count: usize,
) {
    if addrs.contains(&reference) {
        return;
    }
    addrs.push(reference);
    unsafe {
        if !in_range(reference) {
            return;
        }
        let Some(table) = reference.as_ref() else {
            return;
        };
        if let Some((name, size)) = map.get(table) {
            noita_api::print!("{}{name} {size}", " ".repeat(count));
            for i in 1..size / 4 {
                check_global(reference.add(i), map, addrs, count + 1);
            }
        } else if in_range(table)
            && let Some(inner) = (table as *const usize)
                .cast::<*const usize>()
                .as_ref()
                .copied()
        {
            check_global(inner, map, addrs, count);
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
