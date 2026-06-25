use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use zip::ZipArchive;
use zip::write::FileOptions;




use crate::model::ModMetadata;

#[derive(Debug)]
pub struct ModEditorState {
    pub zip_path: PathBuf,
    pub staging_dir: TempDir,
    pub manifest: ModMetadata,
}

impl ModEditorState {
    pub fn staging_root(&self) -> &Path {
        self.staging_dir.path()
    }
}

pub fn open_for_editing(zip_path: &Path) -> Result<ModEditorState, String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open mod zip '{zip_path:?}': {e}"))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to parse zip '{zip_path:?}': {e}"))?;

    let staging_dir = tempfile::TempDir::new().map_err(|e| format!("TempDir error: {e}"))?;
    let root = staging_dir.path();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Zip entry error at index {i}: {e}"))?;

        let raw_name = entry.name().to_string();
        let name = raw_name.replace('\\', "/");


        if entry.is_dir() {
            continue;
        }

        let out_path = safe_join(root, Path::new(&name.as_str()))
            .ok_or_else(|| format!("Invalid zip entry path: '{raw_name}'"))?;

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir '{parent:?}': {e}"))?;
        }

        let mut contents = Vec::new();
        entry
            .read_to_end(&mut contents)
            .map_err(|e| format!("Failed to read zip entry '{raw_name}': {e}"))?;

        let mut f = fs::File::create(&out_path)
            .map_err(|e| format!("Failed to write extracted file '{out_path:?}': {e}"))?;
        f.write_all(&contents)
            .map_err(|e| format!("Failed to write extracted file '{out_path:?}': {e}"))?;
    }

    let mut manifest = ModMetadata::from_zip_or_sidecar(zip_path);

    if manifest.version.trim().is_empty() {
        manifest.version = "0.1.0".to_string();
    }
    if manifest.game_version.trim().is_empty() {
        manifest.game_version = "unknown".to_string();
    }

    let manifest_path = root.join("blitz-mod.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize blitz-mod.json: {e}"))?;
    fs::write(&manifest_path, manifest_json)
        .map_err(|e| format!("Failed to write manifest '{manifest_path:?}': {e}"))?;

    Ok(ModEditorState {

        zip_path: zip_path.to_path_buf(),
        staging_dir,
        manifest,
    })
}

pub fn save_and_repack(state: &mut ModEditorState) -> Result<(), String> {
    let zip_path = state.zip_path.clone();

    let manifest_path = state.staging_root().join("blitz-mod.json");
    let manifest_json = serde_json::to_string_pretty(&state.manifest)
        .map_err(|e| format!("Failed to serialize blitz-mod.json: {e}"))?;
    fs::write(&manifest_path, manifest_json)
        .map_err(|e| format!("Failed to write manifest '{manifest_path:?}': {e}"))?;

    let staging_root = state.staging_root();

    let file = fs::File::create(&zip_path)
        .map_err(|e| format!("Failed to create output zip '{zip_path:?}': {e}"))?;
    let mut writer = zip::ZipWriter::new(file);

    pack_dir_contents_enforce_data(staging_root, staging_root, &mut writer)?;

    writer
        .finish()
        .map_err(|_e| format!("Failed to finalize zip '{zip_path:?}'"))?;


    Ok(())

}

fn pack_dir_contents_enforce_data(
    root: &Path,
    dir: &Path,
    writer: &mut zip::ZipWriter<fs::File>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir)
        .map_err(|e| format!("read_dir failed for '{dir:?}': {e}"))?
    {
        let entry = entry.map_err(|e| format!("read_dir entry error: {e}"))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map_err(|e| format!("strip_prefix failed for '{path:?}': {e}"))?;

        if rel.as_os_str().is_empty() {
            continue;
        }

        if path.is_dir() {
            pack_dir_contents_enforce_data(root, &path, writer)?;
            continue;
        }

        let rel_str = rel.to_string_lossy().replace('\\', "/");

        let zip_entry_name = if rel_str == "blitz-mod.json" {
            rel_str
        } else {
            format!("Data/{}", rel_str)
        };

        writer
            .start_file::<_, ()>(&zip_entry_name, FileOptions::default())
            .map_err(|e| format!("Zip start_file failed: {e}"))?;

        let mut f = fs::File::open(&path)
            .map_err(|e| format!("Failed to open '{path:?}' for zip: {e}"))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)
            .map_err(|e| format!("Failed to read '{path:?}' for zip: {e}"))?;

        writer
            .write_all(&buf)
            .map_err(|e| format!("Failed to write '{path:?}' into zip: {e}"))?;
    }

    Ok(())
}


fn safe_join(base: &Path, path: &Path) -> Option<PathBuf> {
    use std::path::Component;

    let mut out = base.to_path_buf();
    for comp in path.components() {
        match comp {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            Component::ParentDir => return None,
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(out)
}

