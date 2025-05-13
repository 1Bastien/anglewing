pub mod linux;
pub mod macos;

#[cfg(target_os = "macos")]
pub use macos::check_and_setup_installation;

#[cfg(target_os = "linux")]
pub use linux::check_and_setup_installation;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub use linux::check_and_setup_installation;