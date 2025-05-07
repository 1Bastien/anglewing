mod platform;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;

// Variable globale pour le chemin du fichier de log
static DEBUG_LOG_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

// Fonction d'utilitaire pour écrire dans le fichier log
fn write_to_debug_log(message: &str) {
  if let Some(log_path) = DEBUG_LOG_PATH.lock().unwrap().as_ref() {
    if let Ok(mut file) = OpenOptions::new()
      .create(true)
      .append(true)
      .open(log_path) 
    {
      let _ = writeln!(file, "{}", message);
    }
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

  tauri::Builder::default()
    .plugin(tauri_plugin_process::init())
    .setup(|app| {
      // Configuration de la journalisation pour Windows
      if cfg!(target_os = "windows") {
        // Pour Windows, on active la journalisation détaillée
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Debug)
            .build(),
        )?;
        
        // Créer un fichier de log à côté de l'exécutable
        let exe_dir = std::env::current_exe()
          .map_err(|e| format!("Failed to get executable path: {}", e))
          .unwrap()
          .parent()
          .map(|p| p.to_path_buf())
          .unwrap_or_default();
          
        let log_path = exe_dir.join("anglewing_debug.log");
        println!("Log file will be created at: {:?}", log_path);
        
        // Initialiser la variable globale avec le chemin du fichier
        *DEBUG_LOG_PATH.lock().unwrap() = Some(log_path.clone());
        
        // Écrire un message au démarrage
        write_to_debug_log(&format!("---- APPLICATION STARTED ----"));
        write_to_debug_log(&format!("Executable dir: {:?}", exe_dir));
        
      } else {
        // Pour les autres plateformes, on garde la configuration simple
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      
      // Check if we need to perform the initial setup
      if let Err(e) = platform::check_and_setup_installation(app.handle()) {
        log::error!("Failed to setup installation: {}", e);
        write_to_debug_log(&format!("Failed to setup installation: {}", e));
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
  write_to_debug_log(&format!("======= APPEL DEPUIS LE FRONTEND - get_public_folder_path ======="));
  
  let exe_path = std::env::current_exe()
      .map_err(|e| {
        let err = format!("Impossible d'obtenir le chemin de l'exécutable: {}", e);
        log::error!("{}", err);
        write_to_debug_log(&format!("ERROR: {}", err));
        err
      })?;
  
  log::debug!("Chemin de l'exécutable: {:?}", exe_path);
  write_to_debug_log(&format!("Chemin de l'exécutable: {:?}", exe_path));
  
  let exe_dir = match exe_path.parent() {
    Some(dir) => dir.to_path_buf(),
    None => {
      let err = "Impossible d'obtenir le répertoire parent de l'exécutable".to_string();
      log::error!("{}", err);
      write_to_debug_log(&format!("ERROR: {}", err));
      return Err(err);
    }
  };
  
  log::debug!("Répertoire de l'exécutable: {:?}", exe_dir);
  write_to_debug_log(&format!("Répertoire de l'exécutable: {:?}", exe_dir));
  
  // Sur Windows, on renvoie simplement une chaîne vide car on utilisera le système de ressources de Tauri
  #[cfg(target_os = "windows")]
  {
    let result = "".to_string();
    write_to_debug_log(&format!("Windows: utilisation du système de ressources de Tauri"));
    write_to_debug_log(&format!("Returning empty string for Windows"));
    return Ok(result);
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
    log::debug!("Répertoire de l'application: {:?}", app_dir);
    write_to_debug_log(&format!("Répertoire de l'application: {:?}", app_dir));
    
    // Utiliser les fonctions spécifiques à la plateforme pour déterminer le chemin
    #[cfg(target_os = "macos")]
    let public_dir = platform::macos::get_public_folder_path(&app_dir);
    
    #[cfg(target_os = "linux")]
    let public_dir = platform::linux::get_public_folder_path(&app_dir);
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let public_dir = app_dir.parent()
      .map(|p| p.join("public"))
      .unwrap_or_else(|| app_dir.join("public"));
    
    log::info!("Dossier public: {:?}", public_dir);
    write_to_debug_log(&format!("Dossier public: {:?}", public_dir));
    
    if !public_dir.exists() {
      let err = format!("Le dossier public n'existe pas: {:?}", public_dir);
      log::error!("{}", err);
      write_to_debug_log(&format!("ERROR: {}", err));
      return Err(err);
    }
    
    if !public_dir.is_dir() {
      let err = format!("Le chemin n'est pas un dossier: {:?}", public_dir);
      log::error!("{}", err);
      write_to_debug_log(&format!("ERROR: {}", err));
      return Err(err);
    }
    
    if let Ok(entries) = std::fs::read_dir(&public_dir) {
      log::info!("Contenu du dossier public:");
      write_to_debug_log("Contenu du dossier public:");
      for entry in entries {
        if let Ok(entry) = entry {
          log::info!("  - {:?}", entry.path());
          write_to_debug_log(&format!("  - {:?}", entry.path()));
        }
      }
    } else {
      log::error!("Impossible de lire le contenu du dossier public");
      write_to_debug_log("ERROR: Impossible de lire le contenu du dossier public");
    }
    
    let result = public_dir.to_string_lossy().to_string();
    write_to_debug_log(&format!("Returning public folder path: {}", result));
    write_to_debug_log(&format!("Le composant Home va utiliser ce chemin: {} (veuillez vérifier la console frontend)", result));
    write_to_debug_log(&format!("======= FIN APPEL DEPUIS LE FRONTEND ======="));
    return Ok(result);
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