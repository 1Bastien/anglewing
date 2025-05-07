use std::path::Path;

#[allow(dead_code)]
pub fn check_and_setup_installation(_app_handle: &tauri::AppHandle) -> Result<(), String> {
  // No special installation required for Linux
  Ok(())
}

#[allow(dead_code)]
pub fn get_public_folder_path(app_dir: &Path) -> std::path::PathBuf {
  // Priority 1: Check for _up_/public at the same level as the app
  let up_dir = app_dir.join("_up_");
  let up_public_dir = up_dir.join("public");
  if up_dir.exists() && up_public_dir.exists() {
    return up_public_dir;
  }
  
  // Priority 2: Check for _up_/public in the parent directory
  if let Some(parent_dir) = app_dir.parent() {
    let parent_up_dir = parent_dir.join("_up_");
    let parent_up_public_dir = parent_up_dir.join("public");
    if parent_up_dir.exists() && parent_up_public_dir.exists() {
      return parent_up_public_dir;
    }
  }
  
  // Priority 3: Check standard Linux installation directories for _up_/public
  let usr_anglewing_path = Path::new("/usr/Anglewing");
  let usr_anglewing_up_dir = usr_anglewing_path.join("_up_");
  let usr_anglewing_up_public = usr_anglewing_up_dir.join("public");
  if usr_anglewing_up_dir.exists() && usr_anglewing_up_public.exists() {
    return usr_anglewing_up_public;
  }
  
  let opt_anglewing_path = Path::new("/opt/Anglewing");
  let opt_anglewing_up_dir = opt_anglewing_path.join("_up_");
  let opt_anglewing_up_public = opt_anglewing_up_dir.join("public");
  if opt_anglewing_up_dir.exists() && opt_anglewing_up_public.exists() {
    return opt_anglewing_up_public;
  }
  
  let usr_share_path = Path::new("/usr/share/Anglewing");
  let usr_share_up_dir = usr_share_path.join("_up_");
  let usr_share_up_public = usr_share_up_dir.join("public");
  if usr_share_up_dir.exists() && usr_share_up_public.exists() {
    return usr_share_up_public;
  }
  
  // Priority 4: Check for direct public folders (fallback)
  let same_level_public = app_dir.join("public");
  if same_level_public.exists() {
    return same_level_public;
  }
  
  if let Some(parent_dir) = app_dir.parent() {
    let parent_public = parent_dir.join("public");
    if parent_public.exists() {
      return parent_public;
    }
  }
  
  // Check standard locations for direct public folder
  let usr_anglewing_public = usr_anglewing_path.join("public");
  if usr_anglewing_public.exists() {
    return usr_anglewing_public;
  }
  
  let opt_anglewing_public = opt_anglewing_path.join("public");
  if opt_anglewing_public.exists() {
    return opt_anglewing_public;
  }
  
  let usr_share_public = usr_share_path.join("public");
  if usr_share_public.exists() {
    return usr_share_public;
  }
  
  // Fallback: Return the expected path for _up_/public at the executable level
  // This will likely fail but is consistent with Windows implementation
  up_public_dir
} 