pub mod windows;
pub mod macos;
pub mod linux;

pub fn check_and_setup_installation(app_handle: &tauri::AppHandle) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    return windows::check_and_setup_installation(app_handle);
  }
  
  #[cfg(target_os = "macos")]
  {
    return macos::check_and_setup_installation(app_handle);
  }
  
  #[cfg(target_os = "linux")]
  {
    return linux::check_and_setup_installation(app_handle);
  }
  
  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  {
    return Ok(());
  }
} 