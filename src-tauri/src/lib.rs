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
  
  let parent_dir = match app_dir.parent() {
    Some(dir) => {
      log::debug!("Répertoire parent du .app: {:?}", dir);
      dir.to_path_buf()
    },
    None => {
      let err = "Impossible de déterminer le dossier parent du .app".to_string();
      log::error!("{}", err);
      return Err(err);
    }
  };
  
  let public_dir = parent_dir.join("public");
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