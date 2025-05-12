mod platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

  tauri::Builder::default()
    .plugin(tauri_plugin_process::init())
    .setup(|app| {
      // Check if we need to perform the initial setup
      if let Err(e) = platform::check_and_setup_installation(app.handle()) {
        // We still keep this error log as it's critical for installation
        log::error!("Failed to setup installation: {}", e);
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

#[tauri::command]
fn close_application(app_handle: tauri::AppHandle) {
  app_handle.exit(0);
}

#[tauri::command]
fn get_public_folder_path() -> Result<String, String> {
  let exe_path = std::env::current_exe()
      .map_err(|e| format!("Impossible d'obtenir le chemin de l'exécutable: {}", e))?;
  
  log::info!("Executable path: {:?}", exe_path);
  
  let exe_dir = match exe_path.parent() {
    Some(dir) => dir.to_path_buf(),
    None => {
      log::error!("Failed to get executable parent directory");
      return Err("Impossible d'obtenir le répertoire parent de l'exécutable".to_string());
    }
  };
  
  log::info!("Executable directory: {:?}", exe_dir);
  
  // Sur Windows, on renvoie le chemin vers _up_/public au lieu d'une chaîne vide
  #[cfg(target_os = "windows")]
  {
    // Utiliser la fonction de la plateforme Windows pour trouver le bon chemin
    let public_dir = platform::windows::get_public_folder_path(&exe_dir);
    
    // Vérifier que le dossier existe
    if !public_dir.exists() {
      return Err(format!("Le dossier public n'existe pas: {:?}", public_dir));
    }
    
    return Ok(public_dir.to_string_lossy().to_string());
  }
  
  #[cfg(target_os = "macos")]
  let app_dir = exe_dir
    .parent() // Remonter à Contents
    .and_then(|p| p.parent()) // Remonter à .app
    .map(|p| p.to_path_buf())
    .unwrap_or(exe_dir.clone());
  
  #[cfg(target_os = "linux")]
  let app_dir = exe_dir.clone();
  
  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  let app_dir = exe_dir.clone();
  
  #[cfg(not(target_os = "windows"))]
  {
    // Utiliser les fonctions spécifiques à la plateforme pour déterminer le chemin
    #[cfg(target_os = "macos")]
    let public_dir = platform::macos::get_public_folder_path(&app_dir);
    
    #[cfg(target_os = "linux")]
    let public_dir = platform::linux::get_public_folder_path(&app_dir);
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let public_dir = app_dir.parent()
      .map(|p| p.join("public"))
      .unwrap_or_else(|| app_dir.join("public"));
    
    if !public_dir.exists() {
      return Err(format!("Le dossier public n'existe pas: {:?}", public_dir));
    }
    
    if !public_dir.is_dir() {
      return Err(format!("Le chemin n'est pas un dossier: {:?}", public_dir));
    }
    
    // Check if the directory is readable
    match std::fs::read_dir(&public_dir) {
      Ok(_) => log::info!("Public directory is readable"),
      Err(e) => {
        log::error!("Public directory is not readable: {}", e);
        return Err(format!("Le dossier public n'est pas lisible: {}", e));
      }
    }
    
    let path_str = public_dir.to_string_lossy().to_string();
    log::info!("Returning public directory path: {}", path_str);
    return Ok(path_str);
  }
}

#[tauri::command]
async fn put_system_to_sleep() -> Result<(), String> {
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

  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  {
    Err("System sleep not supported on this platform".to_string())
  }
}

#[tauri::command]
async fn reset_inactivity_timer() -> Result<(), String> {
  Ok(())
} 