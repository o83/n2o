#![feature(core_intrinsics)]
#![feature(target_feature)]

// $ rustc -C opt-level=3 -C target-feature=+avx --emit asm intint.rs
// $ cat intint.s | grep mul

#[target_feature = "+avx2"]
pub fn mul_array<'a>(x: &mut[f64], y: &[f64]) {
    debug_assert!(x.len() == y.len());
    unsafe {
        std::intrinsics::assume(x.as_ptr() as usize % 64 == 0);
        std::intrinsics::assume(y.as_ptr() as usize % 64 == 0);
    }
    for i in 0 .. x.len() {
        x[i] = y[i] * x[i];
    }
}

fn main() {
    let mut a = vec![0.0, 1.0, 4.0, 9.0, 0.0, 1.0, 4.0, 9.0];
    let b = vec![0.0, 2.0, 6.0, 0.0, 0.0, 1.0, 4.0, 9.0];
    //let mut a = vec![0, 1, 4, 9, 0, 1, 4, 9];
    //let b     = vec![0, 2, 6, 0, 0, 1, 4, 9];
    mul_array(a.as_mut_slice(), b.as_slice());
    println!("{:?}", a);
}
