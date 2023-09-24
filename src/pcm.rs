pub fn to_buf(pcm: &[i16]) -> &[u8] {
    let len = pcm.len() * std::mem::size_of::<i16>();
    let ptr = pcm.as_ptr() as *const u8;
    return unsafe { std::slice::from_raw_parts(ptr, len) };
}

pub fn from_buf(buf: &[u8], len: usize) -> &[i16] {
    let ptr = buf.as_ptr() as *const i16;
    let pcm_len = len / std::mem::size_of::<i16>();
    return unsafe { std::slice::from_raw_parts(ptr, pcm_len) };
}
