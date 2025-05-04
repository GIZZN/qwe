// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use std::io::{self, ErrorKind};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PasswordEntry {
    name: String,
    username: String,
    password: String,
    url: Option<String>,
    notes: Option<String>,
}

struct PasswordStore {
    passwords: Mutex<HashMap<String, PasswordEntry>>,
    file_path: PathBuf,
}

impl PasswordStore {
    fn new(config: &tauri::Config) -> Result<Self, String> {
        // Создаем директорию для хранения данных
        let app_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./data"))
            .join("password-manager");
        
        // Создаем директорию, если не существует
        fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
        
        let file_path = app_dir.join("passwords.json");
        
        let passwords = match fs::read_to_string(&file_path) {
            Ok(data) => {
                // Файл существует, десериализуем данные
                match serde_json::from_str::<HashMap<String, PasswordEntry>>(&data) {
                    Ok(map) => map,
                    Err(e) => {
                        eprintln!("Failed to parse passwords file: {}", e);
                        HashMap::new()
                    }
                }
            },
            Err(e) if e.kind() == ErrorKind::NotFound => {
                // Файл не найден, создаем пустой HashMap
                HashMap::new()
            },
            Err(e) => {
                // Другая ошибка при чтении файла
                eprintln!("Error reading passwords file: {}", e);
                HashMap::new()
            }
        };
        
        Ok(Self {
            passwords: Mutex::new(passwords),
            file_path,
        })
    }
    
    fn save_to_disk(&self) -> Result<(), String> {
        let passwords = self.passwords.lock()
            .map_err(|_| "Failed to lock password store".to_string())?;
            
        let json = serde_json::to_string_pretty(&*passwords)
            .map_err(|e| format!("Failed to serialize passwords: {}", e))?;
            
        fs::write(&self.file_path, json)
            .map_err(|e| format!("Failed to write passwords to disk: {}", e))?;
            
        Ok(())
    }
}

#[tauri::command]
fn add_password(
    state: State<PasswordStore>,
    name: String,
    username: String,
    password: String,
    url: Option<String>,
    notes: Option<String>,
) -> Result<(), String> {
    let entry = PasswordEntry {
        name: name.clone(),
        username,
        password,
        url,
        notes,
    };
    
    let mut store = state.passwords.lock().map_err(|_| "Failed to lock password store".to_string())?;
    store.insert(name, entry);
    
    // Сохраняем изменения на диск
    drop(store); // Освобождаем блокировку перед вызовом save_to_disk
    state.save_to_disk()?;
    
    Ok(())
}

#[tauri::command]
fn get_passwords(state: State<PasswordStore>) -> Result<Vec<PasswordEntry>, String> {
    let store = state.passwords.lock().map_err(|_| "Failed to lock password store".to_string())?;
    Ok(store.values().cloned().collect())
}

#[tauri::command]
fn delete_password(state: State<PasswordStore>, name: String) -> Result<(), String> {
    let mut store = state.passwords.lock().map_err(|_| "Failed to lock password store".to_string())?;
    store.remove(&name);
    
    // Сохраняем изменения на диск
    drop(store); // Освобождаем блокировку перед вызовом save_to_disk
    state.save_to_disk()?;
    
    Ok(())
}

#[tauri::command]
fn generate_password(length: u8) -> String {
    let charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()";
    let length = length.max(8).min(32) as usize;
    
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);
    
    for _ in 0..length {
        let idx = rng.gen_range(0..charset.len());
        password.push(charset.chars().nth(idx).unwrap());
    }
    
    password
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();
    
    // Инициализируем хранилище паролей
    let password_store = match PasswordStore::new(&context.config()) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Failed to initialize password store: {}", e);
            // Используем временное хранилище в памяти, если не удалось создать постоянное
            PasswordStore {
                passwords: Mutex::new(HashMap::new()),
                file_path: PathBuf::new(),
            }
        }
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(password_store)
        .invoke_handler(tauri::generate_handler![
            add_password,
            get_passwords,
            delete_password,
            generate_password
        ])
        .run(context)
        .expect("error while running tauri application");
}
