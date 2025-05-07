use std::fs;
use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
pub fn check_and_setup_installation(app_handle: &tauri::AppHandle) -> Result<(), String> {
  log::info!("Checking if Windows installation setup is needed...");
  
  // Get the current executable path
  let exe_path = std::env::current_exe()
    .map_err(|e| format!("Failed to get executable path: {}", e))?;
  
  log::debug!("Executable path: {:?}", exe_path);
  
  // Get the directory containing the executable
  let exe_dir = exe_path.parent()
    .ok_or_else(|| "Failed to determine executable directory".to_string())?;
  
  log::debug!("Executable directory: {:?}", exe_dir);
  
  // Check if the current directory is named "Anglewing"
  let current_dir_is_anglewing = exe_dir.file_name()
    .map(|name| name == "Anglewing")
    .unwrap_or(false);
  
  // Expected installation directory is Program Files\Anglewing
  let program_files = std::env::var("ProgramFiles")
    .unwrap_or_else(|_| "C:\\Program Files".to_string());
  
  let expected_install_dir = Path::new(&program_files).join("Anglewing");
  log::debug!("Expected install directory: {:?}", expected_install_dir);
  
  // If we're already in a directory named "Anglewing" OR within Program Files\Anglewing,
  // then we're good to go - no need for installation
  if current_dir_is_anglewing || exe_dir.starts_with(&expected_install_dir) {
    log::info!("App is already properly installed. No need for setup.");
    return Ok(());
  }
  
  // If we're not in the correct structure, we'll move to Program Files\Anglewing
  log::info!("App needs to be moved to Anglewing directory");
  
  // Create the Anglewing directory if it doesn't exist
  if !expected_install_dir.exists() {
    fs::create_dir_all(&expected_install_dir)
      .map_err(|e| format!("Failed to create Anglewing directory: {}", e))?;
  }
  
  // Create a temporary batch script to move the app after it's closed
  let script_path = std::env::temp_dir().join("anglewing_install.bat");
  let app_name = exe_path.file_name()
    .ok_or_else(|| "Failed to get executable name".to_string())?
    .to_string_lossy();
  
  let target_exe_path = expected_install_dir.join(&*app_name);
  
  // Create the batch script content - only copy the executable, not creating public folders
  let script_content = format!(
    r#"@echo off
timeout /t 2 /nobreak > nul
echo Moving application to installation directory...

if not exist "{}" mkdir "{}"

rem Copy the executable and related files
robocopy "{}" "{}" /E /MOVE

rem Set permissions
icacls "{}" /grant Everyone:(OI)(CI)RX /T

rem Start the application from its new location
start "" "{}"

rem Delete this script
(goto) 2>nul & del "%~f0"
"#,
    expected_install_dir.to_string_lossy(),
    expected_install_dir.to_string_lossy(),
    exe_dir.to_string_lossy(),
    expected_install_dir.to_string_lossy(),
    expected_install_dir.to_string_lossy(),
    target_exe_path.to_string_lossy()
  );
  
  fs::write(&script_path, script_content)
    .map_err(|e| format!("Failed to write installation script: {}", e))?;
  
  // Execute the script in the background
  Command::new("cmd")
    .args(["/C", "start", "/b", script_path.to_str().unwrap()])
    .spawn()
    .map_err(|e| format!("Failed to execute installation script: {}", e))?;
  
  // Exit the app so the script can move it
  app_handle.exit(0);
  
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
  
  // Check common locations
  let program_files = std::env::var("ProgramFiles")
    .unwrap_or_else(|_| "C:\\Program Files".to_string());
  
  let expected_install_dir = Path::new(&program_files).join("Anglewing");
  let app_in_expected_location = app_dir.starts_with(&expected_install_dir);
  
  let appdata_path = std::env::var("LOCALAPPDATA")
    .unwrap_or_else(|_| "".to_string());
  
  // Try locations in this order:
  // 1. Same level as executable (already checked)
  // 2. If we're in Program Files\Anglewing, use that
  // 3. Check in AppData\Local if available
  // 4. Fallback to parent directory of executable
  
  if app_in_expected_location {
    // If we're in the expected location, the public dir is at the same level
    if !expected_install_dir.join("public").exists() {
      // Create the directory structure if it doesn't exist
      let public_dir = expected_install_dir.join("public");
      if let Err(e) = fs::create_dir_all(&public_dir) {
        log::error!("Failed to create public directory in Program Files: {}", e);
      } else {
        let animations_dir = public_dir.join("animations");
        let backgrounds_dir = public_dir.join("backgrounds");
        
        if let Err(e) = fs::create_dir_all(&animations_dir) {
          log::error!("Failed to create animations directory: {}", e);
        }
        
        if let Err(e) = fs::create_dir_all(&backgrounds_dir) {
          log::error!("Failed to create backgrounds directory: {}", e);
        }
      }
    }
    return expected_install_dir.join("public");
  } else if !appdata_path.is_empty() {
    // Try AppData\Local\Anglewing
    let appdata_public = Path::new(&appdata_path).join("Anglewing").join("public");
    if !appdata_public.exists() {
      if let Err(e) = fs::create_dir_all(&appdata_public) {
        log::error!("Failed to create public directory in AppData: {}", e);
      } else {
        let animations_dir = appdata_public.join("animations");
        let backgrounds_dir = appdata_public.join("backgrounds");
        
        if let Err(e) = fs::create_dir_all(&animations_dir) {
          log::error!("Failed to create animations directory: {}", e);
        }
        
        if let Err(e) = fs::create_dir_all(&backgrounds_dir) {
          log::error!("Failed to create backgrounds directory: {}", e);
        }
      }
    }
    return appdata_public;
  }
  
  // Fallback to the parent directory of the executable
  let parent_public = app_dir.parent()
    .map(|p| p.join("public"))
    .unwrap_or_else(|| app_dir.join("public"));
  
  // Create directory structure if it doesn't exist
  if !parent_public.exists() {
    if let Err(e) = fs::create_dir_all(&parent_public) {
      log::error!("Failed to create public directory in parent folder: {}", e);
    } else {
      let animations_dir = parent_public.join("animations");
      let backgrounds_dir = parent_public.join("backgrounds");
      
      if let Err(e) = fs::create_dir_all(&animations_dir) {
        log::error!("Failed to create animations directory: {}", e);
      }
      
      if let Err(e) = fs::create_dir_all(&backgrounds_dir) {
        log::error!("Failed to create backgrounds directory: {}", e);
      }
    }
  }
  
  parent_public
} 