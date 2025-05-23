use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use tauri::Manager;

fn is_dir_empty_or_missing(path: &Path) -> bool {
  if !path.exists() {
    return true;
  }
  
  match fs::read_dir(path) {
    Ok(mut entries) => entries.next().is_none(),
    Err(e) => {
      log::error!("Failed to read directory {:?}: {}", path, e);
      true
    }
  }
}

fn are_important_files_missing(path: &Path) -> bool {
  let important_files = ["config.json", "readme.txt"];
  for file in important_files.iter() {
    if !path.join(file).exists() {
      return true;
    }
  }
  false
}

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
      log::info!("Copied file: {:?}", target);
    }
    
    #[cfg(unix)]
    {
      let mode = if path.is_dir() { 0o755 } else { 0o644 };
      fs::set_permissions(
        &target,
        fs::Permissions::from_mode(mode)
      ).map_err(|e| format!("Failed to set permissions for {:?}: {}", target, e))?;
    }
  }
  Ok(())
}

#[allow(dead_code)]
pub fn check_and_setup_installation(app_handle: &tauri::AppHandle) -> Result<(), String> {
  log::info!("Linux: Checking installation setup");
  
  let target_dir = Path::new("/home/docaret/.anglewing");
  let user_public_path = target_dir.join("public");
  
  let resource_path = app_handle
    .path()
    .resolve("public", tauri::path::BaseDirectory::Resource)
    .map_err(|e| format!("Could not find resource directory: {}", e))?;
  
  log::info!("Resource path: {:?}", resource_path);
  log::info!("User public path: {:?}", user_public_path);
  
  let animations_dir = user_public_path.join("animations");
  let backgrounds_dir = user_public_path.join("backgrounds");
  
  if is_dir_empty_or_missing(&user_public_path) || 
     is_dir_empty_or_missing(&animations_dir) || 
     is_dir_empty_or_missing(&backgrounds_dir) ||
     are_important_files_missing(&user_public_path) {
    log::info!("Public directory is empty, missing required subdirectories, or missing important files. Copying resources...");
    
    for dir in [&user_public_path, &animations_dir, &backgrounds_dir] {
      if let Err(e) = fs::create_dir_all(dir) {
        log::error!("Failed to create directory {:?}: {}", dir, e);
        return Err(format!("Failed to create directory structure: {}", e));
      }
      
      #[cfg(unix)]
      {
        if let Err(e) = fs::set_permissions(dir, fs::Permissions::from_mode(0o755)) {
          log::error!("Failed to set permissions for {:?}: {}", dir, e);
        }
      }
    }
    
    copy_dir_contents(&resource_path, &user_public_path)?;
    log::info!("Successfully copied public resources to user directory");
  } else {
    log::info!("Public directory exists and contains all required files and subdirectories");
  }
  
  Ok(())
}

#[allow(dead_code)]
pub fn get_public_folder_path(_app_dir_: &Path) -> std::path::PathBuf {
  let is_dev = std::env::var("TAURI_DEBUG").is_ok();
  
  if is_dev {
    let current_dir = std::env::current_dir()
      .expect("Could not get current directory");
    let dev_path = current_dir
      .parent()
      .unwrap_or(&current_dir)
      .join("public");
    
    log::info!("Development mode: using public directory at {:?}", dev_path);
    return dev_path;
  }
  
  let target_dir = Path::new("/home/docaret/.anglewing");
  let lib_path = target_dir.join("public");
  
  log::info!("Production mode: using public directory at {:?}", lib_path);
  
  if !lib_path.exists() {
    log::info!("Public directory does not exist at {:?}", lib_path);
  } else {
    log::info!("Public directory exists at {:?}", lib_path);
    #[cfg(unix)]
    {
      if let Ok(metadata) = fs::metadata(&lib_path) {
        log::info!("Current permissions: {:o}", metadata.permissions().mode());
      }
    }
  }
  
  lib_path
} 