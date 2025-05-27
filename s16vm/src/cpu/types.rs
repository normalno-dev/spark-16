// Converts 12-bits signed (offsets, for example) into i16 for simpler usage in the emulator
pub fn convert_12bit_to_signed(val: u16) -> i16 {
    let val = val & 0x0FFF;

    if val & 0x0800 != 0 {
        // negative
        (val | 0xF000) as i16
    } else {
        val as i16
    }
}
