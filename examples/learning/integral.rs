const A: f64 = 0.0;
const B: f64 = 2.0;
const N: usize = 1000000;

fn f(x: f64) -> f64 { x * x }

// Integrates `f` in the range `A`, `B` using `N` samples
fn main() {
    let mut area = 0.0;
    for _ in 0..N {
        let x: f64 = (B - A) * fastrand::f64() + A;
        area += f(x);
    }
    println!("I = {}", (B - A) * (area / N as f64));
    println!("(analytically: {})", ((1.0 / 3.0) * (B * B * B)) - ((1.0 / 3.0) * (A * A * A)));
}
