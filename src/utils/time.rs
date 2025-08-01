pub fn clock(hours: u32, mins: u32) -> char {
    let mut idx: u32 = hours % 12;
    if mins >= 45 {
        idx += 1;
    }

    if idx == 0 {
        idx = 12;
    }

    if mins >= 15 && mins < 45 {
        idx += 12;
    };
    char::from_u32(0x1F54F + idx).unwrap()
}
