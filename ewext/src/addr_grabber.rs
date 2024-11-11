use std::{os::raw::c_void, ptr};

use iced_x86::{Decoder, DecoderOptions, Mnemonic};

pub(crate) unsafe fn grab_addr_from_instruction(
    func: *const c_void,
    offset: isize,
    expected_mnemonic: Mnemonic,
) -> *mut c_void {
    let instruction_addr = func.wrapping_offset(offset);
    // We don't really have an idea of how many bytes the instruction takes, so just take *enough* bytes for most cases.
    let instruction_bytes = ptr::read_unaligned(instruction_addr.cast::<[u8; 16]>());
    let mut decoder = Decoder::with_ip(
        32,
        &instruction_bytes,
        instruction_addr as u64,
        DecoderOptions::NONE,
    );
    let instruction = decoder.decode();

    if instruction.mnemonic() != expected_mnemonic {
        println!("Encountered unexpected mnemonic: {}", instruction);
    }
    assert_eq!(instruction.mnemonic(), expected_mnemonic);

    instruction.memory_displacement32() as *mut c_void
}
