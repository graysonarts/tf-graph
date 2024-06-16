use std::{
    env,
    path::{Path, PathBuf},
};

pub struct WithWorkingDirectory {
    pub(crate) original: PathBuf,
}

impl WithWorkingDirectory {
    pub fn new<P: AsRef<Path>>(new_cwd: P) -> WithWorkingDirectory {
        let original = env::current_dir().expect("Unable to get current directory");
        env::set_current_dir(&new_cwd).expect("Unable to set current directory");
        tracing::info!("+CWD: {:?}", new_cwd.as_ref());
        WithWorkingDirectory { original }
    }
}

impl Drop for WithWorkingDirectory {
    fn drop(&mut self) {
        env::set_current_dir(&self.original).expect("Unable to reset current directory");
        tracing::info!("-CWD: {:?}", self.original);
    }
}

pub fn with_cwd<R>(new_cwd: impl AsRef<Path>, f: impl FnOnce() -> R) -> R {
    let cwd = WithWorkingDirectory::new(new_cwd);
    tracing::info!("Running Command");
    let res = f();
    drop(cwd);

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_working_directory() {
        {
            let _cwd = WithWorkingDirectory::new(PathBuf::from("/private/tmp"));
            assert_eq!(env::current_dir().unwrap(), PathBuf::from("/private/tmp"));
        }

        assert_eq!(
            env::current_dir().unwrap(),
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        );
    }
}
