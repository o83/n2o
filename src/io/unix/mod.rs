
pub mod errno;
pub mod tcp;

#[cfg(any(target_os = "linux"))]
mod epoll;

#[cfg(any(target_os = "linux"))]
pub use ::io::unix::epoll::{Events, Selector};

#[cfg(any(target_os = "macos"))]
mod kqueue;

#[cfg(any(target_os = "macos"))]
pub use ::io::unix::kqueue::{Events, Selector};

pub mod lazy;
pub mod fd;
pub mod stdio;
