use std::fs;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use dirs;
use tauri::Manager;

#[allow(dead_code)]
pub fn check_and_setup_installation(app_handle: &tauri::AppHandle) -> Result<(), String> {
  log::info!("Linux: Setting up user directory structure");
  
  // Get the home directory path
  let home_dir = dirs::home_dir()
    .ok_or_else(|| "Could not find home directory".to_string())?;
  let user_public_path = home_dir.join(".anglewing/public");
  
  // Get the resource path (the bundled public folder)
  let resource_path = app_handle
    .path()
    .resolve("public", tauri::path::BaseDirectory::Resource)
    .map_err(|e| format!("Could not find resource directory: {}", e))?;
  
  log::info!("Resource path: {:?}", resource_path);
  log::info!("User public path: {:?}", user_public_path);
  
  // Create the base directory if it doesn't exist
  if !user_public_path.exists() {
    fs::create_dir_all(&user_public_path)
      .map_err(|e| format!("Failed to create user public directory: {}", e))?;
  }
  
  // Function to recursively copy directory contents
  fn copy_dir_contents(from: &Path, to: &Path) -> Result<(), String> {
    if !from.exists() {
      return Err(format!("Source path does not exist: {:?}", from));
    }
    
    if !to.exists() {
      fs::create_dir_all(to)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;
    }
    
    for entry in fs::read_dir(from)
      .map_err(|e| format!("Failed to read source directory: {}", e))? {
      let entry = entry
        .map_err(|e| format!("Failed to read directory entry: {}", e))?;
      let path = entry.path();
      let target = to.join(path.file_name()
        .ok_or_else(|| "Failed to get file name".to_string())?);
      
      if path.is_dir() {
        copy_dir_contents(&path, &target)?;
      } else {
        fs::copy(&path, &target)
          .map_err(|e| format!("Failed to copy file {:?} to {:?}: {}", path, target, e))?;
      }
      
      // Set permissions to 755 for directories and 644 for files
      let mode = if path.is_dir() { 0o755 } else { 0o644 };
      fs::set_permissions(
        &target,
        fs::Permissions::from_mode(mode)
      ).map_err(|e| format!("Failed to set permissions for {:?}: {}", target, e))?;
    }
    Ok(())
  }
  
  // Copy the contents from resource directory to user directory
  copy_dir_contents(&resource_path, &user_public_path)?;
  
  log::info!("Successfully copied public resources to user directory");
  Ok(())
}

#[allow(dead_code)]
pub fn get_public_folder_path(_app_dir_: &Path) -> std::path::PathBuf {
  let home_dir = dirs::home_dir()
    .expect("Could not find home directory");
  let lib_path = home_dir.join(".anglewing/public");
  
  // Create the directory structure if it doesn't exist
  if !lib_path.exists() {
    log::info!("Creating public directory structure at {:?}", lib_path);
    
    if let Err(e) = fs::create_dir_all(&lib_path) {
      log::error!("Failed to create public directory: {}", e);
    } else {
      let animations_dir = lib_path.join("animations");
      let backgrounds_dir = lib_path.join("backgrounds");
      
      let dirs = [lib_path.as_path(), &animations_dir, &backgrounds_dir];
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
  
  lib_path
} 