

@public
fn add(a: int, b: int) -> int {
    return a + b;
}


@public 
interface Shape {
    fn area() -> float;
}

@public
class Circle : ExampleTrait {
    @public | @const
    area: float;

    @impl
    fn area(self) -> float {
        self.area
    }
}