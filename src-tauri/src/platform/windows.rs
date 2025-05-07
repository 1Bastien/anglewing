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
  
  // Check if there's a "public" folder at the same level as the executable
  let public_dir = exe_dir.join("public");
  let public_dir_exists = public_dir.exists();
  
  // If we're already in a directory named "Anglewing" AND we have a public directory,
  // then we're good to go - no need for installation
  if current_dir_is_anglewing && public_dir_exists {
    log::info!("App is already properly installed. No need for setup.");
    return Ok(());
  }
  
  // If we're not in the correct structure, we'll move to Program Files\Anglewing
  log::info!("App needs to be moved to Anglewing directory with public folder");
  
  // Expected installation directory is Program Files\Anglewing
  let program_files = std::env::var("ProgramFiles")
    .unwrap_or_else(|_| "C:\\Program Files".to_string());
  
  let expected_install_dir = Path::new(&program_files).join("Anglewing");
  log::debug!("Expected install directory: {:?}", expected_install_dir);
  
  // Create the Anglewing directory if it doesn't exist
  if !expected_install_dir.exists() {
    fs::create_dir_all(&expected_install_dir)
      .map_err(|e| format!("Failed to create Anglewing directory: {}", e))?;
  }
  
  // Create the public directory and subdirectories
  let target_public_dir = expected_install_dir.join("public");
  if !target_public_dir.exists() {
    fs::create_dir_all(&target_public_dir)
      .map_err(|e| format!("Failed to create public directory: {}", e))?;
  }
  
  // Create subdirectories for public
  let animations_dir = target_public_dir.join("animations");
  let backgrounds_dir = target_public_dir.join("backgrounds");
  
  if !animations_dir.exists() {
    fs::create_dir_all(&animations_dir)
      .map_err(|e| format!("Failed to create animations directory: {}", e))?;
  }
  
  if !backgrounds_dir.exists() {
    fs::create_dir_all(&backgrounds_dir)
      .map_err(|e| format!("Failed to create backgrounds directory: {}", e))?;
  }
  
  // Check if we have an embedded public folder in the app that we can copy
  let embedded_public_dir = exe_dir.join("resources").join("public");
  
  // Create a temporary batch script to move the app after it's closed
  let script_path = std::env::temp_dir().join("anglewing_install.bat");
  let app_name = exe_path.file_name()
    .ok_or_else(|| "Failed to get executable name".to_string())?
    .to_string_lossy();
  
  let target_exe_path = expected_install_dir.join(&*app_name);
  
  // Create the batch script content
  let script_content = format!(
    r#"@echo off
timeout /t 2 /nobreak > nul
echo Moving application to installation directory...

if not exist "{}" mkdir "{}"

rem Copy the executable and related files
robocopy "{}" "{}" /E /MOVE

rem Copy public folder from resources if it exists
if exist "{}" (
  echo Copying embedded public folder...
  robocopy "{}" "{}" /E
) else (
  echo Warning: Could not find embedded public folder.
)

rem Set permissions
icacls "{}" /grant Everyone:(OI)(CI)RX /T
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
    embedded_public_dir.to_string_lossy(),
    embedded_public_dir.to_string_lossy(),
    target_public_dir.to_string_lossy(),
    expected_install_dir.to_string_lossy(),
    target_public_dir.to_string_lossy(),
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
  
  // Fallback: check if the app is in Program Files\Anglewing
  let program_files = std::env::var("ProgramFiles")
    .unwrap_or_else(|_| "C:\\Program Files".to_string());
  
  let expected_install_dir = Path::new(&program_files).join("Anglewing");
  let app_in_expected_location = app_dir.starts_with(&expected_install_dir);
  
  if app_in_expected_location {
    // If we're in the expected location, try the public dir at that level
    expected_install_dir.join("public")
  } else {
    // Fallback: check if public dir exists as a sibling to wherever the app is
    app_dir.parent()
      .map(|p| p.join("public"))
      .unwrap_or_else(|| app_dir.join("public"))
  }
} 