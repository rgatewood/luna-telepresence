use std::io;
use std::path::PathBuf;

pub fn data_directory() -> io::Result<PathBuf> {
    dirs::data_local_dir()
        .map(|directory| directory.join(crate::brand::MODEL_LIBRARY_DIRECTORY_NAME))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "local data directory unavailable"))
}

pub fn models_directory() -> io::Result<PathBuf> {
    Ok(data_directory()?.join("models"))
}

pub fn summary_models_directory() -> io::Result<PathBuf> {
    Ok(models_directory()?.join("summary"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_library_is_independent_of_the_tauri_app_identifier() {
        let path = models_directory().unwrap();
        assert!(path.ends_with("Luna Telepresence Model Library/models"));
        assert!(!path.to_string_lossy().contains("com.luna.telepresence"));
        assert!(!path.to_string_lossy().contains("com.meetily.ai"));
    }
}
