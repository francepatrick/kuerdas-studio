use kuerdas_audio::{Engine, SessionView, StartConfig, Status};
use std::sync::{Arc, Mutex};
use tauri_plugin_dialog::DialogExt;
type EngineState<'a> = tauri::State<'a, Arc<Mutex<Engine>>>;
#[tauri::command]
pub fn list_devices() -> Result<Vec<kuerdas_devices::DeviceInfo>, String> {
    kuerdas_devices::enumerate()
}
#[tauri::command]
pub fn start_monitoring(engine: EngineState<'_>, config: StartConfig) -> Result<(), String> {
    engine.lock().map_err(|e| e.to_string())?.start(config)
}
#[tauri::command]
pub fn stop_monitoring(engine: EngineState<'_>) -> Result<(), String> {
    engine.lock().map_err(|e| e.to_string())?.stop_monitoring();
    Ok(())
}
#[tauri::command]
pub fn engine_status(engine: EngineState<'_>) -> Result<Status, String> {
    Ok(engine.lock().map_err(|e| e.to_string())?.status())
}
#[tauri::command]
pub fn session_state(engine: EngineState<'_>) -> Result<SessionView, String> {
    Ok(engine.lock().map_err(|e| e.to_string())?.session())
}
#[tauri::command]
pub fn set_parameters(engine: EngineState<'_>, input_trim: f32) -> Result<(), String> {
    if !input_trim.is_finite() {
        return Err("Input trim must be finite".into());
    }
    engine
        .lock()
        .map_err(|e| e.to_string())?
        .params
        .update(1.0, input_trim, 0.0, true)
}
#[tauri::command]
pub async fn start_recording(
    engine: EngineState<'_>,
    config: StartConfig,
    tone: String,
    target_track: Option<String>,
) -> Result<(), String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        engine
            .lock()
            .map_err(|e| e.to_string())?
            .record_into(config, tone, target_track)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn play_transport(engine: EngineState<'_>, config: StartConfig) -> Result<(), String> {
    engine.lock().map_err(|e| e.to_string())?.play(config)
}
#[tauri::command]
pub async fn stop_transport(engine: EngineState<'_>) -> Result<(), String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        engine.lock().map_err(|e| e.to_string())?.stop();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn seek_transport(engine: EngineState<'_>, frame: u64) -> Result<(), String> {
    engine.lock().map_err(|e| e.to_string())?.seek(frame)
}
#[tauri::command]
pub fn set_loop(engine: EngineState<'_>, enabled: bool) -> Result<(), String> {
    engine.lock().map_err(|e| e.to_string())?.set_loop(enabled)
}
#[tauri::command]
pub async fn import_backing(
    app: tauri::AppHandle,
    engine: EngineState<'_>,
    sample_rate: u32,
) -> Result<bool, String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(files) = app
            .dialog()
            .file()
            .set_title("Import audio tracks")
            .add_filter("Audio", &["wav", "mp3", "m4a", "flac", "ogg", "oga"])
            .blocking_pick_files()
        else {
            return Ok(false);
        };
        let paths = files
            .into_iter()
            .map(|file| file.into_path().map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        engine
            .lock()
            .map_err(|e| e.to_string())?
            .import_many(&paths, sample_rate)?;
        Ok(true)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn update_track(
    engine: EngineState<'_>,
    id: String,
    gain: f32,
    pan: f32,
    mute: bool,
    solo: bool,
) -> Result<(), String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        engine
            .lock()
            .map_err(|e| e.to_string())?
            .update_track(&id, gain, pan, mute, solo)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn remove_track(engine: EngineState<'_>, id: String) -> Result<(), String> {
    engine.lock().map_err(|e| e.to_string())?.remove_track(&id)
}
#[tauri::command]
pub async fn export_audio(
    app: tauri::AppHandle,
    engine: EngineState<'_>,
    id: Option<String>,
) -> Result<bool, String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let name = if id.is_some() {
            "guitar-take.wav"
        } else {
            "kuerdas-mix.wav"
        };
        let Some(file) = app
            .dialog()
            .file()
            .set_title("Export WAV")
            .set_file_name(name)
            .add_filter("WAV audio", &["wav"])
            .blocking_save_file()
        else {
            return Ok(false);
        };
        let path = file.into_path().map_err(|e| e.to_string())?;
        let engine = engine.lock().map_err(|e| e.to_string())?;
        if let Some(id) = id {
            engine.export_track(&id, &path)?;
        } else {
            engine.export_mix(&path)?;
        }
        Ok(true)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_audio_paths(
    engine: EngineState<'_>,
    paths: Vec<String>,
    sample_rate: u32,
) -> Result<(), String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let paths = paths
            .into_iter()
            .map(std::path::PathBuf::from)
            .collect::<Vec<_>>();
        engine
            .lock()
            .map_err(|e| e.to_string())?
            .import_many(&paths, sample_rate)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn create_track(
    engine: EngineState<'_>,
    name: String,
    sample_rate: u32,
) -> Result<String, String> {
    engine
        .lock()
        .map_err(|e| e.to_string())?
        .create_track(&name, sample_rate)
}
#[tauri::command]
pub fn rename_track(engine: EngineState<'_>, id: String, name: String) -> Result<(), String> {
    engine
        .lock()
        .map_err(|e| e.to_string())?
        .rename_track(&id, &name)
}
#[tauri::command]
pub fn rename_clip(engine: EngineState<'_>, id: String, name: String) -> Result<(), String> {
    engine
        .lock()
        .map_err(|e| e.to_string())?
        .rename_clip(&id, &name)
}
#[tauri::command]
pub fn move_clip(
    engine: EngineState<'_>,
    id: String,
    target_track: String,
    frame: u64,
) -> Result<(), String> {
    engine
        .lock()
        .map_err(|e| e.to_string())?
        .move_clip(&id, &target_track, frame)
}
#[tauri::command]
pub fn split_clip(engine: EngineState<'_>, id: String, frame: u64) -> Result<(), String> {
    engine
        .lock()
        .map_err(|e| e.to_string())?
        .split_clip(&id, frame)
}
#[tauri::command]
pub async fn combine_tracks(
    engine: EngineState<'_>,
    ids: Vec<String>,
    name: String,
) -> Result<String, String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        engine
            .lock()
            .map_err(|e| e.to_string())?
            .combine_tracks(&ids, &name)
    })
    .await
    .map_err(|e| e.to_string())?
}

type WorkspaceState<'a> = tauri::State<'a, Arc<Mutex<kuerdas_audio::WorkspaceCatalog>>>;
#[tauri::command]
pub fn workspace_list(library: WorkspaceState<'_>) -> Result<kuerdas_audio::WorkspaceView, String> {
    library.lock().map_err(|e| e.to_string())?.view()
}
#[tauri::command]
pub async fn workspace_create(
    engine: EngineState<'_>,
    library: WorkspaceState<'_>,
    name: String,
) -> Result<kuerdas_audio::WorkspaceView, String> {
    let engine = engine.inner().clone();
    let library = library.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut library = library.lock().map_err(|e| e.to_string())?;
        let info = library.create(&name)?;
        library.open(&mut *engine.lock().map_err(|e| e.to_string())?, &info.id)?;
        library.view()
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn workspace_open(
    engine: EngineState<'_>,
    library: WorkspaceState<'_>,
    id: String,
) -> Result<kuerdas_audio::WorkspaceView, String> {
    let engine = engine.inner().clone();
    let library = library.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut library = library.lock().map_err(|e| e.to_string())?;
        library.open(&mut *engine.lock().map_err(|e| e.to_string())?, &id)?;
        library.view()
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn workspace_rename(
    library: WorkspaceState<'_>,
    id: String,
    name: String,
) -> Result<kuerdas_audio::WorkspaceView, String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    library.rename(&id, &name)?;
    library.view()
}
#[tauri::command]
pub fn workspace_close(
    engine: EngineState<'_>,
    library: WorkspaceState<'_>,
) -> Result<kuerdas_audio::WorkspaceView, String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    library.close(&mut *engine.lock().map_err(|e| e.to_string())?)?;
    library.view()
}
#[tauri::command]
pub async fn process_recorded_track(
    engine: EngineState<'_>,
    id: String,
    settings: kuerdas_audio::RecordedEffects,
) -> Result<String, String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        engine
            .lock()
            .map_err(|e| e.to_string())?
            .process_recorded_track(&id, settings)
    })
    .await
    .map_err(|e| e.to_string())?
}
