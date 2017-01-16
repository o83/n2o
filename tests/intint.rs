#![feature(core_intrinsics)]
#![feature(target_feature)]

// $ rustc -C opt-level=3 -C target-feature=+avx --emit asm intint.rs
// $ cat intint.s | grep mul

#[target_feature = "+avx"]
pub fn mul_array_float<'a>(x: &mut[f64], y: &[f64]) {
    let n: Vec<f64> = x.iter().zip(y).map(|(d,c)|d*c).collect();
    println!("{:?}",n);
}

#[target_feature = "+avx"]
pub fn mul_array<'a>(x: &mut [u64], y: &[u64]) {
    let n: Vec<u64> = x.iter().zip(y).map(|(d,c)|d*c).collect();
    println!("{:?}",n);
}

fn main() {
    let mut fa = vec![0.0, 1.0, 4.0, 9.0, 0.0, 1.0, 4.0, 9.0, 0.0, 1.0, 4.0, 9.0, 0.0, 1.0, 4.0, 9.0];
    let fb     = vec![0.0, 2.0, 6.0, 0.0, 0.0, 1.0, 4.0, 9.0, 0.0, 2.0, 6.0, 0.0, 0.0, 1.0, 4.0, 9.0];
    let mut a  = vec![0, 1, 4, 9, 0, 1, 4, 9, 0, 1, 4, 9, 0, 1, 4, 9];
    let b      = vec![0, 2, 6, 0, 0, 1, 4, 9, 0, 1, 4, 9, 0, 1, 4, 9];
    mul_array_float(fa.as_mut_slice(), fb.as_slice());
    mul_array(a.as_mut_slice(), b.as_slice());
    println!("{:?} {:?}", a, fa);
}
