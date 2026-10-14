// Independently transcribed layouts from Tables 2-16/17 and the record diagrams.
// No production record/value encoders or catalog metadata are used.
pub fn alarm_literal(index: u16) -> [u8; 23] {
    let mut b = [0xee; 23];
    b[1..5].copy_from_slice(&[1, 2, 3, 4]);
    match index % 6 {
        0 => b[0] = 0,
        1 => {
            b[0] = 0xc1;
            b[5..9].copy_from_slice(&[33, 101, 32, 3]);
            b[9..19].copy_from_slice(b"STATUS ON ");
            b[19..23].copy_from_slice(&[0, 0, 0x80, 0x3f]);
        }
        2 => {
            b[0] = 0x42; b[5] = 6;
            b[6..19].copy_from_slice(b"FST CONDITION");
            b[19..23].copy_from_slice(&[1, 0, 0xc0, 0x7f]);
        }
        3 => { b[0] = 0x83; b[5..23].copy_from_slice(b"USER TEXT ALARM   "); }
        4 => {
            b[0] = 4; b[5..19].copy_from_slice(b"USER VALUE    ");
            b[19..23].copy_from_slice(&[0, 0, 0x48, 0xc1]);
        }
        _ => b[0] = 0xff,
    }
    b
}
pub fn event_literal(index: u16) -> [u8; 22] {
    let mut b = [0xdd; 22];
    b[0] = (index % 9) as u8;
    b[1..5].copy_from_slice(&[0x78, 0x56, 0x34, 0x12]);
    match index % 9 {
        0 => {}
        1 => {
            b[5..12].copy_from_slice(&[b'A', b'B', b'C', 103, 16, 21, 7]);
            b[12..16].copy_from_slice(&[0, 0, 0x48, 0x41]);
            b[16..20].copy_from_slice(&[0, 0, 0x80, 0x3f]);
        }
        2 => { b[5] = 154; b[6..22].copy_from_slice(b"MODULE INSERTED "); }
        3 => {
            b[5] = 6; b[6..10].copy_from_slice(&[0, 0, 0x48, 0x41]);
            b[10..20].copy_from_slice(b"FST EVENT ");
        }
        4 => { b[5..9].copy_from_slice(&[b'X', b'Y', b'Z', 249]); b[9..22].copy_from_slice(b"CONFIG CHANGE"); }
        5 | 6 => b[5..9].copy_from_slice(&[1, 2, 3, 4]),
        7 => {
            b[5..11].copy_from_slice(&[b'A', b'B', b'C', 103, 16, 21]);
            b[11..15].copy_from_slice(&[0, 0, 0x80, 0x3f]);
            b[15..19].copy_from_slice(&[0, 0, 0x48, 0x41]);
        }
        _ => b[0] = 200,
    }
    b
}
