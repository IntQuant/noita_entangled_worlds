use libc::{SIGSTOP, kill, pid_t};
use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::os::unix::fs::FileExt;
fn main() {
    let map = get_map();
    let mut args = args();
    let Some(pid) = args.nth(1) else {
        println!("no pid");
        return;
    };
    let pid = pid.parse::<usize>().unwrap();
    unsafe {
        kill(pid_t::from(pid as i32), SIGSTOP);
    }
    let path = format!("/proc/{pid}/mem");
    let mem = File::open(path).unwrap();
    #[allow(unused)]
    let print = |addr: u32| {
        check_global(addr, &mem, &map, &mut Vec::new(), None).print(0, 0, 0, None);
    };
    #[allow(unused)]
    let print_sized = |addr: u32| {
        let elem = Elem::from_addr(
            addr,
            &mem,
            &map,
            &mut vec![addr],
            "Unk",
            get_size(&mem, addr).unwrap() as usize,
            false,
        );
        elem.print(0, 0, 0, None);
    };
    print_sized(read_byte(&mem, 0x0122374c).unwrap());
}
fn get_size(mem: &File, addr: u32) -> Option<u32> {
    if addr < 16
        || read_byte(mem, addr - 16)? != addr
        || read_byte(mem, addr - 12)? != 1
        || read_byte(mem, addr - 8)? != u32::MAX - 1
    {
        return None;
    }
    read_byte(mem, addr - 4)
}
#[allow(unused)]
fn read_unsized(mem: &File, addr: u32) -> Option<Vec<u32>> {
    let size = get_size(mem, addr)?;
    read(mem, addr, size as usize)
}
fn read_byte(mem: &File, addr: u32) -> Option<u32> {
    let mut buf = [0; 4];
    mem.read_exact_at(&mut buf, addr as u64).ok()?;
    Some(u32::from_le_bytes(buf))
}
#[allow(unused)]
fn read(mem: &File, addr: u32, size: usize) -> Option<Vec<u32>> {
    let size = (size + 0x11) & !0x11;
    let mut buf = vec![0; size];
    mem.read_exact_at(&mut buf, addr as u64).ok()?;
    Some(
        buf.chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    )
}
fn get_map() -> HashMap<u32, (String, usize)> {
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
        let addr = u32::from_str_radix(addr, 16).unwrap();
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
    map
}
#[derive(Default, PartialEq, Debug)]
pub struct Struct {
    name: String,
    size: usize,
    fields: Vec<Elem>,
}
#[derive(Debug)]
pub enum Elem {
    Ref(Box<Elem>, u32),
    Struct(Struct, u32),
    VFTable(u32),
    Array(Box<Elem>, Vec<u32>, usize),
    Usize(u32),
    Recursive(u32),
    TooLarge(u32),
    Failed(u32),
}
impl PartialEq for Elem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Elem::Array(r1, n1, _), Elem::Array(r2, n2, _)) => r1 == r2 && n1 == n2,
            (Elem::Ref(r1, _), Elem::Ref(r2, _)) => r1 == r2,
            (Elem::Struct(r1, _), Elem::Struct(r2, _)) => r1 == r2,
            (Elem::VFTable(_), Elem::VFTable(_)) => true,
            (Elem::Usize(_), Elem::Usize(_)) => true,
            (Elem::Recursive(r1), Elem::Recursive(r2)) => r1 == r2,
            (Elem::Failed(_), Elem::Failed(_)) => true,
            (Elem::TooLarge(_), Elem::TooLarge(_)) => true,
            _ => false,
        }
    }
}
impl Elem {
    pub fn array_eq(&self, other: &Self) -> bool {
        match self {
            Elem::Array(e, _, _) => e.as_ref() == other,
            e => e == other,
        }
    }
    pub fn size(&self) -> usize {
        match self {
            Elem::Ref(_, _)
            | Elem::VFTable(_)
            | Elem::Usize(_)
            | Elem::Recursive(_)
            | Elem::Failed(_)
            | Elem::TooLarge(_) => 4,
            Elem::Struct(e, _) => e.size,
            Elem::Array(e, _, n) => e.size() * n,
        }
    }
    pub fn value(&self) -> Option<u32> {
        Some(match self {
            Elem::Ref(_, v) => *v,
            Elem::Struct(_, v) => *v,
            Elem::VFTable(v) => *v,
            Elem::Array(_, _, _) => return None,
            Elem::Usize(v) => *v,
            Elem::Recursive(v) => *v,
            Elem::TooLarge(v) => *v,
            Elem::Failed(v) => *v,
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub fn from_addr(
        mut reference: u32,
        mem: &File,
        map: &HashMap<u32, (String, usize)>,
        addrs: &mut Vec<u32>,
        name: &str,
        size: usize,
        skip: bool,
    ) -> Self {
        if addrs.len() > 6 {
            return Elem::TooLarge(reference);
        }
        let mut s = Struct::new(name, size);
        let ptr = reference;
        if skip {
            s.fields
                .push(Elem::VFTable(read_byte(mem, reference).unwrap()));
            reference += 4;
        }
        let mut i = 0;
        while i < if skip {
            (size / 4).saturating_sub(1)
        } else {
            size / 4
        } {
            let len = addrs.len();
            let e = check_global(reference + 4 * i as u32, mem, map, addrs, Some(name));
            i += (e.size() / 4).max(1);
            if let Some(last) = s.fields.last_mut()
                && last.array_eq(&e)
            {
                if let Elem::Array(_, vec, n) = last {
                    if let Some(v) = e.value() {
                        vec.push(v);
                    }
                    *n += 1;
                } else {
                    let mut vec = Vec::new();
                    if let Some(v) = last.value() {
                        vec.push(v);
                    }
                    if let Some(v) = e.value() {
                        vec.push(v);
                    }
                    *last = Elem::Array(Box::new(e), vec, 2);
                };
            } else {
                s.fields.push(e);
            }
            while len < addrs.len() {
                addrs.pop();
            }
        }
        Elem::Struct(s, ptr)
    }
}
fn check_global(
    reference: u32,
    mem: &File,
    map: &HashMap<u32, (String, usize)>,
    addrs: &mut Vec<u32>,
    parent: Option<&str>,
) -> Elem {
    let Some(table) = read_byte(mem, reference) else {
        return Elem::Usize(reference);
    };
    if addrs.contains(&table) {
        return Elem::Recursive(table);
    }
    addrs.push(reference);
    let addr_size = get_size(mem, reference).unwrap_or(u32::MAX);
    if let Some((name, size)) = map.get(&table) {
        if Some(name.as_ref()) == parent {
            Elem::VFTable(table)
        } else {
            Elem::from_addr(
                reference,
                mem,
                map,
                addrs,
                name,
                if addr_size != u32::MAX {
                    addr_size as usize
                } else {
                    *size
                },
                true,
            )
        }
    } else if let Some(size) = get_size(mem, table) {
        Elem::Ref(
            if size == 4
                || read_byte(mem, table)
                    .map(|a| map.get(&a).is_some())
                    .unwrap_or_default()
            {
                Box::new(check_global(table, mem, map, addrs, None))
            } else {
                Box::new(Elem::from_addr(
                    table,
                    mem,
                    map,
                    addrs,
                    "Unk",
                    size as usize,
                    false,
                ))
            },
            table,
        )
    } else {
        Elem::Usize(table)
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
    fn print(&self, n: usize, count: usize, entry: usize, v: u32, array: Option<(usize, &[u32])>) {
        println!(
            "{}[{entry}]{}{}<{}>{}({})",
            "  ".repeat(n),
            "&".repeat(count),
            self.name,
            self.size,
            array.map(|(a, _)| format!("[{a}]")).unwrap_or_default(),
            array
                .map(|(_, b)| b
                    .iter()
                    .map(|v| format!("0x{v:x}"))
                    .collect::<Vec<String>>()
                    .join(","))
                .unwrap_or(format!("0x{v:x}")),
        );
        let mut e = 0;
        for f in self.fields.iter() {
            f.print(n + 1, 0, e / 4, None);
            e += f.size();
        }
    }
}
impl Elem {
    fn print(&self, n: usize, count: usize, e: usize, array: Option<(usize, &[u32])>) {
        match self {
            Elem::Ref(r, _) => r.print(n, count + 1, e, array),
            Elem::Array(r, v, k) => r.print(n, count, e, Some((*k, v))),
            Elem::Struct(s, v) => s.print(n, count, e, *v, array),
            Elem::Usize(v) => {
                println!(
                    "{}[{e}]{}usize{}({})",
                    "  ".repeat(n),
                    "&".repeat(count),
                    array.map(|(a, _)| format!("[{a}]")).unwrap_or_default(),
                    array
                        .map(|(_, b)| b
                            .iter()
                            .map(|v| format!("0x{v:x}"))
                            .collect::<Vec<String>>()
                            .join(","))
                        .unwrap_or(format!("0x{v:x}")),
                )
            }
            Elem::Recursive(v) => {
                println!(
                    "{}[{e}]{}recursive{}({})",
                    "  ".repeat(n),
                    "&".repeat(count),
                    array.map(|(a, _)| format!("[{a}]")).unwrap_or_default(),
                    array
                        .map(|(_, b)| b
                            .iter()
                            .map(|v| format!("0x{v:x}"))
                            .collect::<Vec<String>>()
                            .join(","))
                        .unwrap_or(format!("0x{v:x}")),
                )
            }
            Elem::VFTable(v) => {
                println!(
                    "{}[{e}]{}VFTable{}({})",
                    "  ".repeat(n),
                    "&".repeat(count),
                    array.map(|(a, _)| format!("[{a}]")).unwrap_or_default(),
                    array
                        .map(|(_, b)| b
                            .iter()
                            .map(|v| format!("0x{v:x}"))
                            .collect::<Vec<String>>()
                            .join(","))
                        .unwrap_or(format!("0x{v:x}")),
                )
            }
            Elem::TooLarge(v) => {
                println!(
                    "{}[{e}]{}TooLarge{}({})",
                    "  ".repeat(n),
                    "&".repeat(count),
                    array.map(|(a, _)| format!("[{a}]")).unwrap_or_default(),
                    array
                        .map(|(_, b)| b
                            .iter()
                            .map(|v| format!("0x{v:x}"))
                            .collect::<Vec<String>>()
                            .join(","))
                        .unwrap_or(format!("0x{v:x}")),
                )
            }
            Elem::Failed(v) => {
                println!(
                    "{}[{e}]{}Failed{}({})",
                    "  ".repeat(n),
                    "&".repeat(count),
                    array.map(|(a, _)| format!("[{a}]")).unwrap_or_default(),
                    array
                        .map(|(_, b)| b
                            .iter()
                            .map(|v| format!("0x{v:x}"))
                            .collect::<Vec<String>>()
                            .join(","))
                        .unwrap_or(format!("0x{v:x}")),
                )
            }
        }
    }
}
