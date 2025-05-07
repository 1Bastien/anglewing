use std::fs;
use std::path::Path;

#[allow(dead_code)]
pub fn check_and_setup_installation(app_handle: &tauri::AppHandle) -> Result<(), String> {
  log::info!("Checking if Linux installation setup is needed...");
  
  // Get the current executable path
  let exe_path = std::env::current_exe()
    .map_err(|e| format!("Failed to get executable path: {}", e))?;
  
  log::debug!("Executable path: {:?}", exe_path);
  
  // Get the directory containing the executable
  let exe_dir = exe_path.parent()
    .ok_or_else(|| "Failed to determine executable directory".to_string())?;
  
  log::debug!("Executable directory: {:?}", exe_dir);
  
  // Expected installation locations
  let application_dir = Path::new("/usr/share/applications/Anglewing");
  let local_application_dir = dirs::home_dir()
    .map(|h| h.join(".local/share/applications/Anglewing"))
    .ok_or_else(|| "Failed to determine home directory".to_string())?;
  
  // Check if we're already in the right location
  if exe_dir.starts_with(application_dir) || exe_dir.starts_with(&local_application_dir) {
    let public_dir = exe_dir.join("resources/public");
    if public_dir.exists() {
      log::info!("App is already properly installed with public resources. No need for setup.");
      return Ok(());
    }
  }
  
  // For Linux, we'll use the local application directory for installation
  let target_dir = local_application_dir;
  
  // Create the target directory if it doesn't exist
  if !target_dir.exists() {
    fs::create_dir_all(&target_dir)
      .map_err(|e| format!("Failed to create target directory: {}", e))?;
  }
  
  // Create the public directory structure
  let target_public_dir = target_dir.join("resources/public");
  let animations_dir = target_public_dir.join("animations");
  let backgrounds_dir = target_public_dir.join("backgrounds");
  
  if !target_public_dir.exists() {
    fs::create_dir_all(&target_public_dir)
      .map_err(|e| format!("Failed to create public directory: {}", e))?;
  }
  
  if !animations_dir.exists() {
    fs::create_dir_all(&animations_dir)
      .map_err(|e| format!("Failed to create animations directory: {}", e))?;
  }
  
  if !backgrounds_dir.exists() {
    fs::create_dir_all(&backgrounds_dir)
      .map_err(|e| format!("Failed to create backgrounds directory: {}", e))?;
  }
  
  // Check if we have embedded resources in the app that we can copy
  let embedded_public_dir = exe_dir.join("resources").join("public");
  
  if embedded_public_dir.exists() {
    // Copy embedded resources
    copy_directory(&embedded_public_dir, &target_public_dir)
      .map_err(|e| format!("Failed to copy embedded resources: {}", e))?;
    log::info!("Copied embedded resources to target location");
  }
  
  // Create a desktop entry file
  let desktop_entry_path = dirs::home_dir()
    .map(|h| h.join(".local/share/applications/anglewing.desktop"))
    .ok_or_else(|| "Failed to determine desktop entry path".to_string())?;
  
  let app_name = exe_path.file_name()
    .ok_or_else(|| "Failed to get executable name".to_string())?
    .to_string_lossy();
  
  let desktop_entry_content = format!(
    r#"[Desktop Entry]
Name=Anglewing
Comment=Anglewing Application
Exec={}
Icon={}/resources/icons/128x128.png
Terminal=false
Type=Application
Categories=Utility;
"#,
    target_dir.join(&*app_name).to_string_lossy(),
    target_dir.to_string_lossy()
  );
  
  fs::write(&desktop_entry_path, desktop_entry_content)
    .map_err(|e| format!("Failed to create desktop entry: {}", e))?;
  
  // Give execute permissions to the desktop entry
  std::process::Command::new("chmod")
    .args(["+x", desktop_entry_path.to_str().unwrap()])
    .status()
    .map_err(|e| format!("Failed to set desktop entry permissions: {}", e))?;
  
  // Copy the executable to the target location
  let target_exe_path = target_dir.join(&*app_name);
  fs::copy(&exe_path, &target_exe_path)
    .map_err(|e| format!("Failed to copy executable: {}", e))?;
  
  // Give execute permissions to the executable
  std::process::Command::new("chmod")
    .args(["+x", target_exe_path.to_str().unwrap()])
    .status()
    .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
  
  // Start the application from its new location
  std::process::Command::new(&target_exe_path)
    .spawn()
    .map_err(|e| format!("Failed to start the application: {}", e))?;
  
  // Exit the app
  app_handle.exit(0);
  
  Ok(())
}

fn copy_directory(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
  if !dst.exists() {
    fs::create_dir_all(dst)?;
  }
  
  for entry_result in fs::read_dir(src)? {
    let entry = entry_result?;
    let file_type = entry.file_type()?;
    let dst_path = dst.join(entry.file_name());
    
    if file_type.is_dir() {
      copy_directory(&entry.path(), &dst_path)?;
    } else {
      fs::copy(entry.path(), dst_path)?;
    }
  }
  
  Ok(())
}

#[allow(dead_code)]
pub fn get_public_folder_path(app_dir: &Path) -> std::path::PathBuf {
  // First, check if there's a public folder in the same directory as the app
  let resources_public = app_dir.join("resources").join("public");
  if resources_public.exists() {
    return resources_public;
  }
  
  // Check standard Linux locations
  let application_public = Path::new("/usr/share/applications/Anglewing/resources/public");
  if application_public.exists() {
    return application_public.to_path_buf();
  }
  
  // Check local user application directory
  if let Some(home) = dirs::home_dir() {
    let local_public = home.join(".local/share/applications/Anglewing/resources/public");
    if local_public.exists() {
      return local_public;
    }
  }
  
  // Fallback: current directory or parent
  app_dir.parent()
    .map(|p| p.join("public"))
    .unwrap_or_else(|| app_dir.join("public"))
} 