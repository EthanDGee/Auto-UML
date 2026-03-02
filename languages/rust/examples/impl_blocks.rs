struct Calculator;

impl Calculator {
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}

impl Calculator {
    fn clear(&mut self) {
        // ...
    }
}

struct AdvancedCalculator {
    base: Calculator,
}

impl AdvancedCalculator {
    fn multiply(&self, a: f64, b: f64) -> f64 { a * b }
}

struct Config<T> {
    data: T,
}

impl<T> Config<T> {
    fn get(&self) -> &T { &self.data }
}
