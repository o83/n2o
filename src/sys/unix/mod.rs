#[cfg(any(target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "linux"))]
pub use ::sys::unix::linux::set_affinity;

#[cfg(any(target_os = "macos"))]
mod bsd;