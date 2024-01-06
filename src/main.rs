// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    process::Command,
    sync::Mutex,
};
use tauri::{Manager, State};

#[derive(Default, Serialize, Deserialize)]
struct AppState {
    setupmgr: Mutex<SetupManager>,
}
fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_setups,
            new_setup,
            remove_setup,
            load_setup,
            edit_setup_name,
            set_config,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct Setup {
    name: String,
    mods: Vec<Mod>,
}

impl Setup {
    fn new(name: String, mods: Vec<Mod>) -> Self {
        Self { name, mods }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Mod {
    name: String,
}

#[tauri::command]
fn get_setups(state: State<AppState>) -> Result<Vec<Setup>, String> {
    let mut setupmgr = state.setupmgr.lock().unwrap();
    setupmgr.update();
    let setups = setupmgr.get_setups()?.into_values().collect();
    Ok(setups)
}

#[tauri::command]
fn new_setup(state: State<AppState>) -> Result<Setup, String> {
    let mut setupmgr = state.setupmgr.lock().unwrap();
    let name = if setupmgr.setups.contains_key("New Setup") {
        let mut key = "New Setup (1)".to_string();
        let mut i = 0;
        while setupmgr.setups.contains_key(&key) {
            i += 1;
            key = format!("New Setup ({})", i)
        }
        key
    } else {
        "New Setup".to_string()
    };
    let setup = setupmgr
        .create_setup(name)
        .map_err(|_| "Couldn't create setup!".to_string())?;
    Ok(setup)
}

#[tauri::command]
fn remove_setup(state: State<AppState>, name: String) -> Result<(), String> {
    let mut setupmgr = state.setupmgr.lock().unwrap();
    setupmgr.remove_setup(&name).unwrap();
    Ok(())
}

#[tauri::command]
fn edit_setup_name(
    state: State<AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let mut setupmgr = state.setupmgr.lock().unwrap();
    let setup = match setupmgr.get_setup(&old_name) {
        Ok(setup) => setup.name.clone(),
        Err(_) => return Err("Couldn't find setup".to_string()),
    };
    setupmgr.edit_setup(&setup, new_name).unwrap();
    Ok(())
}

#[tauri::command]
fn set_config(state: State<AppState>, config: HashMap<String, String>) -> Result<(), String> {
    if let Some(path) = config.get("path") {
        let mut setupmgr = state.setupmgr.lock().unwrap();
        setupmgr.path = path.to_owned();
        setupmgr.update();
    }
    if let Some(windows_copy) = config.get("windows_copy") {
        let mut setupmgr = state.setupmgr.lock().unwrap();
        if windows_copy == "true" {
            setupmgr.windows_copy = true
        }
        if windows_copy == "false" {
            setupmgr.windows_copy = false
        }
    };
    Ok(())
}

#[tauri::command]
fn load_setup(state: State<AppState>, name: String) -> Result<(), String> {
    let setupmgr = state.setupmgr.lock().unwrap();
    let setup = match setupmgr.get_setup(&name) {
        Ok(setup) => setup,
        Err(_) => return Err("Couldn't find setup!".to_string()),
    };
    setupmgr.load_setup(setup)
}

#[derive(Default, Serialize, Deserialize)]
struct SetupManager {
    path: String,
    windows_copy: bool,
    setups: HashMap<String, Setup>,
}
impl SetupManager {
    fn get_dirs(path: &Path) -> Vec<String> {
        let read_result = fs::read_dir(path);
        if read_result.is_err() {
            return Vec::new();
        };
        read_result
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect()
    }
    fn modmgr_dir(&self) -> Result<String, String> {
        let path = Path::new(&self.path);
        let game_dirs = SetupManager::get_dirs(path);
        if game_dirs.contains(&"BepInEx".to_string()) {
            let bepinex_path = path.join("BepInEx");
            let bepinex_dirs = SetupManager::get_dirs(&bepinex_path);
            if bepinex_dirs.contains(&"ModMgr".to_string()) {
                let modmgr_path = bepinex_path.join("ModMgr");
                return Ok(modmgr_path.to_str().unwrap().to_string());
            } else {
                fs::create_dir(bepinex_path.join("ModMgr")).map_err(|err| {
                    "Couldn't create ModMgr folder! ".to_string() + &err.to_string()
                })?;
                let modmgr_path = bepinex_path.join("ModMgr");
                return Ok(modmgr_path.to_str().unwrap().to_string());
            }
        } else {
            Err("BepInEx not found!".to_string())
        }
    }
    fn plugin_dir(&self) -> Result<String, String> {
        let path = Path::new(&self.path);
        let game_dir = SetupManager::get_dirs(path);
        if game_dir.contains(&"BepInEx".to_string()) {
            let bepinex_path = path.join("BepInEx");
            let bepinex_dirs = SetupManager::get_dirs(&bepinex_path);
            if bepinex_dirs.contains(&"plugins".to_string()) {
                let plugin_dir = bepinex_path.join("plugins");
                return Ok(plugin_dir.to_str().unwrap().to_string());
            } else {
                Err("plugins not found!".to_string())
            }
        } else {
            Err("BepInEx not found!".to_string())
        }
    }
    fn get_setups(&self) -> Result<HashMap<String, Setup>, String> {
        let modmgr = self.modmgr_dir()?;
        let setups_path = Path::new(&modmgr);
        let mut setups = HashMap::new();
        for setup in SetupManager::get_dirs(setups_path) {
            let setup_dir = Path::new(&setups_path).join(&setup);
            let mods = SetupManager::get_dirs(&setup_dir)
                .into_iter()
                .map(|entry| Mod {
                    name: entry.to_string(),
                })
                .collect();
            let setup = Setup::new(setup, mods);
            setups.insert(setup.name.clone(), setup);
        }
        Ok(setups)
    }
    fn update(&mut self) {
        let setups = self.get_setups().unwrap_or_default();
        self.setups = setups
    }
    fn get_setup(&self, name: &String) -> Result<&Setup, ()> {
        let modmgr = self.modmgr_dir().unwrap();
        let setup = self.setups.get(name).unwrap();
        let setup_path = Path::new(&modmgr).join(&setup.name);
        if !setup_path.exists() {
            return Err(());
        }
        Ok(setup)
    }
    fn get_setup_path(&self, setup: &Setup) -> Result<String, String> {
        let modmgr = self.modmgr_dir()?;
        let name = &setup.name;
        let setup_path = Path::new(&modmgr).join(name);
        return Ok(setup_path.to_str().unwrap().to_string());
    }
    fn create_setup(&mut self, name: String) -> Result<Setup, String> {
        let modmgr = self.modmgr_dir()?;
        let path = Path::new(&modmgr).join(&name);
        if path.exists() {
            return Err(format!("{:?} already exists!", path));
        }
        if fs::create_dir(&path).is_err() {
            return Err(format!("Error creating dir at {:?}", path));
        }
        self.update();
        let mods = Vec::new();
        let setup = Setup::new(name, mods);
        Ok(setup)
    }
    fn edit_setup(&mut self, setup: &String, name: String) -> Result<(), String> {
        let setup = self.get_setup(setup).unwrap();
        let setup_path = self.get_setup_path(setup)?;
        let path = Path::new(&setup_path).with_file_name(name);
        fs::rename(setup_path, path).unwrap();
        self.update();
        Ok(())
    }
    fn load_setup(&self, setup: &Setup) -> Result<(), String> {
        let plugins_dir = self.plugin_dir()?;
        let setup_path = self.get_setup_path(setup)?;
        let plugins_old = plugins_dir.clone() + ".old";
        if self.windows_copy {
            Command::new("cmd")
                .args(["/C", "copy", "/Y", &plugins_dir, &plugins_old])
                .output()
                .map_err(|err| "Backup failed! ".to_string() + &err.to_string())
                .map(|_| ())?;
            Command::new("cmd")
                .args(["/C", "copy", "/Y", &setup_path, &plugins_dir])
                .output()
                .map_err(|err| "Loading failed! ".to_string() + &err.to_string())
                .map(|_| ())?;
            Ok(())
        } else {
            fs::remove_dir_all(&plugins_old)
                .map_err(|err| "Failed to remove Backup! ".to_string() + &err.to_string())?;
            fs::rename(&plugins_dir, plugins_old)
                .map_err(|err| "Backup failed! ".to_string() + &err.to_string())?;
            fs::rename(setup_path, plugins_dir)
                .map_err(|err| "Loading failed! ".to_string() + &err.to_string())?;
            Ok(())
        }
    }
    fn remove_setup(&mut self, setup: &String) -> Result<(), String> {
        let setup = self.get_setup(setup).map_err(|_| "Couldn't find setup!")?;
        let setup_path = self.get_setup_path(setup)?;
        fs::remove_dir_all(setup_path).map_err(|err| err.to_string())?;
        self.update();
        Ok(())
    }
}
