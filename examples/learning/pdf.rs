const N: usize = 1000000;

fn f(x: f64) -> f64 { x * x }
fn updf(_x: f64) -> f64 { 0.5 } // 50% a sample will fall in any bin
fn uicd(x: f64) -> f64 { 2.0 * x }
fn lpdf(x: f64) -> f64 { 0.5 * x }
fn licd(x: f64) -> f64 { (4.0 * x).sqrt() }
fn qpdf(x: f64) -> f64 { 3.0 / 8.0 * x * x }
fn qicd(x: f64) -> f64 { 8.0 * x.powf(1.0 / 3.0) }

fn f3d(x: [f64; 3]) -> f64 { x[2] * x[2] }
fn pdf3d(_x: [f64; 3]) -> f64 { 1.0 / (4.0 * std::f64::consts::PI) }

pub fn random3_uniform() -> [f64; 3] {
    loop {
        let p = [1.0 - 2.0 * fastrand::f64(), 1.0 - 2.0 * fastrand::f64(), 1.0 - 2.0 * fastrand::f64()];
        let lensq = p[0] * p[0] + p[1] * p[1] + p[2] * p[2];
        if lensq < 1.0 {
            return [p[0] / lensq.sqrt(), p[1] / lensq.sqrt(), p[2] / lensq.sqrt()];
        }
    }
}

fn approximate(f: impl Fn(f64) -> f64, pdf: impl Fn(f64) -> f64, icd: impl Fn(f64) -> f64) -> f64 {
    let mut area = 0.0;
    for _ in 0..N {
        let r = fastrand::f64();
        if r == 0.0 {
            // Ignore values that createa a NaN later
            continue;
        }
        let x: f64 = icd(r);
        // Weight the sample by the inverse of how likely we are to sample
        // there; indeed, if we sample somewhere more often, to not get a
        // bias from the fact we do so, we must weight samples we visit
        // often down and those we sample infrequently up
        area += f(x) / pdf(x);
    }
    area / N as f64
}

fn approximate_3d(f: impl Fn([f64; 3]) -> f64, pdf: impl Fn([f64; 3]) -> f64) -> f64 {
    let mut area = 0.0;
    for _ in 0..N {
        let x = random3_uniform();
        // Weight the sample by the inverse of how likely we are to sample
        // there; indeed, if we sample somewhere more often, to not get a
        // bias from the fact we do so, we must weight samples we visit
        // often down and those we sample infrequently up
        area += f(x) / pdf(x);
    }
    area / N as f64
}

// Integrates `f` over the uniform PDF 1/2 (CDF x/2, ICD 2x) using `N` samples
fn main() {
    println!("(uniform)         I = {}", approximate(f, updf, uicd));
    println!("(linear)          I = {}", approximate(f, lpdf, licd));
    println!("(quadratic)       I = {}", approximate(f, qpdf, qicd));
    println!("(random3_uniform) I = {}", approximate_3d(f3d, pdf3d));
}
