#

```quor

@import("io")

@public
fn add(a: int, b: int) -> int {
    print("a")

    return a + b;
}

@public
interface Shape {
    fn area(self) -> float;
}

@public
class Circle: Shape {
    @[private, const]
    radius: float;

    @constructor
    fn new(radius: int) -> Circle {
        self~radius = radius;
        return self;
    }

    @override
    fn area() -> float {
        return 3.14 * self~radius * self~radius;
    }
}

fn main()->int {
    let circle: Circle = Circle::new(5);
    let area: float = circle~area();

    io::print(area);

    return 0;
}

```
