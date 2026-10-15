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

// Independent Table 2-19 oracle: no production request/reply codec is used.
// Timestamp bytes deliberately differ from values; each point/period has a distinct float.
pub fn multi_history_literal(segment: u8, start: u16, first: u8, points: u8, current: u16, periods: u8) -> Vec<u8> {
    let [lo, hi] = start.to_le_bytes();
    let mut b = vec![segment, lo, hi];
    b.extend(current.to_le_bytes());
    b.push(((u16::from(points) + 1) * u16::from(periods)) as u8);
    for period in 0..periods {
        b.extend((0x12345600u32 + u32::from(start) + u32::from(period)).to_le_bytes());
        for point in 0..points {
            b.extend((f32::from(first) + f32::from(point) + 1000.0 * f32::from(period)).to_le_bytes());
        }
    }
    b
}

// Tables 2-20/2-21; independent literal offsets, not production codec output.
pub fn day_indices_literal(segment:u8)->Vec<u8>{vec![segment,4,0,3,0,2,0,2,0]}
pub fn day_history_literal(segment:u8,point:u8,day:u8,month:u8)->Vec<u8>{
    let mut b=vec![segment,point,day,month,3,0,2,0];
    let words=if point==255 {[[0x78,0x56,0x34,0x12],[255;4],[0;4],[1,0,0,0],[0,0,0,0x80]]}else{[[0,0,0x80,0x3f],[0,0,0,0xc0],[0x34,0x12,0xc0,0x7f],[0,0,0,0x80],[0,0,0x80,0x7f]]};
    for word in words{b.extend(word);} b
}

// Independent Table 2-22 time-major oracle. Point order, not ascending order,
// controls value positions; duplicated requested points duplicate their word.
pub fn selected_history_literal(segment:u8,start:u16,points:&[u8],timestamps:bool,current:u16,periods:u8)->Vec<u8>{
    let mut b=vec![1,segment];b.extend(current.to_le_bytes());b.extend([periods,u8::from(timestamps),points.len() as u8]);
    for period in 0..periods {
        if timestamps {b.extend((0x12345600u32+u32::from(start)+u32::from(period)).to_le_bytes());}
        for point in points {b.extend((f32::from(*point)+1000.0*f32::from(period)).to_le_bytes());}
    }b
}
