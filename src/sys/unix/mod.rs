#[cfg(any(target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "linux"))]
pub use ::sys::unix::linux::set_affinity;

#[cfg(any(target_os = "macos"))]
mod bsd;

#[cfg(any(target_os = "macos"))]
pub use ::sys::unix::bsd::set_affinity;