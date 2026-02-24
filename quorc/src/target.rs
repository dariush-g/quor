use std::sync::OnceLock;

static TARGET_ARCH: OnceLock<String> = OnceLock::new();
static TARGET_OS: OnceLock<String> = OnceLock::new();
static DEBUG_MODE: OnceLock<bool> = OnceLock::new();

pub fn init_target(
    arch_override: Option<String>,
    os_override: Option<String>,
    in_debug_mode: bool,
) {
    TARGET_ARCH
        .set(arch_override.unwrap_or(std::env::consts::ARCH.to_string()))
        .ok();
    TARGET_OS
        .set(os_override.unwrap_or(std::env::consts::OS.to_string()))
        .ok();
    DEBUG_MODE.set(in_debug_mode).ok();
}

pub fn target_arch() -> &'static str {
    TARGET_ARCH.get().unwrap()
}

pub fn target_os() -> &'static str {
    TARGET_OS.get().unwrap()
}

pub fn in_debug_mode() -> bool {
    *DEBUG_MODE.get().unwrap()
}
