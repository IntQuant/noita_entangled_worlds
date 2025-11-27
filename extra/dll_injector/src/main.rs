#![windows_subsystem = "windows"]
use std::ffi::CString;
use std::mem::transmute;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows::Win32::{
    Foundation::*,
    System::{Diagnostics::Debug::WriteProcessMemory, Memory::*, Threading::*},
};
use windows::core::PCSTR;
unsafe fn inject_and_resume(exe: &str, dll: &str) -> windows::core::Result<()> {
    unsafe {
        let exe_c = CString::new(exe).unwrap();
        let dll_c = CString::new(dll).unwrap();
        let si = STARTUPINFOA::default();
        let mut pi = PROCESS_INFORMATION::default();
        CreateProcessA(
            PCSTR(exe_c.as_ptr() as _),
            None,
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            None,
            &si,
            &mut pi,
        )?;
        let remote_mem = VirtualAllocEx(
            pi.hProcess,
            None,
            dll_c.as_bytes_with_nul().len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        WriteProcessMemory(
            pi.hProcess,
            remote_mem,
            dll_c.as_ptr() as _,
            dll_c.as_bytes_with_nul().len(),
            None,
        )?;
        let kernel32 = GetModuleHandleA(PCSTR(c"kernel32.dll".as_ptr().cast()))?;
        let loadlibrary_ptr = GetProcAddress(kernel32, PCSTR(c"LoadLibraryA".as_ptr().cast()));
        let h_thread = CreateRemoteThread(
            pi.hProcess,
            None,
            0,
            Some(transmute::<
                unsafe extern "system" fn() -> isize,
                unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
            >(loadlibrary_ptr.unwrap())),
            Some(remote_mem),
            0,
            None,
        )?;
        WaitForSingleObject(h_thread, INFINITE);
        ResumeThread(pi.hThread);
        CloseHandle(h_thread)?;
        CloseHandle(pi.hThread)?;
        CloseHandle(pi.hProcess)?;
    }
    Ok(())
}
fn main() {
    unsafe {
        inject_and_resume(r"./noita_back.exe", r"./malloc_probe.dll").unwrap();
    }
}
