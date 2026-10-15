// Independent Table 2-14 offsets, pp.2-22–23. These bytes are fixture data,
// not controller captures or evidence resolving DEC-4.01. No production encoders.
pub fn min_max_literal(segment:u8,point:u8)->[u8;50] {
    [segment,point,128,103,16,21,
     0,0,0x48,0x41, 0,0,0x80,0xbf, 0,0,0xc8,0x42,
     1,2,3,4,5, 6,7,8,9,10,
     0,0,0x20,0xc1, 0,0,0x48,0x43,
     11,12,13,14,2, 15,16,17,18,12, 0,0,0xc0,0x3f]
}

// Table 2-18, p.2-32: independent header and four-byte sample words.
pub fn single_history_literal(segment:u8,point:u8,kind:u8,index:u16,count:u8)->Vec<u8> {
    let mut b=vec![segment,point];b.extend(index.to_le_bytes());b.push(count);
    for _ in 0..count {b.extend(if kind>=3 {[0x78,0x56,0x34,0x12]}else{[0,0,0x48,0x41]});}b
}
