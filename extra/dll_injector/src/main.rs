#![windows_subsystem = "windows"]
use dll_syringe::{Syringe, process::OwnedProcess};
use std::process::Command;
unsafe fn inject_and_resume(exe: &str, dll: &str) -> windows::core::Result<()> {
    Command::new(exe).spawn()?;
    let syringe = Syringe::for_process(OwnedProcess::find_first_by_name("noita_back.exe").unwrap());
    syringe.inject(dll).unwrap();
    Ok(())
}
fn main() {
    unsafe {
        inject_and_resume(
            r"Z:/home/.local/share/Steam/steamapps/common/Noita/noita_back.exe",
            r"Z:/home/.r/noita_entangled_worlds/extra/malloc_probe/target/i686-pc-windows-gnu/release/malloc_probe.dll",
        )
        .unwrap();
    }
}
