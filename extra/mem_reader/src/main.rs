use eframe::{Frame, NativeOptions};
use egui::{CentralPanel, ComboBox, Context, Ui, UiBuilder};
use libc::{SIGSTOP, kill, pid_t};
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
                        if ui
                            .add_sized(
                                [200.0, 20.0],
                                egui::TextEdit::singleline(&mut self.reference),
                            )
                            .changed()
                            && self.reference.starts_with("0x")
                            && let Ok(n) = u32::from_str_radix(&self.reference[2..], 16)
                        {
                            self.elem =
                                Elem::check_global(n, &self.reader, &self.map, &mut vec![n], None);
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
                    egui::ScrollArea::vertical().id_salt("1").show(ui, |ui| {
                        if let Some(d) = self.elem.show(
                            ui,
                            &self.reader,
                            &self.map,
                            &mut self.elem.reference().map(|a| vec![a]).unwrap_or_default(),
                            0,
                            0,
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
                    egui::ScrollArea::vertical().id_salt("2").show(ui, |ui| {
                        if let Some(data) = &self.data {
                            match data {
                                Data::Values(refs, data) => {
                                    if !refs.is_empty() {
                                        ui.label(
                                            refs.iter()
                                                .map(|(a, b)| format!("0x{a:08x}->0x{b:08x}"))
                                                .collect::<Vec<String>>()
                                                .join(","),
                                        );
                                        ui.separator();
                                    }
                                    for (i, n) in data.iter().enumerate() {
                                        if data.len() != 1 {
                                            ui.label(i.to_string());
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
                                        ui.separator();
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
    Values(Vec<(u32, u32)>, Vec<u32>),
}
pub struct Reader {
    mem: File,
}
impl Reader {
    fn new(pid: usize) -> Self {
        unsafe {
            kill(pid_t::from(pid as i32), SIGSTOP);
        }
        let path = format!("/proc/{pid}/mem");
        Reader {
            mem: File::open(path).unwrap(),
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
#[derive(Debug)]
pub struct Struct {
    name: String,
    size: usize,
    fields: Option<Vec<Elem>>,
    reference: u32,
    skip: bool,
}
#[derive(Debug)]
pub enum Elem {
    Ref(Box<Elem>, u32, u32),
    Struct(Struct, u32),
    VFTable(u32),
    Array(Box<Elem>, Vec<u32>),
    Usize(u32),
    Recursive(u32),
    TooLarge(u32),
    Failed(u32),
}
impl PartialEq for Elem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Elem::Array(r1, n1), Elem::Array(r2, n2)) => r1 == r2 && n1 == n2,
            (Elem::Ref(r1, _, _), Elem::Ref(r2, _, _)) => r1 == r2,
            (Elem::Struct(_, _), Elem::Struct(_, _)) => false,
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
    pub fn reference(&self) -> Option<u32> {
        match self {
            Elem::Ref(_, r, _) => Some(*r),
            Elem::Struct(_, r) => Some(*r),
            _ => None,
        }
    }
    pub fn data(&self) -> Vec<u32> {
        vec![match self {
            Elem::Ref(_, v, _) => *v,
            Elem::Struct(_, v) => *v,
            Elem::VFTable(v) => *v,
            Elem::Array(_, v) => return v.clone(),
            Elem::Usize(v) => *v,
            Elem::Recursive(v) => *v,
            Elem::TooLarge(v) => *v,
            Elem::Failed(v) => *v,
        }]
    }
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ui: &mut Ui,
        reader: &Reader,
        map: &HashMap<u32, (String, usize)>,
        addrs: &mut Vec<u32>,
        count: usize,
        entry: usize,
        mut refs: Vec<(u32, u32)>,
        display_type: DisplayType,
    ) -> Option<Data> {
        match self {
            Elem::Struct(s, r) => {
                addrs.push(*r);
                let resp =
                    egui::CollapsingHeader::new(s.print(count, entry, *r, None)).show(ui, |ui| {
                        let mut e = 0;
                        let mut ret = None;
                        for f in s.fields(reader, map, addrs) {
                            let len = addrs.len();
                            ret = ret.or(f.show(
                                ui,
                                reader,
                                map,
                                addrs,
                                0,
                                e / 4,
                                Vec::new(),
                                display_type,
                            ));
                            e += f.size();
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
            s => {
                if ui
                    .label(s.print(count, entry, None, display_type))
                    .hovered()
                {
                    return Some(Data::Values(mem::take(&mut refs), s.data()));
                }
            }
        }
        None
    }
    pub fn array_eq(&self, other: &Self) -> bool {
        match self {
            Elem::Array(e, _) => e.as_ref() == other,
            Elem::Struct(_, _) => false,
            e => e == other,
        }
    }
    pub fn size(&self) -> usize {
        match self {
            Elem::Ref(_, _, _)
            | Elem::VFTable(_)
            | Elem::Usize(_)
            | Elem::Recursive(_)
            | Elem::Failed(_)
            | Elem::TooLarge(_) => 4,
            Elem::Struct(e, _) => e.size,
            Elem::Array(e, v) => e.size() * v.len(),
        }
    }
    pub fn value(&self) -> Option<u32> {
        Some(match self {
            Elem::Ref(_, v, _) => *v,
            Elem::Struct(_, v) => *v,
            Elem::VFTable(v) => *v,
            Elem::Array(_, _) => return None,
            Elem::Usize(v) => *v,
            Elem::Recursive(v) => *v,
            Elem::TooLarge(v) => *v,
            Elem::Failed(v) => *v,
        })
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
        Elem::Struct(s, reference)
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
        if addrs.contains(&table) {
            return Elem::Recursive(table);
        }
        addrs.push(reference);
        let addr_size = mem.get_size(reference).unwrap_or(u32::MAX);
        if let Some((name, size)) = map.get(&table) {
            if Some(name.as_ref()) == parent {
                Elem::VFTable(table)
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
    ) -> &mut Vec<Elem> {
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
    ) -> Vec<Elem> {
        let mut reference = self.reference;
        let mut fields = Vec::with_capacity(self.size / 4);
        if self.skip {
            fields.push(Elem::VFTable(mem.read_byte(reference).unwrap()));
            reference += 4;
        }
        let mut i = 0;
        while i < if self.skip {
            (self.size / 4).saturating_sub(1)
        } else {
            self.size / 4
        } {
            let len = addrs.len();
            let e = Elem::check_global(reference + 4 * i as u32, mem, map, addrs, Some(&self.name));
            i += (e.size() / 4).max(1);
            if let Some(last) = fields.last_mut()
                && last.array_eq(&e)
            {
                if let Elem::Array(_, vec) = last {
                    if vec.len() >= 1024 {
                        fields.push(e);
                    } else if let Some(v) = e.value() {
                        vec.push(v);
                    }
                } else {
                    let mut vec = Vec::new();
                    if let Some(v) = last.value() {
                        vec.push(v);
                    }
                    if let Some(v) = e.value() {
                        vec.push(v);
                    }
                    *last = Elem::Array(Box::new(e), vec);
                };
            } else {
                fields.push(e);
            }
            while len < addrs.len() {
                addrs.pop();
            }
        }
        fields.shrink_to_fit();
        fields
    }
    #[allow(clippy::too_many_arguments)]
    fn print(&self, count: usize, entry: usize, v: u32, array: Option<&[u32]>) -> String {
        format!(
            "[{entry}]{}{}<{}>{}({})",
            "&".repeat(count),
            self.name,
            self.size,
            array.map(|a| format!("[{}]", a.len())).unwrap_or_default(),
            array
                .map(|b| b
                    .iter()
                    .map(|v| format!("0x{v:08x}"))
                    .collect::<Vec<String>>()
                    .join(","))
                .unwrap_or(format!("0x{v:08x}")),
        )
    }
}
#[allow(clippy::too_many_arguments)]
impl Elem {
    fn print(
        &self,
        count: usize,
        e: usize,
        array: Option<&[u32]>,
        display_type: DisplayType,
    ) -> String {
        match self {
            Elem::Ref(r, _, _) => r.print(count + 1, e, array, display_type),
            Elem::Array(r, v) => r.print(count, e, Some(v), display_type),
            Elem::Struct(s, v) => s.print(count, e, *v, array),
            Elem::Usize(v) => {
                format!(
                    "[{e}]{}usize{}({})",
                    "&".repeat(count),
                    array.map(|a| format!("[{}]", a.len())).unwrap_or_default(),
                    display_type.print_opt(array, v)
                )
            }
            Elem::Recursive(v) => {
                format!(
                    "[{e}]{}recursive{}({})",
                    "&".repeat(count),
                    array.map(|a| format!("[{}]", a.len())).unwrap_or_default(),
                    display_type.print_opt(array, v)
                )
            }
            Elem::VFTable(v) => {
                format!(
                    "[{e}]{}VFTable{}({})",
                    "&".repeat(count),
                    array.map(|a| format!("[{}]", a.len())).unwrap_or_default(),
                    display_type.print_opt(array, v)
                )
            }
            Elem::TooLarge(v) => {
                format!(
                    "[{e}]{}TooLarge{}({})",
                    "&".repeat(count),
                    array.map(|a| format!("[{}]", a.len())).unwrap_or_default(),
                    display_type.print_opt(array, v)
                )
            }
            Elem::Failed(v) => {
                format!(
                    "[{e}]{}Failed{}({})",
                    "&".repeat(count),
                    array.map(|a| format!("[{}]", a.len())).unwrap_or_default(),
                    display_type.print_opt(array, v)
                )
            }
        }
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
    fn print_opt(self, a: Option<&[u32]>, v: &u32) -> String {
        if DisplayType::None == self {
            return String::new();
        }
        a.map(|b| {
            b.iter()
                .map(|v| self.print(v))
                .collect::<Vec<String>>()
                .join(",")
        })
        .unwrap_or(self.print(v))
    }
}
