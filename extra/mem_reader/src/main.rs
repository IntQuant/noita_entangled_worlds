use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::os::unix::fs::FileExt;
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
//use noita_api::types::*;
fn main() {
    let map = get_map();
    let mut args = args();
    let Some(pid) = args.nth(1) else {
        2!("no pid");
        return;
    };
    let pid = pid.parse::<usize>().unwrap();
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
            &mut Vec::new(),
            "Unk",
            get_size(&mem, addr).unwrap() as usize,
            false,
        );
        elem.print(0, 0, 0, None);
    };
    print_sized(read_byte(&mem, 0x0122374c).unwrap());
}
#[derive(Default, PartialEq, Debug)]
pub struct Struct {
    name: String,
    size: usize,
    fields: Vec<Elem>,
}
#[derive(Debug, PartialEq)]
pub enum Elem {
    Ref(Box<Elem>),
    Struct(Struct),
    VFTable,
    Array(Box<Elem>, usize),
    #[allow(unused)]
    Usize,
    #[allow(unused)]
    Recursive(usize),
}
impl Elem {
    pub fn array_eq(&self, other: &Self) -> bool {
        match self {
            Elem::Array(e, _) => e.as_ref() == other,
            e => e == other,
        }
    }
    pub fn size(&self) -> usize {
        match self {
            Elem::Ref(_) | Elem::VFTable | Elem::Usize | Elem::Recursive(_) => 4,
            Elem::Struct(e) => e.size,
            Elem::Array(e, n) => e.size() * n,
        }
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
        let mut s = Struct::new(name, size);
        if skip {
            s.fields.push(Elem::VFTable);
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
            i += e.size() / 4;
            if let Some(last) = s.fields.last_mut()
                && last.array_eq(&e)
            {
                if let Elem::Array(_, n) = last {
                    *n += 1;
                } else {
                    s.fields.pop();
                    s.fields.push(Elem::Array(Box::new(e), 2));
                };
            } else {
                s.fields.push(e);
            }
            while len < addrs.len() {
                addrs.pop();
            }
        }
        Elem::Struct(s)
    }
}
fn check_global(
    reference: u32,
    mem: &File,
    map: &HashMap<u32, (String, usize)>,
    addrs: &mut Vec<u32>,
    parent: Option<&str>,
) -> Elem {
    if let Some(n) = addrs.iter().position(|n| *n == reference) {
        return Elem::Recursive(addrs.len() - n);
    }
    addrs.push(reference);
    let addr_size = get_size(mem, reference).unwrap_or(u32::MAX);
    let Some(table) = read_byte(mem, reference) else {
        return Elem::Usize;
    };
    if let Some((name, size)) = map.get(&table) {
        if Some(name.as_ref()) == parent {
            Elem::VFTable
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
    } else if let Some(size) = get_size(mem, table)
        && let Some(inner) = read_byte(mem, table)
    {
        Elem::Ref(if size == 4 {
            Box::new(check_global(inner, mem, map, addrs, None))
        } else {
            Box::new(Elem::from_addr(
                inner,
                mem,
                map,
                addrs,
                "Unk",
                size as usize,
                false,
            ))
        })
    } else {
        Elem::Usize
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
    fn print(&self, n: usize, count: usize, entry: usize, array: Option<usize>) {
        println!(
            "{}[{entry}]{}{}<{}>{}",
            "  ".repeat(n),
            "&".repeat(count),
            self.name,
            self.size,
            array.map(|a| format!("[{a}]")).unwrap_or_default()
        );
        let mut e = 0;
        for f in self.fields.iter() {
            f.print(n + 1, 0, e / 4, None);
            e += f.size();
        }
    }
}
impl Elem {
    fn print(&self, n: usize, count: usize, e: usize, array: Option<usize>) {
        match self {
            Elem::Ref(r) => r.print(n, count + 1, e, array),
            Elem::Array(r, k) => r.print(n, count, e, Some(*k)),
            Elem::Struct(s) => s.print(n, count, e, array),
            Elem::Usize => {
                println!(
                    "{}[{e}]{}usize{}",
                    "  ".repeat(n),
                    "&".repeat(count),
                    array.map(|a| format!("[{a}]")).unwrap_or_default()
                )
            }
            Elem::Recursive(k) => {
                println!(
                    "{}[{e}]{}recursive<{k}>{}",
                    "  ".repeat(n),
                    "&".repeat(count),
                    array.map(|a| format!("[{a}]")).unwrap_or_default()
                )
            }
            Elem::VFTable => {
                println!(
                    "{}[{e}]{}VFTable{}",
                    "  ".repeat(n),
                    "&".repeat(count),
                    array.map(|a| format!("[{a}]")).unwrap_or_default()
                )
            }
        }
    }
}
