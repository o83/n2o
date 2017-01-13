
pub mod errno;
pub mod tcp;

#[cfg(any(target_os = "sel4"))]
mod linux;

#[cfg(any(target_os = "linux"))]
mod linux;
mod xsock;

#[cfg(any(target_os = "linux"))]
pub use ::io::unix::linux::{Events, Selector};

#[cfg(any(target_os = "macos"))]
mod bsd;

#[cfg(any(target_os = "macos"))]
pub use ::io::unix::bsd::{Events, Selector};

// pub mod fd;
pub mod stdio;
