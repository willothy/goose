extern "C" {
    pub fn k_memset(ptr: *mut u8, value: u8, count: usize);

    pub fn k_memcpy(dst: *mut u8, src: *const u8, size: usize);

    pub fn k_memmove(dst: *const u8, src: *const u8, size: usize);

    pub fn k_memcmp(s1: *const u8, s2: *const u8, n: usize) -> i8;
}
