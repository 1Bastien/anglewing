#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

  tauri::Builder::default()
    .plugin(tauri_plugin_process::init())
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      
      #[cfg(target_os = "macos")]
      {
        // Check if we need to perform the initial setup
        if let Err(e) = check_and_setup_installation(app.handle()) {
          log::error!("Failed to setup installation: {}", e);
        }
      }
      
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      close_application, 
      put_system_to_sleep, 
      reset_inactivity_timer,
      get_public_folder_path
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

// Function to check if this is the first launch and perform the installation if needed
#[cfg(target_os = "macos")]
fn check_and_setup_installation(app_handle: &tauri::AppHandle) -> Result<(), String> {
  use std::fs;
  use std::path::Path;
  use std::process::Command;
  
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

#[tauri::command]
fn close_application(app_handle: tauri::AppHandle) {
  app_handle.exit(0);
}

#[tauri::command]
fn get_public_folder_path() -> Result<String, String> {
  log::info!("Récupération du chemin du dossier public");
  
  let exe_path = std::env::current_exe()
      .map_err(|e| {
        let err = format!("Impossible d'obtenir le chemin de l'exécutable: {}", e);
        log::error!("{}", err);
        err
      })?;
  
  log::debug!("Chemin de l'exécutable: {:?}", exe_path);
  
  let exe_dir = match exe_path.parent() {
    Some(dir) => dir.to_path_buf(),
    None => {
      let err = "Impossible d'obtenir le répertoire parent de l'exécutable".to_string();
      log::error!("{}", err);
      return Err(err);
    }
  };
  
  log::debug!("Répertoire de l'exécutable: {:?}", exe_dir);
  
  #[cfg(target_os = "macos")]
  let app_dir = exe_dir
    .parent() // Remonter à Contents
    .and_then(|p| p.parent()) // Remonter à .app
    .map(|p| p.to_path_buf());
  
  #[cfg(not(target_os = "macos"))]
  let app_dir = Some(exe_dir);
  
  let app_dir = match app_dir {
    Some(dir) => {
      log::debug!("Répertoire du bundle .app: {:?}", dir);
      dir
    },
    None => {
      let err = "Impossible de déterminer le dossier parent de l'application".to_string();
      log::error!("{}", err);
      return Err(err);
    }
  };
  
  // First try to look for the public directory in the expected installation structure
  // (next to the .app in /Applications/Anglewing/)
  #[cfg(target_os = "macos")]
  let expected_install_dir = std::path::Path::new("/Applications/Anglewing");
  #[cfg(target_os = "macos")]
  let app_in_expected_location = app_dir.starts_with(expected_install_dir);
  
  #[cfg(target_os = "macos")]
  let public_dir = if app_in_expected_location {
    // If we're in the expected location, the public dir is a sibling to the .app
    expected_install_dir.join("public")
  } else {
    // Fallback: check if public dir exists as a sibling to wherever the app is
    app_dir.parent()
      .map(|p| p.join("public"))
      .unwrap_or_else(|| app_dir.join("public"))
  };

  #[cfg(not(target_os = "macos"))]
  let public_dir = app_dir.parent()
    .map(|p| p.join("public"))
    .unwrap_or_else(|| app_dir.join("public"));
  
  log::info!("Dossier public: {:?}", public_dir);
  
  if !public_dir.exists() {
    let err = format!("Le dossier public n'existe pas: {:?}", public_dir);
    log::error!("{}", err);
    return Err(err);
  }
  
  if !public_dir.is_dir() {
    let err = format!("Le chemin n'est pas un dossier: {:?}", public_dir);
    log::error!("{}", err);
    return Err(err);
  }
  
  if let Ok(entries) = std::fs::read_dir(&public_dir) {
    log::info!("Contenu du dossier public:");
    for entry in entries {
      if let Ok(entry) = entry {
        log::info!("  - {:?}", entry.path());
      }
    }
  } else {
    log::error!("Impossible de lire le contenu du dossier public");
  }
  
  Ok(public_dir.to_string_lossy().to_string())
}

#[tauri::command]
async fn put_system_to_sleep() -> Result<(), String> {
  #[cfg(target_os = "macos")]
  {
    match std::process::Command::new("pmset")
      .args(["sleepnow"])
      .status()
    {
      Ok(_) => Ok(()),
      Err(e) => Err(format!("Failed to put macOS to sleep: {}", e)),
    }
  }

  #[cfg(target_os = "windows")]
  {
    match std::process::Command::new("rundll32.exe")
      .args(["powrprof.dll,SetSuspendState", "0,1,0"])
      .status()
    {
      Ok(_) => Ok(()),
      Err(e) => Err(format!("Failed to put Windows to sleep: {}", e)),
    }
  }

  #[cfg(target_os = "linux")]
  {
    let systemd_result = std::process::Command::new("systemctl")
      .args(["suspend"])
      .status();
    
    if systemd_result.is_err() {
      match std::process::Command::new("dbus-send")
        .args(["--system", "--print-reply", "--dest=org.freedesktop.login1", 
              "/org/freedesktop/login1", "org.freedesktop.login1.Manager.Suspend", "boolean:true"])
        .status()
      {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to put Linux to sleep: {}", e)),
      }
    } else {
      Ok(())
    }
  }

  #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
  {
    Err("System sleep not supported on this platform".to_string())
  }
}

#[tauri::command]
async fn reset_inactivity_timer() -> Result<(), String> {
  Ok(())
} 