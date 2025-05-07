use std::fs;
use std::path::Path;

pub fn check_and_setup_installation(app_handle: &tauri::AppHandle) -> Result<(), String> {
  log::info!("Checking if installation setup is needed for Windows...");
  
  // Get the current executable path
  let exe_path = std::env::current_exe()
    .map_err(|e| format!("Failed to get executable path: {}", e))?;
  
  log::debug!("Executable path: {:?}", exe_path);
  
  // Get the directory containing the executable
  let exe_dir = exe_path.parent()
    .ok_or_else(|| "Failed to determine executable directory".to_string())?;
  
  log::debug!("Executable directory: {:?}", exe_dir);
  
  // Check if we have the _up_ folder with a public directory inside
  let up_dir = if let Some(parent_dir) = exe_dir.parent() {
    parent_dir.join("_up_")
  } else {
    Path::new(exe_dir).join("_up_")
  };
  
  let up_public_dir = up_dir.join("public");
  
  if up_dir.exists() && up_public_dir.exists() {
    log::info!("Found _up_/public folder. No additional setup needed.");
    return Ok(());
  }
  
  // Check if we have a public folder directly at the executable level
  let direct_public_dir = exe_dir.join("public");
  if direct_public_dir.exists() {
    log::info!("Found public folder at executable level. No additional setup needed.");
    return Ok(());
  }
  
  // Check if there's a public folder in the standard install locations
  let program_files = std::env::var("ProgramFiles")
    .unwrap_or_else(|_| "C:\\Program Files".to_string());
  let program_files_x86 = std::env::var("ProgramFiles(x86)")
    .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
  
  let expected_install_dir = Path::new(&program_files).join("Anglewing");
  let expected_install_dir_x86 = Path::new(&program_files_x86).join("Anglewing");
  
  let program_files_up_dir = expected_install_dir.join("_up_");
  let program_files_up_public = program_files_up_dir.join("public");
  
  let program_files_x86_up_dir = expected_install_dir_x86.join("_up_");
  let program_files_x86_up_public = program_files_x86_up_dir.join("public");
  
  if (program_files_up_dir.exists() && program_files_up_public.exists()) || 
     (program_files_x86_up_dir.exists() && program_files_x86_up_public.exists()) {
    log::info!("Found _up_/public folder in Program Files. No additional setup needed.");
    return Ok(());
  }

  // Also check Program Files for direct public folder
  let program_files_public = expected_install_dir.join("public");
  let program_files_x86_public = expected_install_dir_x86.join("public");
  
  if program_files_public.exists() || program_files_x86_public.exists() {
    log::info!("Public folder exists in Program Files. No need for setup.");
    return Ok(());
  }
  
  // If we reach here, we should create a marker indicating we've checked for the _up_ folder
  // but we won't create empty directories anymore
  let first_run_marker = exe_dir.join(".anglewing_configured");
  
  if !first_run_marker.exists() {
    // Log a warning that we couldn't find the public folder
    log::warn!("Could not find _up_/public folder. Application might not work correctly without resources.");
    
    // Create marker file to avoid checking again in the future
    fs::write(&first_run_marker, "configured")
      .map_err(|e| format!("Failed to create configuration marker: {}", e))?;
  }
  
  Ok(())
}

#[allow(dead_code)]
pub fn get_public_folder_path(app_dir: &Path) -> std::path::PathBuf {
  // Priority 1: Check for _up_/public folder at parent of the executable location
  let up_dir = if let Some(parent_dir) = app_dir.parent() {
    parent_dir.join("_up_")
  } else {
    app_dir.join("_up_")
  };
  
  let up_public_dir = up_dir.join("public");
  if up_dir.exists() && up_public_dir.exists() {
    log::info!("Found _up_/public folder: {:?}", up_public_dir);
    return up_public_dir;
  }
  
  // Priority 2: Check for a direct public folder at the same level as the executable
  let direct_public_dir = app_dir.join("public");
  if direct_public_dir.exists() {
    log::info!("Found public folder at executable location: {:?}", direct_public_dir);
    return direct_public_dir;
  }
  
  // Priority 3: Check in Program Files locations for _up_/public
  let program_files = std::env::var("ProgramFiles")
    .unwrap_or_else(|_| "C:\\Program Files".to_string());
  let program_files_x86 = std::env::var("ProgramFiles(x86)")
    .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
  
  let anglewing_dir = Path::new(&program_files).join("Anglewing");
  let anglewing_dir_x86 = Path::new(&program_files_x86).join("Anglewing");
  
  let program_files_up_dir = anglewing_dir.join("_up_");
  let program_files_up_public = program_files_up_dir.join("public");
  
  if program_files_up_dir.exists() && program_files_up_public.exists() {
    log::info!("Found _up_/public folder in Program Files: {:?}", program_files_up_public);
    return program_files_up_public;
  }
  
  let program_files_x86_up_dir = anglewing_dir_x86.join("_up_");
  let program_files_x86_up_public = program_files_x86_up_dir.join("public");
  
  if program_files_x86_up_dir.exists() && program_files_x86_up_public.exists() {
    log::info!("Found _up_/public folder in Program Files (x86): {:?}", program_files_x86_up_public);
    return program_files_x86_up_public;
  }
  
  // Priority 4: Check direct public folders in Program Files
  let program_files_public = anglewing_dir.join("public");
  if program_files_public.exists() {
    log::info!("Found public folder in Program Files: {:?}", program_files_public);
    return program_files_public;
  }
  
  let program_files_x86_public = anglewing_dir_x86.join("public");
  if program_files_x86_public.exists() {
    log::info!("Found public folder in Program Files (x86): {:?}", program_files_x86_public);
    return program_files_x86_public;
  }
  
  // Priority 5: Check for AppData location
  if let Ok(app_data) = std::env::var("APPDATA") {
    // Check for _up_/public in AppData
    let app_data_up_dir = Path::new(&app_data).join("Anglewing").join("_up_");
    let app_data_up_public = app_data_up_dir.join("public");
    
    if app_data_up_dir.exists() && app_data_up_public.exists() {
      log::info!("Found _up_/public folder in AppData: {:?}", app_data_up_public);
      return app_data_up_public;
    }
    
    // Check for direct public in AppData
    let app_data_public = Path::new(&app_data).join("Anglewing").join("public");
    if app_data_public.exists() {
      log::info!("Found public folder in AppData: {:?}", app_data_public);
      return app_data_public;
    }
  }
  
  // If we haven't found anything until now, we should look for _up_/public in all parent directories
  // This is useful for development scenarios
  let mut current_dir = app_dir;
  for _ in 0..5 {  // Limit to 5 parent directories to avoid infinite loop
    if let Some(parent) = current_dir.parent() {
      let parent_up_dir = parent.join("_up_");
      let parent_up_public = parent_up_dir.join("public");
      
      if parent_up_dir.exists() && parent_up_public.exists() {
        log::info!("Found _up_/public in parent directory: {:?}", parent_up_public);
        return parent_up_public;
      }
      
      current_dir = parent;
    } else {
      break;
    }
  }
  
  // If we reach here, we couldn't find any valid public directory
  // Log a warning and return the path to where it should be (even if it doesn't exist)
  log::warn!("Could not find any valid public directory. Using default location: {:?}", up_public_dir);
  up_public_dir
}
