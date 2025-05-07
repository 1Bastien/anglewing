use std::fs;
use std::path::Path;

#[allow(dead_code)]
pub fn check_and_setup_installation(_app_handle: &tauri::AppHandle) -> Result<(), String> {
  log::info!("Linux doesn't require special installation. Will create folders at runtime.");
  Ok(())
}

#[allow(dead_code)]
pub fn get_public_folder_path(app_dir: &Path) -> std::path::PathBuf {
  // First, check if there's a public folder in the same directory as the app
  let same_level_public = app_dir.join("public");
  if same_level_public.exists() {
    return same_level_public;
  }
  
  // Check if we need to create the public folder and its subdirectories
  if !same_level_public.exists() {
    // Try to create the public directory
    if let Err(e) = fs::create_dir_all(&same_level_public) {
      log::error!("Failed to create public directory: {}", e);
    } else {
      // Create animations and backgrounds subdirectories
      let animations_dir = same_level_public.join("animations");
      let backgrounds_dir = same_level_public.join("backgrounds");
      
      if let Err(e) = fs::create_dir_all(&animations_dir) {
        log::error!("Failed to create animations directory: {}", e);
      }
      
      if let Err(e) = fs::create_dir_all(&backgrounds_dir) {
        log::error!("Failed to create backgrounds directory: {}", e);
      }
    }
  }
  
  // Try standard locations, in order:
  // 1. Same level as executable (already checked)
  // 2. /opt/Anglewing/
  // It's less common for apps to install to /usr/local/bin, so we might need a different approach
  let opt_path = Path::new("/opt/Anglewing");
  
  if opt_path.exists() {
    let opt_public = opt_path.join("public");
    if !opt_public.exists() {
      if let Err(e) = fs::create_dir_all(&opt_public) {
        log::error!("Failed to create public directory in /opt: {}", e);
      } else {
        let animations_dir = opt_public.join("animations");
        let backgrounds_dir = opt_public.join("backgrounds");
        
        if let Err(e) = fs::create_dir_all(&animations_dir) {
          log::error!("Failed to create animations directory: {}", e);
        }
        
        if let Err(e) = fs::create_dir_all(&backgrounds_dir) {
          log::error!("Failed to create backgrounds directory: {}", e);
        }
      }
    }
    return opt_public;
  }
  
  // Try in the user's home directory
  if let Ok(home_dir) = std::env::var("HOME") {
    let home_public = Path::new(&home_dir).join(".anglewing").join("public");
    if !home_public.exists() {
      if let Err(e) = fs::create_dir_all(&home_public) {
        log::error!("Failed to create public directory in home: {}", e);
      } else {
        let animations_dir = home_public.join("animations");
        let backgrounds_dir = home_public.join("backgrounds");
        
        if let Err(e) = fs::create_dir_all(&animations_dir) {
          log::error!("Failed to create animations directory: {}", e);
        }
        
        if let Err(e) = fs::create_dir_all(&backgrounds_dir) {
          log::error!("Failed to create backgrounds directory: {}", e);
        }
      }
    }
    return home_public;
  }
  
  // Fallback: check if public dir exists as a sibling to wherever the app is
  app_dir.parent()
    .map(|p| p.join("public"))
    .unwrap_or_else(|| app_dir.join("public"))
} 