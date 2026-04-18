use std::env::args;
use std::fs::File;
use std::io::Read;
fn main() {
    let mut args = args();
    args.next().unwrap();
    let old = args.next().unwrap();
    let new = args.next().unwrap();
    let iter = Iter::new(&old, &new);
    for (ptr, vo, vn) in iter {
        if let Some(vo) = vo {
            if vo != vn {
                println!("0x{ptr:08x} {} {}", vo.len(), vn.len());
                print(&vo);
                print_diff(&vo, &vn);
                print(&vn);
            }
        } else {
            println!("0x{ptr:08x} {}", vn.len());
            print(&vn);
        }
    }
}
fn print(v: &[u8]) {
    let mut iter = v.chunks_exact(4);
    for (i, c) in iter.by_ref().enumerate() {
        if i.is_multiple_of(16) {
            print!("{:04} ", i / 16);
        }
        print!("{:02x}{:02x}{:02x}{:02x} ", c[0], c[1], c[2], c[3]);
        if (i + 1).is_multiple_of(16) {
            println!()
        }
    }
    let s = iter.remainder();
    match s.len() {
        0 => println!(),
        1 => println!("{:02x}", s[0]),
        2 => println!("{:02x}{:02x}", s[0], s[1]),
        3 => println!("{:02x}{:02x}{:02x}", s[0], s[1], s[2]),
        _ => unreachable!(),
    }
}
fn print_diff(o: &[u8], n: &[u8]) {
    for (i, (o, n)) in o.chunks(64).zip(n.chunks(64)).enumerate() {
        for (j, (o, n)) in o.chunks(4).zip(n.chunks(4)).enumerate() {
            if o != n {
                print!("{i},{j} ");
            }
        }
    }
    println!()
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
