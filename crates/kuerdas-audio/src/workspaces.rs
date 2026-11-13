use super::*;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    #[serde(default)]
    pub legacy: bool,
    #[serde(default)]
    pub requires_name: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceView {
    pub items: Vec<WorkspaceInfo>,
    pub active: Option<WorkspaceInfo>,
}
pub struct WorkspaceCatalog {
    root: PathBuf,
    active: Option<WorkspaceInfo>,
}
fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn valid_name(value: &str) -> Result<String, String> {
    let name = value.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err("Enter a workspace name (1–80 characters)".into());
    }
    Ok(name.into())
}
fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.bytes().all(|c| c.is_ascii_hexdigit())
}
fn metadata(path: &Path, info: &WorkspaceInfo) -> Result<(), String> {
    use std::io::Write;
    let pending = path.join("workspace.json.tmp");
    let mut file = std::fs::File::create(&pending).map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(info).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    std::fs::rename(pending, path.join("workspace.json")).map_err(|e| e.to_string())
}
impl WorkspaceCatalog {
    pub fn new(app_data: PathBuf) -> Result<Self, String> {
        let root = app_data.join("Workspaces");
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let catalog = Self { root, active: None };
        let legacy = app_data.join("Sessions/current");
        if legacy.join("session.json").exists() && !catalog.list()?.iter().any(|w| w.legacy) {
            let original = std::fs::read(legacy.join("session.json")).map_err(|e| e.to_string())?;
            let manifest: Manifest = serde_json::from_slice(&original)
                .map_err(|e| format!("Previous session needs recovery: {e}"))?;
            if !manifest.tracks.is_empty() {
                let id = media::unique_id();
                let stage = catalog.root.join(format!(".importing-{id}"));
                std::fs::create_dir(&stage).map_err(|e| e.to_string())?;
                for entry in std::fs::read_dir(&legacy).map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    if entry.file_type().map_err(|e| e.to_string())?.is_file() {
                        std::fs::copy(entry.path(), stage.join(entry.file_name()))
                            .map_err(|e| e.to_string())?;
                    }
                }
                let info = WorkspaceInfo {
                    id: id.clone(),
                    name: "Recovered recordings".into(),
                    created_at: timestamp(),
                    legacy: true,
                    requires_name: true,
                };
                metadata(&stage, &info)?;
                std::fs::rename(stage, catalog.root.join(id)).map_err(|e| e.to_string())?;
            }
        }
        Ok(catalog)
    }
    pub fn list(&self) -> Result<Vec<WorkspaceInfo>, String> {
        let mut items = Vec::new();
        for entry in std::fs::read_dir(&self.root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().into_owned();
            if !valid_id(&id) {
                continue;
            }
            let Ok(bytes) = std::fs::read(entry.path().join("workspace.json")) else {
                continue;
            };
            let Ok(info) = serde_json::from_slice::<WorkspaceInfo>(&bytes) else {
                continue;
            };
            if info.id == id {
                items.push(info);
            }
        }
        items.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| a.name.cmp(&b.name))
        });
        Ok(items)
    }
    pub fn view(&self) -> Result<WorkspaceView, String> {
        Ok(WorkspaceView {
            items: self.list()?,
            active: self.active.clone(),
        })
    }
    pub fn create(&mut self, name: &str) -> Result<WorkspaceInfo, String> {
        let name = valid_name(name)?;
        let info = WorkspaceInfo {
            id: media::unique_id(),
            name,
            created_at: timestamp(),
            legacy: false,
            requires_name: false,
        };
        let path = self.root.join(&info.id);
        std::fs::create_dir(&path).map_err(|e| e.to_string())?;
        metadata(&path, &info)?;
        Ok(info)
    }
    pub fn rename(&mut self, id: &str, name: &str) -> Result<(), String> {
        let mut info = self.find(id)?;
        info.name = valid_name(name)?;
        info.requires_name = false;
        metadata(&self.root.join(id), &info)?;
        if self.active.as_ref().is_some_and(|w| w.id == id) {
            self.active = Some(info);
        }
        Ok(())
    }
    fn find(&self, id: &str) -> Result<WorkspaceInfo, String> {
        if !valid_id(id) {
            return Err("Invalid workspace identifier".into());
        }
        self.list()?
            .into_iter()
            .find(|w| w.id == id)
            .ok_or("Workspace not found".into())
    }
    pub fn open(&mut self, engine: &mut Engine, id: &str) -> Result<(), String> {
        let info = self.find(id)?;
        if info.requires_name {
            return Err("Give the recovered recordings a workspace name first".into());
        }
        engine.open_workspace(self.root.join(id))?;
        self.active = Some(info);
        Ok(())
    }
    pub fn close(&mut self, engine: &mut Engine) -> Result<(), String> {
        engine.close_workspace()?;
        self.active = None;
        Ok(())
    }
}
impl Engine {
    pub fn open_workspace(&mut self, path: PathBuf) -> Result<(), String> {
        if self.streams.is_some() || self.recorder.is_some() {
            return Err("Stop audio before switching workspaces".into());
        }
        let mut next = Self::default();
        next.configure_storage(path)?;
        next.params = self.params.clone();
        next.revision = self.revision + 1;
        *self = next;
        Ok(())
    }
    pub fn close_workspace(&mut self) -> Result<(), String> {
        if self.streams.is_some() || self.recorder.is_some() {
            return Err("Stop audio before closing the workspace".into());
        }
        let mut next = Self::default();
        next.params = self.params.clone();
        next.revision = self.revision + 1;
        *self = next;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> PathBuf {
        std::env::temp_dir().join(format!("kuerdas-workspaces-{}", media::unique_id()))
    }
    #[test]
    fn workspace_names_are_required_and_sessions_are_isolated() {
        let base = base();
        let mut library = WorkspaceCatalog::new(base.clone()).unwrap();
        let mut engine = Engine::default();
        assert!(library.view().unwrap().active.is_none());
        assert!(library.create("  ").is_err());
        assert!(engine.create_track("No workspace", 44100).is_err());
        let a = library.create("Rhythm / ideas").unwrap();
        let b = library.create("Lead session").unwrap();
        library.open(&mut engine, &a.id).unwrap();
        engine.create_track("Guitar", 44100).unwrap();
        library.open(&mut engine, &b.id).unwrap();
        assert!(engine.tracks.is_empty());
        library.open(&mut engine, &a.id).unwrap();
        assert_eq!(engine.tracks[0].info.name, "Guitar");
        library.rename(&a.id, "Renamed session").unwrap();
        assert_eq!(
            library.view().unwrap().active.unwrap().name,
            "Renamed session"
        );
        library.close(&mut engine).unwrap();
        assert!(engine.storage.as_os_str().is_empty());
        drop(engine);
        std::fs::remove_dir_all(base).unwrap();
    }
    #[test]
    fn legacy_recordings_are_copied_once_and_must_be_named() {
        let base = base();
        let legacy = base.join("Sessions/current");
        let mut old = Engine::default();
        old.configure_storage(legacy.clone()).unwrap();
        old.create_track("Original lane", 44100).unwrap();
        std::fs::write(legacy.join("retained-audio.wav"), b"original bytes").unwrap();
        drop(old);
        let original = std::fs::read(legacy.join("session.json")).unwrap();
        let mut library = WorkspaceCatalog::new(base.clone()).unwrap();
        let recovered = library.list().unwrap().pop().unwrap();
        assert!(recovered.requires_name);
        assert_eq!(
            std::fs::read(legacy.join("session.json")).unwrap(),
            original
        );
        assert_eq!(
            std::fs::read(library.root.join(&recovered.id).join("retained-audio.wav")).unwrap(),
            b"original bytes"
        );
        let mut engine = Engine::default();
        assert!(library.open(&mut engine, &recovered.id).is_err());
        library.rename(&recovered.id, "My first recording").unwrap();
        library.open(&mut engine, &recovered.id).unwrap();
        assert_eq!(engine.tracks[0].info.name, "Original lane");
        drop(engine);
        let again = WorkspaceCatalog::new(base.clone()).unwrap();
        assert_eq!(again.list().unwrap().len(), 1);
        assert!(again.active.is_none());
        std::fs::remove_dir_all(base).unwrap();
    }
}
