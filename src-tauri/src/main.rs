#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;

use std::sync::Mutex;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("impossible de résoudre app_data_dir");
            std::fs::create_dir_all(&app_dir)
                .expect("impossible de créer le répertoire de données");

            let db_path = app_dir.join("interruptlog.db");
            let conn = db::init_db(&db_path).expect("impossible d'initialiser la base SQLite");
            app.manage(Mutex::new(conn));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_people,
            commands::add_person,
            commands::update_person,
            commands::delete_person,
            commands::start_interruption,
            commands::stop_interruption,
            commands::get_today_interruptions,
            commands::get_stats_today,
            commands::export_csv,
            commands::get_active_cad_file,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application Tauri");
}
