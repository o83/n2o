#![feature(target_feature)]

// $ rustc -C opt-level=3 --emit asm core_int.rs
// $ cat core_int.s | grep mul

#[target_feature = "+avx"]
pub fn mul_array_float<'a>(x: &mut [f64], y: &[f64]) {
    for (xi, &yi) in x.iter_mut().zip(y.iter()) {
        *xi *= yi;
    }
}

#[target_feature = "+avx"]
pub fn mul_array<'a>(x: &mut [u64], y: &[u64]) {
    for (xi, &yi) in x.iter_mut().zip(y.iter()) {
        *xi *= yi;
    }
}

fn main() {
    let mut fa = vec![10.0, 1.0, 4.0, 9.0, 10.0, 1.0, 4.0, 9.0, 10.0, 1.0, 4.0, 9.0, 10.0, 1.0, 4.0, 9.0];
    let fb     = vec![10.0, 2.0, 6.0, 2.0, 10.0, 1.0, 4.0, 9.0, 10.0, 2.0, 6.0, 2.0, 10.0, 1.0, 4.0, 9.0];
    let mut  a = vec![10, 1, 4, 9, 10, 1, 4, 9, 10, 1, 4, 9, 10, 1, 4, 9];
    let  b     = vec![10, 2, 6, 2, 10, 1, 4, 9, 10, 2, 6, 2, 10, 1, 4, 9];
    mul_array(&mut a, &b);
    mul_array_float(&mut fa, &fb);
    println!("{:?}", fa);
    println!("{:?}", a);
}

