use std::env::args;
use std::fs::File;
use std::io::Read;
fn main() {
    let mut args = args();
    args.next().unwrap();
    let old = args.next().unwrap();
    let new = args.next().unwrap();
    let disp = args.next().map(|a| a.as_str().into()).unwrap_or(Disp::Hex);
    let iter = Iter::new(&old, &new);
    let mut first = true;
    for (ptr, vo, vn) in iter {
        if let Some(vo) = vo {
            if vo != vn {
                if !first {
                    println!();
                }
                first = false;
                println!("0x{ptr:08x} {} {}", vo.len(), vn.len());
                print(&vo, disp);
                print_diff(&vo, &vn, disp);
                print(&vn, disp);
            }
        } else {
            if !first {
                println!();
            }
            first = false;
            println!("0x{ptr:08x} {}", vn.len());
            print(&vn, disp);
        }
    }
}
fn print(v: &[u8], disp: Disp) {
    let mut iter = v.chunks_exact(4);
    let w = disp.num();
    for (i, c) in iter.by_ref().enumerate() {
        if i.is_multiple_of(w) {
            print!("{:04} ", i / w);
        }
        disp.print(u32::from_le_bytes([c[0], c[1], c[2], c[3]]));
        if (i + 1).is_multiple_of(w) {
            println!()
        }
    }
    let s = iter.remainder();
    match s.len() {
        0 => println!(),
        1 => {
            disp.print(u32::from_le_bytes([s[0], 0, 0, 0]));
            println!()
        }
        2 => {
            disp.print(u32::from_le_bytes([s[0], s[1], 0, 0]));
            println!()
        }
        3 => {
            disp.print(u32::from_le_bytes([s[0], s[1], s[2], 0]));
            println!()
        }
        _ => unreachable!(),
    }
}
fn print_diff(o: &[u8], n: &[u8], disp: Disp) {
    let w = disp.num();
    for (i, (o, n)) in o.chunks(4 * w).zip(n.chunks(4 * w)).enumerate() {
        for (j, (o, n)) in o.chunks(4).zip(n.chunks(4)).enumerate() {
            if o != n {
                print!("{i},{j} ");
            }
        }
    }
    println!()
}
#[derive(Clone, Copy)]
enum Disp {
    Hex,
    Bin,
    Int,
    Uint,
    Float,
    Str,
}
impl Disp {
    fn num(self) -> usize {
        match self {
            Self::Bin => 8,
            Self::Float => 4,
            _ => 16,
        }
    }
    fn print(self, n: u32) {
        match self {
            Self::Hex => {
                print!("{n:08x} ")
            }
            Self::Bin => {
                print!("{n:032b} ")
            }
            Self::Int => {
                let n = n.cast_signed();
                if n.is_negative() {
                    print!("{n:010} ")
                } else {
                    print!(" {n:010} ")
                }
            }
            Self::Uint => {
                print!("{n:010} ")
            }
            Self::Float => {
                print!("{} ", f32::from_bits(n))
            }
            Self::Str => {
                print!("{} ", String::from_utf8_lossy(&n.to_le_bytes()))
            }
        }
    }
}
impl From<&str> for Disp {
    fn from(value: &str) -> Self {
        match value {
            "hex" => Self::Hex,
            "bin" => Self::Bin,
            "int" => Self::Int,
            "uint" => Self::Uint,
            "float" => Self::Float,
            "str" => Self::Str,
            _ => unreachable!(),
        }
    }
}
pub struct Iter {
    old: File,
    new: File,
    buf: [u8; 4],
}
impl Iter {
    pub fn new(old: &str, new: &str) -> Self {
        let old = File::open(old).unwrap();
        let new = File::open(new).unwrap();
        let buf = [0, 0, 0, 0];
        Iter { old, new, buf }
    }
}
impl Iterator for Iter {
    type Item = (u32, Option<Vec<u8>>, Vec<u8>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.new.read_exact(&mut self.buf).is_ok() {
            let ptr = u32::from_le_bytes(self.buf);
            let old = if self.old.read_exact(&mut self.buf).is_ok() {
                let old_ptr = u32::from_le_bytes(self.buf);
                assert_eq!(old_ptr, ptr);
                self.old.read_exact(&mut self.buf).unwrap();
                let size = u32::from_le_bytes(self.buf) as usize;
                let mut vec = vec![0; size];
                self.old.read_exact(&mut vec).unwrap();
                Some(vec)
            } else {
                None
            };
            self.new.read_exact(&mut self.buf).unwrap();
            let size = u32::from_le_bytes(self.buf) as usize;
            let mut vec = vec![0; size];
            self.new.read_exact(&mut vec).unwrap();
            Some((ptr, old, vec))
        } else {
            None
        }
    }
}
