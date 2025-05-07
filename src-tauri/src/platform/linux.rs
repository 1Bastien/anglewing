use std::path::Path;

#[allow(dead_code)]
pub fn check_and_setup_installation(_app_handle: &tauri::AppHandle) -> Result<(), String> {
  // No special installation required for Linux
  Ok(())
}

#[allow(dead_code)]
pub fn get_public_folder_path(app_dir: &Path) -> std::path::PathBuf {
  // Priority 1: Check if there's a public folder in the same directory as the app
  let same_level_public = app_dir.join("public");
  if same_level_public.exists() {
    return same_level_public;
  }
  
  // Priority 2: Check if we have resources in the app bundle (for AppImage)
  let resources_dir = app_dir.join("resources");
  if resources_dir.exists() {
    let resources_public = resources_dir.join("public");
    if resources_public.exists() {
      return resources_public;
    }
  }
  
  // Priority 3: Try parent directory (common in development environment)
  if let Some(parent_dir) = app_dir.parent() {
    let parent_public = parent_dir.join("public");
    if parent_public.exists() {
      return parent_public;
    }
    
    // Try parent/parent (two levels up)
    if let Some(grandparent) = parent_dir.parent() {
      let grandparent_public = grandparent.join("public");
      if grandparent_public.exists() {
        return grandparent_public;
      }
    }
  }
  
  // Priority 4: Check in standard Linux installation directories
  let opt_path = Path::new("/opt/Anglewing");
  if opt_path.exists() {
    let opt_public = opt_path.join("public");
    if opt_public.exists() {
      return opt_public;
    }
  }
  
  let usr_share_path = Path::new("/usr/share/anglewing");
  if usr_share_path.exists() {
    let usr_share_public = usr_share_path.join("public");
    if usr_share_public.exists() {
      return usr_share_public;
    }
  }
  
  // Priority 5: Check in user's local application directory
  if let Ok(home) = std::env::var("HOME") {
    let xdg_data_home = std::env::var("XDG_DATA_HOME")
      .unwrap_or_else(|_| format!("{}/.local/share", home));
    
    let user_app_dir = Path::new(&xdg_data_home).join("anglewing");
    if user_app_dir.exists() {
      let user_public = user_app_dir.join("public");
      if user_public.exists() {
        return user_public;
      }
    }
  }
  
  // Fallback: return the expected path at the executable level
  // This will likely fail but is consistent with other implementations
  app_dir.join("public")
} 