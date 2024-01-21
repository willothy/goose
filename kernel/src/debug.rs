pub mod registers {
    use core::arch::asm;

    #[allow(dead_code)]
    #[inline(always)]
    pub fn rsp() -> u64 {
        let rsp: u64;
        unsafe {
            asm! {
                "mov {}, rsp",
                out(reg) rsp,
            };
        }
        rsp
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn rbp() -> u64 {
        let rbp: u64;
        unsafe {
            asm! {
                "mov {}, rbp",
                out(reg) rbp,
            };
        }
        rbp
    }

    // Idea: would it be possible to build a live disassembler that can dump the running kernel
    // code? Could be useful for debugging, but if not it would still be interesting.
}
