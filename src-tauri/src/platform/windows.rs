use std::fs;
use std::path::Path;
use std::process::Command;

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
  
  // Check if we already have the public folder at the current location
  let public_dir = exe_dir.join("public");
  
  // If the public directory exists in the same location as the exe, we're good
  if public_dir.exists() {
    log::info!("Public folder exists at current location. No need for setup.");
    return Ok(());
  }
  
  // Get Program Files locations as potential default installation locations
  let program_files = std::env::var("ProgramFiles")
    .unwrap_or_else(|_| "C:\\Program Files".to_string());
  let program_files_x86 = std::env::var("ProgramFiles(x86)")
    .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
  
  let expected_install_dir = Path::new(&program_files).join("Anglewing");
  let expected_install_dir_x86 = Path::new(&program_files_x86).join("Anglewing");
  
  // Check if there's a public folder in the standard install locations
  let program_files_public = expected_install_dir.join("public");
  let program_files_x86_public = expected_install_dir_x86.join("public");
  
  if program_files_public.exists() || program_files_x86_public.exists() {
    log::info!("Public folder exists in Program Files. No need for setup.");
    return Ok(());
  }
  
  // If we reach here, we need to set up the public folder
  // Create it at the current executable location as the default behavior
  
  log::info!("Setting up public folder at current executable location");
  
  // Create the public directory and its subdirectories
  fs::create_dir_all(&public_dir)
    .map_err(|e| format!("Failed to create public directory: {}", e))?;
    
  fs::create_dir_all(public_dir.join("animations"))
    .map_err(|e| format!("Failed to create animations directory: {}", e))?;
    
  fs::create_dir_all(public_dir.join("backgrounds"))
    .map_err(|e| format!("Failed to create backgrounds directory: {}", e))?;
  
  // Check if we're running from a temporary location
  // We'll only suggest moving if it's a very temporary location
  
  let home_dir = match std::env::var("USERPROFILE") {
    Ok(dir) => dir,
    Err(_) => {
      log::warn!("Could not determine user's home directory.");
      return Ok(());
    }
  };
  
  let desktop_dir = Path::new(&home_dir).join("Desktop");
  let downloads_dir = Path::new(&home_dir).join("Downloads");
  let temp_dir = std::env::temp_dir();
  
  // Only suggest moving if we're in Downloads, Desktop, or the temp directory
  let is_temporary_location = exe_dir.starts_with(desktop_dir) || 
                              exe_dir.starts_with(downloads_dir) || 
                              exe_dir.starts_with(&temp_dir);
  
  // Also check for a "first-run" marker to avoid showing the move prompt on every run
  let first_run_marker = exe_dir.join(".anglewing_configured");
  let is_first_run = !first_run_marker.exists();
  
  if is_temporary_location && is_first_run {
    log::info!("App is running from a temporary location. Will offer to install properly.");
    
    // User chose a custom location - create a marker file to avoid asking again
    if !is_temporary_location {
      fs::write(&first_run_marker, "configured")
        .map_err(|e| format!("Failed to create configuration marker: {}", e))?;
      return Ok(());
    }
    
    // Create target directories in Program Files
    let target_dir = &expected_install_dir;
    if let Err(e) = fs::create_dir_all(target_dir) {
      log::warn!("Failed to create target directory in Program Files: {}. Will keep app in current location.", e);
      // Create marker file to avoid asking again
      fs::write(&first_run_marker, "configured")
        .map_err(|e| format!("Failed to create configuration marker: {}", e))?;
      return Ok(());
    }
    
    // Create the public directory structure in the target location
    let target_public_dir = target_dir.join("public");
    let target_animations_dir = target_public_dir.join("animations");
    let target_backgrounds_dir = target_public_dir.join("backgrounds");
    
    fs::create_dir_all(&target_public_dir)
      .map_err(|e| format!("Failed to create public directory: {}", e))?;
    
    fs::create_dir_all(&target_animations_dir)
      .map_err(|e| format!("Failed to create animations directory: {}", e))?;
    
    fs::create_dir_all(&target_backgrounds_dir)
      .map_err(|e| format!("Failed to create backgrounds directory: {}", e))?;
    
    // Copy the public folder we just created to the target location
    copy_directory(&public_dir, &target_public_dir)
      .map_err(|e| format!("Failed to copy public directory: {}", e))?;
    
    // Create a batch file to move the application after it's closed
    let app_name = exe_path.file_name()
      .ok_or_else(|| "Failed to get executable name".to_string())?
      .to_string_lossy()
      .to_string();
    
    let batch_script_path = std::env::temp_dir().join("anglewing_install.bat");
    
    let target_exe_path = target_dir.join(&app_name);
    let script_content = format!(
      r#"@echo off
echo Waiting for Anglewing to close...
timeout /t 2 /nobreak > nul
echo Moving Anglewing to the Program Files directory...
robocopy "{}" "{}" /E /MOVE
echo Setting up shortcuts...
powershell -Command "$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut([Environment]::GetFolderPath('Desktop') + '\Anglewing.lnk'); $Shortcut.TargetPath = '{}'; $Shortcut.Save()"
echo Starting Anglewing from new location...
start "" "{}"
echo Cleaning up...
del "%~f0"
exit
"#,
      exe_dir.to_string_lossy(),
      target_dir.to_string_lossy(),
      target_exe_path.to_string_lossy(),
      target_exe_path.to_string_lossy()
    );
    
    fs::write(&batch_script_path, script_content)
      .map_err(|e| format!("Failed to write installation script: {}", e))?;
    
    // Execute the batch script
    Command::new("cmd")
      .args(["/c", "start", "/b", batch_script_path.to_str().unwrap()])
      .spawn()
      .map_err(|e| format!("Failed to execute installation script: {}", e))?;
    
    // Exit the app so the script can move it
    app_handle.exit(0);
  } else {
    // Create marker file to avoid asking again in the future
    fs::write(&first_run_marker, "configured")
      .map_err(|e| format!("Failed to create configuration marker: {}", e))?;
  }
  
  Ok(())
}

#[allow(dead_code)]
pub fn get_public_folder_path(app_dir: &Path) -> std::path::PathBuf {
  // Priority 1: Check for a public folder at the same level as the executable
  let public_dir = app_dir.join("public");
  if public_dir.exists() {
    log::info!("Found public folder at executable location: {:?}", public_dir);
    return public_dir;
  }
  
  // Priority 2: Check in Program Files locations
  let program_files = std::env::var("ProgramFiles")
    .unwrap_or_else(|_| "C:\\Program Files".to_string());
  let program_files_x86 = std::env::var("ProgramFiles(x86)")
    .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
  
  let anglewing_dir = Path::new(&program_files).join("Anglewing");
  let anglewing_dir_x86 = Path::new(&program_files_x86).join("Anglewing");
  
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
  
  // Priority 3: Check for AppData location
  if let Ok(app_data) = std::env::var("APPDATA") {
    let app_data_public = Path::new(&app_data).join("Anglewing").join("public");
    if app_data_public.exists() {
      log::info!("Found public folder in AppData: {:?}", app_data_public);
      return app_data_public;
    }
  }
  
  // If we reach here, we need to create a public directory somewhere
  // Try to create it at the app directory level first
  log::info!("No existing public folder found. Creating one at: {:?}", public_dir);
  if let Err(e) = fs::create_dir_all(&public_dir) {
    log::error!("Failed to create public directory: {}", e);
    
    // As a fallback, try to create in AppData
    if let Ok(app_data) = std::env::var("APPDATA") {
      let fallback_dir = Path::new(&app_data).join("Anglewing").join("public");
      if let Err(e) = fs::create_dir_all(&fallback_dir) {
        log::error!("Failed to create fallback public directory in AppData: {}", e);
      } else {
        let animations_dir = fallback_dir.join("animations");
        let backgrounds_dir = fallback_dir.join("backgrounds");
        
        if let Err(e) = fs::create_dir_all(&animations_dir) {
          log::error!("Failed to create animations directory: {}", e);
        }
        
        if let Err(e) = fs::create_dir_all(&backgrounds_dir) {
          log::error!("Failed to create backgrounds directory: {}", e);
        }
        
        return fallback_dir;
      }
    }
  } else {
    let animations_dir = public_dir.join("animations");
    let backgrounds_dir = public_dir.join("backgrounds");
    
    if let Err(e) = fs::create_dir_all(&animations_dir) {
      log::error!("Failed to create animations directory: {}", e);
    }
    
    if let Err(e) = fs::create_dir_all(&backgrounds_dir) {
      log::error!("Failed to create backgrounds directory: {}", e);
    }
    
    return public_dir;
  }
  
  // If everything fails, return the expected path anyway
  public_dir
}


fn copy_directory(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
  if !dst.exists() {
    fs::create_dir_all(dst)?;
  }
  
  for entry in fs::read_dir(src)? {
    let entry = entry?;
    let file_type = entry.file_type()?;
    let src_path = entry.path();
    let dst_path = dst.join(entry.file_name());
    
    if file_type.is_dir() {
      copy_directory(&src_path, &dst_path)?;
    } else {
      fs::copy(&src_path, &dst_path)?;
    }
  }
  
  Ok(())
}
