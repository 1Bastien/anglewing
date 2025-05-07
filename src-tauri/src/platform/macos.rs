use std::fs;
use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
pub fn check_and_setup_installation(app_handle: &tauri::AppHandle) -> Result<(), String> {
  log::info!("Checking if installation setup is needed...");
  
  // Get the current executable path
  let exe_path = std::env::current_exe()
    .map_err(|e| format!("Failed to get executable path: {}", e))?;
  
  log::debug!("Executable path: {:?}", exe_path);
  
  // Get the path to the .app bundle
  let app_bundle_path = exe_path
    .parent() // Contents/MacOS
    .and_then(|p| p.parent()) // Contents
    .and_then(|p| p.parent()) // .app
    .ok_or_else(|| "Failed to determine app bundle path".to_string())?;
  
  log::debug!("App bundle path: {:?}", app_bundle_path);
  
  // Check if this app is already in the Anglewing folder structure
  let expected_install_dir = Path::new("/Applications/Anglewing");
  let app_name = app_bundle_path.file_name()
    .ok_or_else(|| "Failed to get app name".to_string())?
    .to_string_lossy();
  
  // Check if we're already in the right location with the public folder
  let public_dir = app_bundle_path.parent()
    .map(|p| p.join("public"))
    .ok_or_else(|| "Failed to determine public folder path".to_string())?;
  
  if app_bundle_path.starts_with(expected_install_dir) && public_dir.exists() {
    log::info!("App is already properly installed. No need for setup.");
    return Ok(());
  }
  
  // If we're in /Applications but not in the Anglewing folder, we need to move
  if app_bundle_path.parent() == Some(Path::new("/Applications")) {
    log::info!("App is in /Applications but needs to be moved to the Anglewing folder");
    
    // Create the Anglewing directory if it doesn't exist
    if !expected_install_dir.exists() {
      fs::create_dir_all(expected_install_dir)
        .map_err(|e| format!("Failed to create Anglewing directory: {}", e))?;
    }
    
    // Create the public directory
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
    
    // Check if we have an embedded public folder in the .app that we can copy
    let embedded_public_dir = app_bundle_path
      .join("Contents")
      .join("Resources")
      .join("public");
    
    // Create a temporary script to move the app after it's closed
    let script_path = std::env::temp_dir()
      .join("anglewing_install_script.sh");
    
    let target_app_path = expected_install_dir.join(app_name.to_string());
    
    // Prepare the part of the script that copies the public folder
    let copy_public_command = format!(r#"
# Create required directories if they don't exist
mkdir -p "{}/animations"
mkdir -p "{}/backgrounds"

# Copy public folders and config from bundled resources 
if [ -d "{}" ]; then
  cp -R "{}/"* "{}"
  echo "Copied embedded public folder to target directory"
else
  # Fallback to copy from original project location if possible
  if [ -d "/Users/bastienbourgeat/Docaret/Workspace/Anglewing/public" ]; then
    cp -R "/Users/bastienbourgeat/Docaret/Workspace/Anglewing/public/"* "{}"
    echo "Copied public folder from project directory"
  else
    echo "Warning: Could not find source public folder"
  fi
fi
"#, 
      target_public_dir.to_string_lossy(),
      target_public_dir.to_string_lossy(),
      embedded_public_dir.to_string_lossy(),
      embedded_public_dir.to_string_lossy(),
      target_public_dir.to_string_lossy(),
      target_public_dir.to_string_lossy()
    );
    
    // Write the installation script
    let script_content = format!(r#"#!/bin/bash
# Wait for the app to exit
sleep 2
# Move the app to the Anglewing directory
mv "{}" "{}"
{}
# Set permissions
chmod -R 755 "{}"
chmod -R 755 "{}"
# Launch the app from the new location
open "{}"
# Clean up this script
rm "$0"
exit 0
"#, 
      app_bundle_path.to_string_lossy(),
      target_app_path.to_string_lossy(),
      copy_public_command,
      target_app_path.to_string_lossy(),
      target_public_dir.to_string_lossy(),
      target_app_path.to_string_lossy()
    );
    
    fs::write(&script_path, script_content)
      .map_err(|e| format!("Failed to write installation script: {}", e))?;
    
    // Make the script executable
    Command::new("chmod")
      .args(["+x", script_path.to_str().unwrap()])
      .status()
      .map_err(|e| format!("Failed to make script executable: {}", e))?;
    
    // Execute the script in the background
    Command::new("bash")
      .args(["-c", &format!("nohup '{}' >/dev/null 2>&1 &", script_path.to_string_lossy())])
      .spawn()
      .map_err(|e| format!("Failed to execute installation script: {}", e))?;
    
    // Exit the app so the script can move it
    app_handle.exit(0);
  }
  
  Ok(())
}

#[allow(dead_code)]
pub fn get_public_folder_path(app_dir: &Path) -> std::path::PathBuf {
  let expected_install_dir = std::path::Path::new("/Applications/Anglewing");
  let app_in_expected_location = app_dir.starts_with(expected_install_dir);
  
  if app_in_expected_location {
    // If we're in the expected location, the public dir is a sibling to the .app
    expected_install_dir.join("public")
  } else {
    // Fallback: check if public dir exists as a sibling to wherever the app is
    app_dir.parent()
      .map(|p| p.join("public"))
      .unwrap_or_else(|| app_dir.join("public"))
  }
} 