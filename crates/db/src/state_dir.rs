use std::{io::Error as IoError, path::PathBuf};

use redefmt::APPLICATION_NAME;
use xdg::BaseDirectoriesError;

const XDG_ENV_NAME: &str = "XDG_STATE_HOME";

const OVERRIDE_ENV_NAME: &str = "REDEFMT_STATE";

pub struct StateDir;

#[derive(Debug, Clone, Copy, derive_more::Display)]
pub enum StateDirSource {
    #[display("{}", XDG_ENV_NAME)]
    Xdg,
    #[display("{}", OVERRIDE_ENV_NAME)]
    Env,
}

#[derive(Debug, thiserror::Error)]
pub enum StateDirError {
    #[error("failed to load XDG base directories")]
    XdgDirLoad(#[from] BaseDirectoriesError),
    #[error("path may not be empty, source; {0}")]
    EmptyPath(StateDirSource),
    #[error("path must be a valid directory; resolved {0} from source {1}")]
    NonDir(PathBuf, StateDirSource),
    #[error("unable to canonicalize state path; resolved {0} from source {1}")]
    Canonicalize(PathBuf, StateDirSource, #[source] IoError),
    #[error("unable to check whether {0} exists; source {1}")]
    Exists(PathBuf, StateDirSource, #[source] IoError),
    #[error("failed to retrieve file metadatda for {0}; source {1}")]
    Metadata(PathBuf, StateDirSource, #[source] IoError),
    #[error("failed to create state directory and its parent source at {0}; source {1}")]
    Create(PathBuf, StateDirSource, #[source] IoError),
}

impl StateDir {
    /// Resolves `redefmt`'s state directory
    ///
    /// The returned `PathBuf` can be expected to not be empty, exist, be in
    /// canonical form, and a directory.
    pub fn resolve() -> Result<PathBuf, StateDirError> {
        match std::env::var_os(OVERRIDE_ENV_NAME) {
            Some(override_path) => {
                let path_buf = PathBuf::from(override_path);
                Self::prepare_state_directory(path_buf, StateDirSource::Env)
            }
            None => {
                let xdg_base_dirs = xdg::BaseDirectories::with_prefix(APPLICATION_NAME)?.get_state_home();

                Self::prepare_state_directory(xdg_base_dirs, StateDirSource::Xdg)
            }
        }
    }

    fn prepare_state_directory(
        state_dir_path: PathBuf,
        state_dir_source: StateDirSource,
    ) -> Result<PathBuf, StateDirError> {
        if state_dir_path.as_os_str().is_empty() {
            return Err(StateDirError::EmptyPath(state_dir_source));
        }

        let path_exists = state_dir_path
            .try_exists()
            .map_err(|err| StateDirError::Exists(state_dir_path.clone(), state_dir_source, err))?;

        match path_exists {
            true => {
                let canonical_path = state_dir_path
                    .canonicalize()
                    .map_err(|err| StateDirError::Canonicalize(state_dir_path.clone(), state_dir_source, err))?;

                let path_metadata = canonical_path
                    .metadata()
                    .map_err(|err| StateDirError::Metadata(canonical_path.clone(), state_dir_source, err))?;

                if !path_metadata.is_dir() {
                    return Err(StateDirError::NonDir(canonical_path, state_dir_source));
                }
            }
            false => {
                std::fs::create_dir_all(&state_dir_path)
                    .map_err(|err| StateDirError::Create(state_dir_path.clone(), state_dir_source, err))?;
            }
        }

        Ok(state_dir_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial(state_dir)]
    fn uses_xdg() {
        // SAFETY: test called with serial_test
        unsafe {
            assert_uses_env(XDG_ENV_NAME, Some(APPLICATION_NAME));
        }
    }

    #[test]
    #[serial_test::serial(state_dir)]
    fn uses_override_env() {
        // SAFETY: test called with serial_test
        unsafe {
            assert_uses_env(OVERRIDE_ENV_NAME, None);
        }
    }

    /// # SAFETY
    ///
    /// Test calling this function must be serial
    unsafe fn assert_uses_env(env_name: &str, join: Option<&str>) {
        let temp_dir = tempfile::tempdir().unwrap();

        // SAFETY: method should only called by one thread at the time
        unsafe {
            std::env::set_var(env_name, temp_dir.path().as_os_str());
        }

        let actual = StateDir::resolve().unwrap();

        let mut expected = temp_dir.into_path();
        if let Some(join) = join {
            expected = expected.join(join);
        }

        assert_eq!(expected, actual);

        // SAFETY: method should only called by one thread at the time
        unsafe {
            std::env::remove_var(env_name);
        }
    }

    #[test]
    fn valid_if_some() {
        let temp_dir = tempfile::tempdir().unwrap();

        let prepare_result = StateDir::prepare_state_directory(temp_dir.path().to_path_buf(), StateDirSource::Env);

        assert!(prepare_result.is_ok());
    }

    #[test]
    fn create_if_none() {
        let temp_dir = tempfile::tempdir().unwrap();

        let state_dir = temp_dir.path().join("new");

        assert!(!state_dir.exists());

        let returned_dir = StateDir::prepare_state_directory(state_dir.clone(), StateDirSource::Env).unwrap();

        assert_eq!(state_dir, returned_dir);

        assert!(state_dir.exists());
    }

    #[test]
    fn empty_path_error() {
        let state_dir = PathBuf::new();

        let result = StateDir::prepare_state_directory(state_dir, StateDirSource::Env);

        assert!(matches!(result, Err(StateDirError::EmptyPath(_))));
    }

    #[test]
    fn non_dir_error() {
        let state_dir = tempfile::NamedTempFile::new().unwrap();

        let result = StateDir::prepare_state_directory(state_dir.path().to_path_buf(), StateDirSource::Env);

        assert!(matches!(result, Err(StateDirError::NonDir(_, _))));
    }
}
