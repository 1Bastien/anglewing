pub mod windows;
pub mod macos;
pub mod linux;

pub fn check_and_setup_installation(app_handle: &tauri::AppHandle) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    // Sur Windows, on n'a plus besoin de configuration spéciale
    // car on utilise le système de ressources de Tauri
    log::info!("Windows: Utilisation du système de ressources de Tauri, pas de configuration spéciale nécessaire");
    return Ok(());
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