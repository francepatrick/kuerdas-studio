#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
use tauri::Manager;
fn main() {
    let engine = std::sync::Arc::new(std::sync::Mutex::new(kuerdas_audio::Engine::default()));
    let watchdog = engine.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if let Ok(mut engine) = watchdog.lock() {
            engine.check_connection();
        }
    });
    let shutdown = engine.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(engine)
        .setup(move |app| {
            let library = kuerdas_audio::WorkspaceCatalog::new(app.path().app_data_dir()?)?;
            app.manage(std::sync::Arc::new(std::sync::Mutex::new(library)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_devices,
            commands::start_monitoring,
            commands::stop_monitoring,
            commands::engine_status,
            commands::session_state,
            commands::set_parameters,
            commands::start_recording,
            commands::play_transport,
            commands::stop_transport,
            commands::seek_transport,
            commands::set_loop,
            commands::import_backing,
            commands::update_track,
            commands::remove_track,
            commands::export_audio,
            commands::import_audio_paths,
            commands::create_track,
            commands::rename_track,
            commands::rename_clip,
            commands::move_clip,
            commands::split_clip,
            commands::combine_tracks,
            commands::workspace_list,
            commands::workspace_create,
            commands::workspace_open,
            commands::workspace_rename,
            commands::workspace_close,
            commands::process_recorded_track
        ])
        .build(tauri::generate_context!())
        .expect("Unable to launch Kuerdas Studio")
        .run(move |_, event| {
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) {
                if let Ok(mut engine) = shutdown.lock() {
                    engine.stop();
                }
            }
        });
}
