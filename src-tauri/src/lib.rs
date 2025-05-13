mod platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

  tauri::Builder::default()
    .plugin(tauri_plugin_process::init())
    .setup(|app| {
      if let Err(e) = platform::check_and_setup_installation(app.handle()) {
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
      .map_err(|e| format!("Failed to get executable path: {}", e))?;
  
  log::info!("Executable path: {:?}", exe_path);
  
  let exe_dir = match exe_path.parent() {
    Some(dir) => dir.to_path_buf(),
    None => {
      log::error!("Failed to get executable parent directory");
      return Err("Failed to get parent directory of the executable".to_string());
    }
  };
  
  log::info!("Executable directory: {:?}", exe_dir);
  
  #[cfg(target_os = "macos")]
  let app_dir = exe_dir
    .parent()
    .and_then(|p| p.parent())
    .map(|p| p.to_path_buf())
    .unwrap_or(exe_dir.clone());
  
  #[cfg(target_os = "linux")]
  let app_dir = exe_dir.clone();
  
  #[cfg(not(any(target_os = "macos", target_os = "linux")))]
  let app_dir = exe_dir.clone();

  #[cfg(target_os = "macos")]
  let public_path = platform::macos::get_public_folder_path(&app_dir);
  
  #[cfg(target_os = "linux")]
  let public_path = platform::linux::get_public_folder_path(&app_dir);
  
  #[cfg(not(any(target_os = "macos", target_os = "linux")))]
  let public_path = platform::linux::get_public_folder_path(&app_dir);
  
  Ok(public_path.to_string_lossy().to_string())
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

  #[cfg(not(any(target_os = "macos", target_os = "linux")))]
  {
    Err("System sleep not supported on this platform".to_string())
  }
}

#[tauri::command]
async fn reset_inactivity_timer() -> Result<(), String> {
  Ok(())
} 