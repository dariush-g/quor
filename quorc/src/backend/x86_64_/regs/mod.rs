pub fn reg32(r: &str) -> &'static str {
    match r {
        "rax" => "eax",
        "rbx" => "ebx",
        "rcx" => "ecx",
        "rdx" => "edx",
        "rsi" => "esi",
        "rdi" => "edi",
        "r8" => "r8d",
        "r9" => "r9d",
        "r10" => "r10d",
        "r11" => "r11d",
        "r12" => "r12d",
        "r13" => "r13d",
        "r14" => "r14d",
        "r15" => "r15d",
        _ => "eax",
    }
}

pub fn reg8(r: &str) -> &'static str {
    match r {
        "rax" => "al",
        "rbx" => "bl",
        "rcx" => "cl",
        "rdx" => "dl",
        "rsi" => "sil",
        "rdi" => "dil",
        "r8" => "r8b",
        "r9" => "r9b",
        "r10" => "r10b",
        "r11" => "r11b",
        "r12" => "r12b",
        "r13" => "r13b",
        "r14" => "r14b",
        "r15" => "r15b",
        _ => "al",
    }
}
