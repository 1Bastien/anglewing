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
  
  // Expected installation directory is Program Files\Anglewing
  let program_files = std::env::var("ProgramFiles")
    .unwrap_or_else(|_| "C:\\Program Files".to_string());
  
  let expected_install_dir = Path::new(&program_files).join("Anglewing");
  log::debug!("Expected install directory: {:?}", expected_install_dir);
  
  // Check if we're already in the right location
  let public_dir = exe_dir.parent()
    .map(|p| p.join("public"))
    .ok_or_else(|| "Failed to determine public folder path".to_string())?;
  
  if exe_dir.starts_with(&expected_install_dir) && public_dir.exists() {
    log::info!("App is already properly installed. No need for setup.");
    return Ok(());
  }
  
  // If we need to set up the installation
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
  let program_files = std::env::var("ProgramFiles")
    .unwrap_or_else(|_| "C:\\Program Files".to_string());
  
  let expected_install_dir = Path::new(&program_files).join("Anglewing");
  let app_in_expected_location = app_dir.starts_with(&expected_install_dir);
  
  if app_in_expected_location {
    // If we're in the expected location, the public dir is a sibling to the exe
    expected_install_dir.join("public")
  } else {
    // Fallback: check if public dir exists as a sibling to wherever the app is
    app_dir.parent()
      .map(|p| p.join("public"))
      .unwrap_or_else(|| app_dir.join("public"))
  }
} 