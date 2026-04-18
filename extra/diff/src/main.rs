use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::Read;
fn main() {
    let mut args = args();
    args.next().unwrap();
    let file1 = args.next().unwrap();
    let file2 = args.next().unwrap();
    let hash1 = get_hash(&file1);
    let hash2 = get_hash(&file2);
    for (k, v1) in &hash1 {
        if let Some(v2) = hash2.get(k) {
            if v1 != v2 {
                println!(
                    "file1 and file2 contains 0x{k:x}, {} {}",
                    v1.len(),
                    v2.len()
                );
                println!("{v1:?}");
                println!("{v2:?}");
            }
        } else {
            println!("only file1 contains 0x{k:x}");
            println!("{v1:?}");
        }
    }
    for (k, v2) in &hash2 {
        if !hash1.contains_key(k) {
            println!("only file2 contains 0x{k:x}");
            println!("{v2:?}");
        }
    }
}
fn get_hash(s: &str) -> HashMap<u32, Box<[u8]>> {
    let mut file = File::open(s).unwrap();
    let mut hashmap = HashMap::with_capacity(65536);
    let mut buf = [0, 0, 0, 0];
    while file.read_exact(&mut buf).is_ok() {
        let ptr = u32::from_le_bytes(buf);
        file.read_exact(&mut buf).unwrap();
        let size = u32::from_le_bytes(buf);
        let mut vec = vec![0; size as usize];
        file.read_exact(&mut vec).unwrap();
        hashmap.insert(ptr, vec.into());
    }
    hashmap
}
