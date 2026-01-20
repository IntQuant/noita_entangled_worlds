use eframe::{Frame, NativeOptions};
use egui::{CentralPanel, Color32, ComboBox, Context, Ui, UiBuilder};
use libc::{SIGCONT, SIGSTOP, kill, pid_t};
use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::mem;
use std::os::unix::fs::FileExt;
fn main() {
    let map = get_map();
    let mut args = args();
    let Some(pid) = args.nth(1) else {
        println!("no pid");
        return;
    };
    let pid = pid.parse::<usize>().unwrap();
    let reader = Reader::new(pid);
    reader.stop();
    let reference = 0x0122374c;
    let elem = Elem::check_global(reference, &reader, &map, &mut vec![reference], None);
    eframe::run_native(
        &reference.to_string(),
        NativeOptions::default(),
        Box::new(|_| {
            Ok(Box::new(App {
                reader,
                map,
                elem,
                reference: format!("0x{reference:08x}"),
                size: "-1".to_string(),
                data: None,
                display_type: DisplayType::Hex,
            }))
        }),
    )
    .unwrap();
}
impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let (top, rect) = rect.split_top_bottom_at_y(rect.height() * 0.035);
            ui.scope_builder(
                UiBuilder {
                    max_rect: Some(top),
                    ..Default::default()
                },
                |ui| {
                    ui.horizontal(|ui| {
                        if (ui
                            .add_sized(
                                [200.0, 20.0],
                                egui::TextEdit::singleline(&mut self.reference),
                            )
                            .changed()
                            || ui
                                .add_sized(
                                    [200.0, 20.0],
                                    egui::TextEdit::singleline(&mut self.size),
                                )
                                .changed()
                            || ui.button("refresh").clicked())
                            && self.reference.starts_with("0x")
                            && let Ok(addr) = u32::from_str_radix(&self.reference[2..], 16)
                        {
                            if let Ok(size) = self.size.parse::<usize>() {
                                self.elem = Elem::from_addr(addr, "Unk", 4 * size, false);
                            } else {
                                self.elem = Elem::check_global(
                                    addr,
                                    &self.reader,
                                    &self.map,
                                    &mut vec![addr],
                                    None,
                                );
                            }
                        }
                        ComboBox::from_label("")
                            .selected_text(format!("{:?}", self.display_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.display_type,
                                    DisplayType::Hex,
                                    "Hex",
                                );
                                ui.selectable_value(
                                    &mut self.display_type,
                                    DisplayType::Bin,
                                    "Bin",
                                );
                                ui.selectable_value(
                                    &mut self.display_type,
                                    DisplayType::Num,
                                    "Num",
                                );
                                ui.selectable_value(
                                    &mut self.display_type,
                                    DisplayType::SignedNum,
                                    "SignedNum",
                                );
                                ui.selectable_value(
                                    &mut self.display_type,
                                    DisplayType::Float,
                                    "Float",
                                );
                                ui.selectable_value(
                                    &mut self.display_type,
                                    DisplayType::Str,
                                    "Str",
                                );
                                ui.selectable_value(
                                    &mut self.display_type,
                                    DisplayType::None,
                                    "None",
                                );
                            });
                    });
                },
            );
            let (settings_rect, right) = rect.split_left_right_at_fraction(0.5);
            ui.scope_builder(
                UiBuilder {
                    max_rect: Some(settings_rect),
                    ..Default::default()
                },
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("1")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            if let Some(d) = self.elem.show(
                                ui,
                                &self.reader,
                                &self.map,
                                &mut self.elem.reference().map(|a| vec![a]).unwrap_or_default(),
                                0,
                                "f0",
                                Vec::new(),
                                self.display_type,
                            ) {
                                self.data = Some(d);
                            }
                        });
                },
            );
            ui.scope_builder(
                UiBuilder {
                    max_rect: Some(right),
                    ..Default::default()
                },
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("2")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            if let Some(data) = &self.data {
                                match data {
                                    Data::Values(refs, n) => {
                                        if !refs.is_empty() {
                                            ui.label(
                                                refs.iter()
                                                    .map(|(a, b)| format!("0x{a:08x}->0x{b:08x}"))
                                                    .collect::<Vec<String>>()
                                                    .join(","),
                                            );
                                            ui.separator();
                                        }
                                        ui.label(format!("0x{n:08x}"));
                                        ui.label(format!("{n:032b}"));
                                        ui.label(format!("{n:010}"));
                                        if n.cast_signed() < 0 {
                                            ui.label(format!("{:010}", n.cast_signed()));
                                        }
                                        ui.label(f32::from_bits(*n).to_string());
                                        if let Ok(v) = String::from_utf8(n.to_le_bytes().to_vec()) {
                                            ui.label(v);
                                        }
                                    }
                                    Data::Struct(refs) => {
                                        if !refs.is_empty() {
                                            ui.label(
                                                refs.iter()
                                                    .map(|(a, b)| format!("0x{a:08x}->0x{b:08x}"))
                                                    .collect::<Vec<String>>()
                                                    .join(","),
                                            );
                                            ui.separator();
                                        }
                                    }
                                }
                            }
                        });
                },
            );
        });
    }
}
pub struct App {
    reader: Reader,
    map: HashMap<u32, (String, usize)>,
    elem: Elem,
    reference: String,
    size: String,
    data: Option<Data>,
    display_type: DisplayType,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DisplayType {
    Hex,
    Bin,
    Num,
    SignedNum,
    Float,
    Str,
    None,
}
pub enum Data {
    Struct(Vec<(u32, u32)>),
    Values(Vec<(u32, u32)>, u32),
}
pub struct Reader {
    mem: File,
    pid: usize,
}
impl Reader {
    fn new(pid: usize) -> Self {
        let path = format!("/proc/{pid}/mem");
        Reader {
            mem: File::open(path).unwrap(),
            pid,
        }
    }
    fn stop(&self) {
        unsafe {
            kill(pid_t::from(self.pid as i32), SIGSTOP);
        }
    }
    #[allow(dead_code)]
    fn cont(&self) {
        unsafe {
            kill(pid_t::from(self.pid as i32), SIGCONT);
        }
    }
    fn get_size(&self, addr: u32) -> Option<u32> {
        if addr < 16
            || self.read_byte(addr - 16)? != addr
            || self.read_byte(addr - 12)? != 1
            || self.read_byte(addr - 8)? != u32::MAX - 1
        {
            return None;
        }
        self.read_byte(addr - 4)
    }
    #[allow(unused)]
    fn read_unsized(&self, addr: u32) -> Option<Vec<u32>> {
        let size = self.get_size(addr)?;
        self.read(addr, size as usize)
    }
    fn read_byte(&self, addr: u32) -> Option<u32> {
        let mut buf = [0; 4];
        self.mem.read_exact_at(&mut buf, addr as u64).ok()?;
        Some(u32::from_le_bytes(buf))
    }
    #[allow(unused)]
    fn read(&self, addr: u32, size: usize) -> Option<Vec<u32>> {
        let size = (size + 0x11) & !0x11;
        let mut buf = vec![0; size];
        self.mem.read_exact_at(&mut buf, addr as u64).ok()?;
        Some(
            buf.chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect(),
        )
    }
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
#[derive(Debug, Clone)]
pub struct Struct {
    name: String,
    size: usize,
    fields: Option<Vec<(String, Elem)>>,
    reference: u32,
    skip: bool,
}
impl PartialEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.size == other.size && self.fields == other.fields
    }
}
#[derive(Debug, Clone)]
pub enum Elem {
    Ref(Box<Elem>, u32, u32),
    Struct(Struct, u32, usize),
    VFTable(String, u32),
    Array(Vec<Elem>, usize),
    Usize(u32),
    Recursive(u32),
    TooLarge(u32),
    Failed(u32),
    Null,
}
impl PartialEq for Elem {
    fn eq(&self, other: &Self) -> bool {
        if (self.null() && other.is_ref()) || (self.is_ref() && other.null()) {
            return true;
        }
        match (self, other) {
            (Elem::Array(r1, _), Elem::Array(r2, _)) => r1 == r2,
            (Elem::Ref(r1, _, _), Elem::Ref(r2, _, _)) => r1 == r2,
            (Elem::Struct(r1, _, _), Elem::Struct(r2, _, _)) => r1 == r2,
            (Elem::VFTable(_, _), Elem::VFTable(_, _)) => true,
            (Elem::Usize(_), Elem::Usize(_)) => true,
            (Elem::Recursive(r1), Elem::Recursive(r2)) => r1 == r2,
            (Elem::Failed(_), Elem::Failed(_)) => true,
            (Elem::TooLarge(_), Elem::TooLarge(_)) => true,
            _ => false,
        }
    }
}
const LIM: usize = 512;
impl Elem {
    pub fn is_ref(&self) -> bool {
        !matches!(self, Elem::Struct(_, _, _))
    }
    pub fn reference(&self) -> Option<u32> {
        match self {
            Elem::Ref(_, r, _) => Some(*r),
            Elem::Struct(_, r, _) => Some(*r),
            _ => None,
        }
    }
    pub fn data(&self) -> u32 {
        match self {
            Elem::Ref(_, v, _) => *v,
            Elem::Struct(_, v, _) => *v,
            Elem::VFTable(_, v) => *v,
            Elem::Array(_, _) => 0,
            Elem::Usize(v) => *v,
            Elem::Recursive(v) => *v,
            Elem::TooLarge(v) => *v,
            Elem::Failed(v) => *v,
            Elem::Null => 0,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ui: &mut Ui,
        reader: &Reader,
        map: &HashMap<u32, (String, usize)>,
        addrs: &mut Vec<u32>,
        count: usize,
        entry: &str,
        mut refs: Vec<(u32, u32)>,
        display_type: DisplayType,
    ) -> Option<Data> {
        match self {
            Elem::Struct(s, r, lim) => {
                addrs.push(*r);
                let text = s.print(count, entry, *r, Vec::new());
                let resp = egui::CollapsingHeader::new(&text)
                    .id_salt((text, addrs.len(), addrs.last().copied().unwrap_or_default()))
                    .show(ui, |ui| {
                        let mut ret = None;
                        for (i, (n, f)) in s.fields(reader, map, addrs).iter_mut().enumerate() {
                            if i > *lim {
                                if ui.button("...").clicked() {
                                    *lim *= 2;
                                }
                                break;
                            }
                            let len = addrs.len();
                            ret = ret.or(f.show(
                                ui,
                                reader,
                                map,
                                addrs,
                                0,
                                n,
                                Vec::new(),
                                display_type,
                            ));
                            while len < addrs.len() {
                                addrs.pop();
                            }
                        }
                        ret
                    });
                if resp.header_response.hovered() {
                    return Some(Data::Struct(mem::take(&mut refs)));
                }
                return resp.body_returned.flatten();
            }
            Elem::Ref(s, r, v) => {
                addrs.push(*r);
                refs.push((*r, *v));
                return s.show(ui, reader, map, addrs, count + 1, entry, refs, display_type);
            }
            Elem::Array(v, lim) => {
                let text = v.iter().find(|a| !a.null()).unwrap_or(&Elem::Null).print(
                    count,
                    entry,
                    vec![v.len()],
                    display_type,
                );
                let resp = egui::CollapsingHeader::new(&text)
                    .id_salt((text, addrs.len(), addrs.last().copied().unwrap_or_default()))
                    .show(ui, |ui| {
                        let mut ret = None;
                        for (i, n) in v.iter_mut().enumerate() {
                            if i > *lim {
                                if ui.button("...").clicked() {
                                    *lim *= 2;
                                }
                                break;
                            }
                            ret = ret.or(n.show(
                                ui,
                                reader,
                                map,
                                addrs,
                                0,
                                &i.to_string(),
                                Vec::new(),
                                display_type,
                            ));
                        }
                        ret
                    });
                return resp.body_returned.flatten();
            }
            s => {
                if ui
                    .colored_label(
                        Color32::WHITE,
                        format!("      {}", s.print(count, entry, Vec::new(), display_type)),
                    )
                    .hovered()
                {
                    return Some(Data::Values(mem::take(&mut refs), s.data()));
                }
            }
        }
        None
    }
    pub fn null(&self) -> bool {
        match self {
            Elem::Null => true,
            Elem::Ref(r, _, _) => r.null(),
            Elem::Array(v, _) => v.iter().all(|a| a.null()),
            _ => false,
        }
    }
    pub fn array_eq(&self, other: &Self) -> bool {
        if (self.null() && other.is_ref()) || (self.is_ref() && other.null()) {
            return true;
        }
        match self {
            Elem::Array(e, _) => e.iter().find(|a| !a.null()).unwrap_or(&Elem::Null) == other,
            Elem::Struct(_, _, _) => false,
            e => e == other,
        }
    }
    pub fn size(&self) -> usize {
        match self {
            Elem::Ref(_, _, _)
            | Elem::VFTable(_, _)
            | Elem::Usize(_)
            | Elem::Recursive(_)
            | Elem::Failed(_)
            | Elem::Null
            | Elem::TooLarge(_) => 4,
            Elem::Struct(e, _, _) => e.size,
            Elem::Array(e, _) => e.iter().map(|a| a.size()).sum(),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn from_addr(reference: u32, name: &str, size: usize, skip: bool) -> Self {
        let s = Struct {
            name: name.to_string(),
            size,
            fields: None,
            reference,
            skip,
        };
        Elem::Struct(s, reference, LIM)
    }
    fn check_global(
        reference: u32,
        mem: &Reader,
        map: &HashMap<u32, (String, usize)>,
        addrs: &mut Vec<u32>,
        parent: Option<&str>,
    ) -> Elem {
        let Some(table) = mem.read_byte(reference) else {
            return Elem::Usize(reference);
        };
        if table == 0 {
            return Elem::Null;
        }
        if addrs.contains(&table) {
            return Elem::Recursive(table);
        }
        addrs.push(reference);
        let addr_size = mem.get_size(reference).unwrap_or(u32::MAX);
        if let Some((name, size)) = map.get(&table) {
            if Some(name.as_ref()) == parent {
                Elem::VFTable(name.to_string(), table)
            } else {
                Elem::from_addr(
                    reference,
                    name,
                    if addr_size != u32::MAX {
                        addr_size as usize
                    } else {
                        *size
                    },
                    true,
                )
            }
        } else if let Some(size) = mem.get_size(table) {
            Elem::Ref(
                if size == 4
                    || mem
                        .read_byte(table)
                        .map(|a| map.get(&a).is_some())
                        .unwrap_or_default()
                {
                    Box::new(Elem::check_global(table, mem, map, addrs, None))
                } else {
                    Box::new(Elem::from_addr(table, "Unk", size as usize, false))
                },
                reference,
                table,
            )
        } else {
            Elem::Usize(table)
        }
    }
}
impl Struct {
    fn fields(
        &mut self,
        mem: &Reader,
        map: &HashMap<u32, (String, usize)>,
        addrs: &mut Vec<u32>,
    ) -> &mut Vec<(String, Elem)> {
        if self.fields.is_some() {
            self.fields.as_mut().unwrap()
        } else {
            self.fields = Some(self.get_fields(mem, map, addrs));
            self.fields.as_mut().unwrap()
        }
    }
    fn get_fields(
        &self,
        mem: &Reader,
        map: &HashMap<u32, (String, usize)>,
        addrs: &mut Vec<u32>,
    ) -> Vec<(String, Elem)> {
        let mut fields: Vec<(String, Elem)> = Vec::with_capacity(self.size / 4);
        let mut i = 0;
        if self.skip {
            fields.push((
                "f0".to_string(),
                Elem::VFTable(self.name.clone(), mem.read_byte(self.reference).unwrap()),
            ));
            i += 1;
        }
        while i < self.size / 4 {
            let len = addrs.len();
            let e = Elem::check_global(
                self.reference + 4 * i as u32,
                mem,
                map,
                addrs,
                Some(&self.name),
            );
            let size = (e.size() / 4).max(1);
            fields.push((format!("f{i}"), e));
            i += size;
            while len < addrs.len() {
                addrs.pop();
            }
        }
        let mut i = fields.len();
        let mut not_null = false;
        let mut v: Option<usize> = None;
        let mut s = fields.len();
        let mut null = None;
        while i > 0 {
            i -= 1;
            if fields[i].1.null() {
                continue;
            }
            if let Some(k) = v {
                if fields[i].1 == fields[k].1 {
                    not_null = true;
                } else if not_null {
                    let f = fields[i + 1].0.clone();
                    let arr = fields.drain(i + 1..s).map(|(_, b)| b).collect();
                    fields.insert(i + 1, (f, Elem::Array(arr, LIM)));
                    s = i;
                    not_null = false;
                    v = None;
                    null = None
                } else {
                    if let Some((i, v)) = mem::take(&mut null) {
                        let (f, _): &(String, _) = &fields[i];
                        let f = f.clone();
                        let arr: Vec<Elem> = fields.drain(i..v).map(|(_, b)| b).collect();
                        s -= arr.len();
                        fields.insert(i, (f, Elem::Array(arr, LIM)));
                    }
                    if s - k > 2 {
                        let f = fields[k + 1].0.clone();
                        let arr = fields.drain(k + 1..s).map(|(_, b)| b).collect();
                        fields.insert(k + 1, (f, Elem::Array(arr, LIM)));
                    }
                    if k - i > 2 {
                        null = Some((i + 1, k));
                    }
                    s = k;
                    v = Some(i);
                }
            } else {
                v = Some(i);
                null = None
            }
        }
        if not_null {
            if s == fields.len() {
                return if fields.len() == 512 * 512 {
                    let mut f = Vec::with_capacity(512);
                    for _ in 0..512 {
                        let mut fi = Vec::with_capacity(512);
                        fi.extend(fields.drain(0..512).map(|(_, a)| a));
                        f.push(Elem::Array(fi, LIM))
                    }
                    vec![("f0".to_string(), Elem::Array(f, LIM))]
                } else {
                    vec![(
                        "f0".to_string(),
                        Elem::Array(fields.into_iter().map(|(_, b)| b).collect(), LIM),
                    )]
                };
            }
            let v = fields.drain(0..s).map(|(_, b)| b).collect();
            fields.insert(0, ("f0".to_string(), Elem::Array(v, LIM)))
        }
        fields.shrink_to_fit();
        fields
    }
    #[allow(clippy::too_many_arguments)]
    fn print(&self, count: usize, entry: &str, v: u32, array: Vec<usize>) -> String {
        format!(
            "{entry}: {}{}{}<{}>{} {}",
            "[".repeat(array.len()),
            "&".repeat(count),
            self.name,
            display_size(self.size / 4),
            array
                .iter()
                .map(|a| format!("; {}]", display_size(*a)))
                .collect::<Vec<String>>()
                .join(""),
            if array.is_empty() {
                DisplayType::Hex.print(&v)
            } else {
                String::new()
            }
        )
    }
}
#[allow(clippy::too_many_arguments)]
impl Elem {
    fn print(
        &self,
        count: usize,
        e: &str,
        mut array: Vec<usize>,
        display_type: DisplayType,
    ) -> String {
        match self {
            Elem::Ref(r, _, _) => r.print(count + 1, e, array, display_type),
            Elem::Array(r, _) => {
                array.insert(0, r.len());
                r.iter().find(|a| !a.null()).unwrap_or(&Elem::Null).print(
                    count,
                    e,
                    array,
                    display_type,
                )
            }
            Elem::Struct(s, v, _) => s.print(count, e, *v, array),
            Elem::Usize(v) => {
                format!(
                    "{e}: {}{}usize{} {}",
                    "[".repeat(array.len()),
                    "&".repeat(count),
                    array
                        .iter()
                        .map(|a| format!("; {}]", display_size(*a)))
                        .collect::<Vec<String>>()
                        .join(""),
                    if array.is_empty() {
                        display_type.print(v)
                    } else {
                        String::new()
                    }
                )
            }
            Elem::Recursive(v) => {
                format!(
                    "{e}: {}{}recursive{} {}",
                    "[".repeat(array.len()),
                    "&".repeat(count),
                    array
                        .iter()
                        .map(|a| format!("; {}]", display_size(*a)))
                        .collect::<Vec<String>>()
                        .join(""),
                    if array.is_empty() {
                        display_type.print(v)
                    } else {
                        String::new()
                    }
                )
            }
            Elem::VFTable(n, v) => {
                format!(
                    "{e}: {}{}vftable<{n}>{} {}",
                    "[".repeat(array.len()),
                    "&".repeat(count),
                    array
                        .iter()
                        .map(|a| format!("; {}]", display_size(*a)))
                        .collect::<Vec<String>>()
                        .join(""),
                    if array.is_empty() {
                        display_type.print(v)
                    } else {
                        String::new()
                    }
                )
            }
            Elem::TooLarge(v) => {
                format!(
                    "{e}: {}{}large{} {}",
                    "[".repeat(array.len()),
                    "&".repeat(count),
                    array
                        .iter()
                        .map(|a| format!("; {}]", display_size(*a)))
                        .collect::<Vec<String>>()
                        .join(""),
                    if array.is_empty() {
                        display_type.print(v)
                    } else {
                        String::new()
                    }
                )
            }
            Elem::Failed(v) => {
                format!(
                    "{e}: {}{}failed{} {}",
                    "[".repeat(array.len()),
                    "&".repeat(count),
                    array
                        .iter()
                        .map(|a| format!("; {}]", display_size(*a)))
                        .collect::<Vec<String>>()
                        .join(""),
                    if array.is_empty() {
                        display_type.print(v)
                    } else {
                        String::new()
                    }
                )
            }
            Elem::Null => {
                format!(
                    "{e}: {}{}null{}",
                    "[".repeat(array.len()),
                    "&".repeat(count),
                    array
                        .iter()
                        .map(|a| format!("; {}]", display_size(*a)))
                        .collect::<Vec<String>>()
                        .join("")
                )
            }
        }
    }
}
fn display_size(n: usize) -> String {
    let rt = n.isqrt();
    if n >= 256 && rt * rt == n {
        format!("{rt} * {rt}")
    } else {
        n.to_string()
    }
}
impl DisplayType {
    fn print(self, n: &u32) -> String {
        match self {
            DisplayType::Hex => {
                format!("{n:08x}")
            }
            DisplayType::Bin => {
                format!("{n:032b}")
            }
            DisplayType::Num => {
                format!("{n:010}")
            }
            DisplayType::SignedNum => {
                format!("{:010}", n.cast_signed())
            }
            DisplayType::Float => {
                format!("{}", f32::from_bits(*n))
            }
            DisplayType::Str => String::from_utf8(n.to_le_bytes().to_vec()).unwrap_or_default(),
            DisplayType::None => String::new(),
        }
    }
}
