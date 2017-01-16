#![feature(simd, simd_ffi, link_llvm_intrinsics)]
#![allow(non_snake_case)]
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#![feature(target_feature)]

extern crate kernel;
extern crate simdty;

#[test]
#[target_feature = "+avx"]
fn avx_mul() {
    let a = ::simdty::f64x4(0.0, 1.0, 4.0, 9.0);
    let b = ::simdty::f64x4(0.0, 2.0, 6.0, 0.0);
    let c = unsafe { kernel::llvm::x86::avx_max_pd_256(a, b) };

    assert_eq!(c.0, 0.0);
    assert_eq!(c.1, 2.0);
    assert_eq!(c.2, 6.0);
    assert_eq!(c.3, 9.0);
}
