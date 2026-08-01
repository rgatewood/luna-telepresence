use std::io;
use std::path::{Path, PathBuf};

const LEGACY_APP_IDENTIFIER: &str = "com.meetily.ai";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MigrationReport {
    pub files_reused: usize,
    pub bytes_reused: u64,
}

pub fn legacy_models_directory(current_app_data_dir: &Path) -> Option<PathBuf> {
    current_app_data_dir
        .parent()
        .map(|parent| parent.join(LEGACY_APP_IDENTIFIER).join("models"))
}

pub fn migrate_legacy_models(
    legacy_models_dir: &Path,
    current_models_dir: &Path,
) -> io::Result<MigrationReport> {
    let mut report = MigrationReport::default();

    if !legacy_models_dir.is_dir() {
        return Ok(report);
    }

    migrate_directory(
        legacy_models_dir,
        legacy_models_dir,
        current_models_dir,
        &mut report,
    )?;
    Ok(report)
}

fn migrate_directory(
    legacy_root: &Path,
    directory: &Path,
    current_root: &Path,
    report: &mut MigrationReport,
) -> io::Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let metadata = entry.file_type()?;
        let source = entry.path();
        let relative = source
            .strip_prefix(legacy_root)
            .map_err(io::Error::other)?;
        let target = current_root.join(relative);

        if metadata.is_dir() {
            migrate_directory(legacy_root, &source, current_root, report)?;
        } else if metadata.is_file() {
            migrate_file(&source, &target, report)?;
        }
    }

    Ok(())
}

fn migrate_file(
    source: &Path,
    target: &Path,
    report: &mut MigrationReport,
) -> io::Result<()> {
    let source_len = std::fs::metadata(source)?.len();
    if source_len == 0 {
        return Ok(());
    }

    if target
        .metadata()
        .map(|metadata| metadata.len() >= source_len)
        .unwrap_or(false)
    {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("model");
    let staged = target.with_file_name(format!(".{file_name}.luna-migrate"));
    if staged.exists() {
        std::fs::remove_file(&staged)?;
    }

    if std::fs::hard_link(source, &staged).is_err() {
        std::fs::copy(source, &staged)?;
    }

    let staged_len = std::fs::metadata(&staged)?.len();
    if staged_len != source_len {
        let _ = std::fs::remove_file(&staged);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "legacy model migration size mismatch for {}: expected {source_len}, got {staged_len}",
                source.display()
            ),
        ));
    }

    if target.exists() {
        std::fs::remove_file(target)?;
    }
    std::fs::rename(&staged, target)?;

    report.files_reused += 1;
    report.bytes_reused += source_len;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolves_meetily_models_as_a_sibling_of_luna_app_data() {
        let current = Path::new("C:/Users/test/AppData/Roaming/com.luna.telepresence");

        assert_eq!(
            legacy_models_directory(current),
            Some(PathBuf::from(
                "C:/Users/test/AppData/Roaming/com.meetily.ai/models"
            ))
        );
    }

    #[test]
    fn reuses_complete_legacy_models_and_replaces_empty_placeholders() {
        let root = tempfile::tempdir().unwrap();
        let legacy = root.path().join("com.meetily.ai/models");
        let current = root.path().join("com.luna.telepresence/models");
        let legacy_summary = legacy.join("summary/model.gguf");
        let current_summary = current.join("summary/model.gguf");

        fs::create_dir_all(legacy_summary.parent().unwrap()).unwrap();
        fs::create_dir_all(current_summary.parent().unwrap()).unwrap();
        fs::write(&legacy_summary, b"complete-model").unwrap();
        fs::write(&current_summary, b"").unwrap();

        let report = migrate_legacy_models(&legacy, &current).unwrap();

        assert_eq!(fs::read(&current_summary).unwrap(), b"complete-model");
        assert_eq!(report.files_reused, 1);
        assert_eq!(report.bytes_reused, 14);
        assert_eq!(fs::read(&legacy_summary).unwrap(), b"complete-model");
    }

    #[test]
    fn preserves_a_current_model_when_it_is_not_smaller_than_legacy() {
        let root = tempfile::tempdir().unwrap();
        let legacy = root.path().join("legacy");
        let current = root.path().join("current");
        let legacy_model = legacy.join("summary/model.gguf");
        let current_model = current.join("summary/model.gguf");

        fs::create_dir_all(legacy_model.parent().unwrap()).unwrap();
        fs::create_dir_all(current_model.parent().unwrap()).unwrap();
        fs::write(&legacy_model, b"old").unwrap();
        fs::write(&current_model, b"current-model").unwrap();

        let report = migrate_legacy_models(&legacy, &current).unwrap();

        assert_eq!(fs::read(&current_model).unwrap(), b"current-model");
        assert_eq!(report, MigrationReport::default());
    }
}
