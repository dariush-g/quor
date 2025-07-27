let x: i32 = 42;


@!pub
class Example {
    x: i32,
    y: i32

    fn func() {}
}

@pub, @ext Example
class Example2 {

}


@pub
fn add(a: i32, b: i32) {
    return a + b;
}

if (x > 0) {
    print("positive")
} else {
    print("negative")
}

let nums = [1, 2, 3]

