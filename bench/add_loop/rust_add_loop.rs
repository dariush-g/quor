fn add(x: i32, y: i32) -> i32 {
    x.wrapping_add(y)
}

fn main() {
    let mut a: i32 = 0;
    for i in 0..100_000_000 {
        a = add(a, i);
    }
    std::process::exit(a);
}
