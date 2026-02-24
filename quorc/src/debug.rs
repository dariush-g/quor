#[macro_export]
macro_rules! debug_log {
    ($($key:expr => $val:expr),*) => {{
        if $crate::target::in_debug_mode() {
            $(println!("\x1b[35mDEBUG\x1b[0m \x1b[90mquorc:\x1b[0m {} \x1b[90m=\x1b[0m \x1b[33m{}\x1b[0m",
                $key,
                $val
            );)*
        }
    }};
}
