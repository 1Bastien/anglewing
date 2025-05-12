use std::fs;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;

#[allow(dead_code)]
pub fn check_and_setup_installation(_app_handle: &tauri::AppHandle) -> Result<(), String> {
  log::info!("Linux doesn't require special installation. Will create folders at runtime.");
  Ok(())
}

#[allow(dead_code)]
pub fn get_public_folder_path(_app_dir_: &Path) -> std::path::PathBuf {
  let lib_path = Path::new("/usr/lib/Anglewing/_up_/public");
  
  // Create the directory structure if it doesn't exist
  if !lib_path.exists() {
    log::info!("Creating public directory structure at {:?}", lib_path);
    
    if let Err(e) = fs::create_dir_all(&lib_path) {
      log::error!("Failed to create public directory: {}", e);
    } else {
      let animations_dir = lib_path.join("animations");
      let backgrounds_dir = lib_path.join("backgrounds");
      
      let dirs = [lib_path, &animations_dir, &backgrounds_dir];
      for dir in &dirs {
        if let Err(e) = fs::create_dir_all(dir) {
          log::error!("Failed to create directory {:?}: {}", dir, e);
        } else {
          // Set directory permissions to 755 (rwxr-xr-x)
          if let Err(e) = fs::set_permissions(dir, fs::Permissions::from_mode(0o755)) {
            log::error!("Failed to set permissions for {:?}: {}", dir, e);
          }
          log::info!("Successfully created directory with permissions: {:?}", dir);
        }
      }
    }
  } else {
    log::info!("Public directory already exists at {:?}", lib_path);
    // Log current permissions
    if let Ok(metadata) = fs::metadata(&lib_path) {
      log::info!("Current permissions: {:o}", metadata.permissions().mode());
    }
  }
  
  lib_path.to_path_buf()
} 