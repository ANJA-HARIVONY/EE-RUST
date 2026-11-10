mod commands;
mod config;
mod db;
mod domain;
mod error;
mod repo;
mod service;

#[cfg(test)]
mod tests;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let cfg = config::AppConfig::load()?;
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let path = dir.join(&cfg.database_filename);
            app.manage(db::Db::open(&path, cfg)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_bootstrap,
            commands::settings_save,
            commands::low_stock_count,
            commands::clients_list,
            commands::clients_save,
            commands::roles_list,
            commands::employees_list,
            commands::employees_save,
            commands::employees_set_active,
            commands::services_list,
            commands::services_save,
            commands::services_set_active,
            commands::products_list,
            commands::products_save,
            commands::products_set_active,
            commands::stock_adjust,
            commands::stock_history,
            commands::appointments_list,
            commands::appointments_save,
            commands::appointments_set_status,
            commands::invoices_list,
            commands::invoices_get,
            commands::invoices_create,
            commands::invoices_add_line,
            commands::invoices_remove_line,
            commands::invoices_set_notes,
            commands::invoices_issue,
            commands::invoices_mark_paid,
            commands::invoices_cancel,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de EE Salon");
}
