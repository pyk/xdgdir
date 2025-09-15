use std::{
    env,
    fmt,
    path::PathBuf,
};

pub trait Context {
    fn get(&self, key: &str) -> Option<String>;
}

pub struct Env;
impl Context for Env {
    fn get(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    HomeNotSet,
    NotAbsolutePath(String, PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::HomeNotSet => {
                write!(f, "$HOME is not set or empty")
            }
            Error::NotAbsolutePath(key, path) => {
                write!(
                    f,
                    "{key}=\"{path}\" is not absolute path",
                    path = path.display()
                )
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BaseDir {
    pub home: PathBuf,
    pub config: PathBuf,
    pub data: PathBuf,
    pub state: PathBuf,
    pub cache: PathBuf,
    pub runtime: Option<PathBuf>,
    pub bin: PathBuf,
}

impl BaseDir {
    fn ensure_path(key: &str, path: String) -> Result<PathBuf, Error> {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            Ok(path)
        } else {
            Err(Error::NotAbsolutePath(key.to_string(), path))
        }
    }

    fn get_home(context: &impl Context) -> Result<PathBuf, Error> {
        match context.get("HOME") {
            None => Err(Error::HomeNotSet),
            Some(path) if path.is_empty() => Err(Error::HomeNotSet),
            Some(path) => Self::ensure_path("HOME", path),
        }
    }

    fn get_path(
        context: &impl Context,
        key: &str,
        default: PathBuf,
    ) -> Result<PathBuf, Error> {
        match context.get(key) {
            None => Ok(default),
            Some(path) if path.is_empty() => Ok(default),
            Some(path) => Self::ensure_path(key, path),
        }
    }

    fn from_context(context: &impl Context) -> Result<Self, Error> {
        let home = Self::get_home(context)?;
        let bin = home.join(".local").join("bin");
        let data = Self::get_path(
            context,
            "XDG_DATA_HOME",
            home.join(".local").join("share"),
        )?;
        let config = Self::get_path(
            context, //
            "XDG_CONFIG_HOME",
            home.join(".config"),
        )?;
        let state = Self::get_path(
            context,
            "XDG_STATE_HOME",
            home.join(".local").join("state"),
        )?;
        let cache = Self::get_path(
            context, //
            "XDG_CACHE_HOME",
            home.join(".cache"),
        )?;
        let runtime = match context.get("XDG_RUNTIME_DIR") {
            None => Ok(None),
            Some(path) if path.is_empty() => Ok(None),
            Some(path) => Self::ensure_path("XDG_RUNTIME_DIR", path).map(Some),
        }?;

        Ok(BaseDir {
            home,
            bin,
            data,
            config,
            state,
            cache,
            runtime,
        })
    }

    pub fn global() -> Result<Self, Error> {
        Self::from_context(&Env)
    }

    pub fn new(app_name: &str) -> Result<Self, Error> {
        let mut global_dirs = Self::global()?;

        global_dirs.config.push(app_name);
        global_dirs.data.push(app_name);
        global_dirs.state.push(app_name);
        global_dirs.cache.push(app_name);
        if let Some(runtime_path) = global_dirs.runtime.as_mut() {
            runtime_path.push(app_name);
        }

        Ok(global_dirs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        borrow::Borrow,
        collections::HashMap,
        hash::Hash,
    };

    impl<K, S> Context for HashMap<K, S>
    where
        K: Borrow<str> + Eq + Hash,
        S: AsRef<str>,
    {
        fn get(&self, key: &str) -> Option<String> {
            self.get(key).map(|s| s.as_ref().to_string())
        }
    }

    #[test]
    fn home_not_set() {
        let mut context = HashMap::new();
        context.insert("DEBUG", "");
        let result = BaseDir::from_context(&context);
        let error = result.unwrap_err();
        let report = format!("{}", error);
        assert_eq!(error, Error::HomeNotSet);
        assert_eq!(report, "$HOME is not set or empty");
    }

    #[test]
    fn home_empty() {
        let mut context = HashMap::new();
        context.insert("HOME", "");
        let result = BaseDir::from_context(&context);
        let error = result.unwrap_err();
        let report = format!("{}", error);
        assert_eq!(error, Error::HomeNotSet);
        assert_eq!(report, "$HOME is not set or empty");
    }

    #[test]
    fn home_not_absolute() {
        let mut context = HashMap::new();
        context.insert("HOME", "some/dir");
        let result = BaseDir::from_context(&context);
        let error = result.unwrap_err();
        let report = format!("{}", error);
        assert_eq!(
            error,
            Error::NotAbsolutePath("HOME".into(), "some/dir".into())
        );
        assert_eq!(report, "HOME=\"some/dir\" is not absolute path");
    }

    #[test]
    fn bin() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.bin, PathBuf::from("/home/user/.local/bin"));
    }

    #[test]
    fn xdg_data_home_not_set() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.data, PathBuf::from("/home/user/.local/share"));
    }

    #[test]
    fn xdg_data_home_not_absolute() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_DATA_HOME", "some/dir");
        let result = BaseDir::from_context(&context);
        let error = result.unwrap_err();
        let report = format!("{}", error);
        assert_eq!(
            error,
            Error::NotAbsolutePath("XDG_DATA_HOME".into(), "some/dir".into())
        );
        assert_eq!(report, "XDG_DATA_HOME=\"some/dir\" is not absolute path");
    }

    #[test]
    fn xdg_data_home_valid() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_DATA_HOME", "/some/dir");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.data, PathBuf::from("/some/dir"));
    }

    #[test]
    fn xdg_config_home_not_set() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.config, PathBuf::from("/home/user/.config"));
    }

    #[test]
    fn xdg_config_home_not_absolute() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_CONFIG_HOME", "some/dir");
        let result = BaseDir::from_context(&context);
        let error = result.unwrap_err();
        let report = format!("{}", error);
        assert_eq!(
            error,
            Error::NotAbsolutePath("XDG_CONFIG_HOME".into(), "some/dir".into())
        );
        assert_eq!(report, "XDG_CONFIG_HOME=\"some/dir\" is not absolute path");
    }

    #[test]
    fn xdg_config_home_valid() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_CONFIG_HOME", "/some/dir");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.config, PathBuf::from("/some/dir"));
    }

    #[test]
    fn xdg_state_home_not_set() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.state, PathBuf::from("/home/user/.local/state"));
    }

    #[test]
    fn xdg_state_home_not_absolute() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_STATE_HOME", "some/dir");
        let result = BaseDir::from_context(&context);
        let error = result.unwrap_err();
        let report = format!("{}", error);
        assert_eq!(
            error,
            Error::NotAbsolutePath("XDG_STATE_HOME".into(), "some/dir".into())
        );
        assert_eq!(report, "XDG_STATE_HOME=\"some/dir\" is not absolute path");
    }

    #[test]
    fn xdg_state_home_valid() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_STATE_HOME", "/some/dir");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.state, PathBuf::from("/some/dir"));
    }

    #[test]
    fn xdg_cache_home_not_set() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.cache, PathBuf::from("/home/user/.cache"));
    }

    #[test]
    fn xdg_cache_home_not_absolute() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_CACHE_HOME", "some/dir");
        let result = BaseDir::from_context(&context);
        let error = result.unwrap_err();
        let report = format!("{}", error);
        assert_eq!(
            error,
            Error::NotAbsolutePath("XDG_CACHE_HOME".into(), "some/dir".into())
        );
        assert_eq!(report, "XDG_CACHE_HOME=\"some/dir\" is not absolute path");
    }

    #[test]
    fn xdg_cache_home_valid() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_CACHE_HOME", "/some/dir");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.cache, PathBuf::from("/some/dir"));
    }

    #[test]
    fn xdg_runtime_dir_not_set() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.runtime, None);
    }

    #[test]
    fn xdg_runtime_dir_empty() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_RUNTIME_DIR", "");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.runtime, None);
    }

    #[test]
    fn xdg_runtime_dir_not_absolute() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_RUNTIME_DIR", "some/dir");
        let result = BaseDir::from_context(&context);
        let error = result.unwrap_err();
        let report = format!("{}", error);
        assert_eq!(
            error,
            Error::NotAbsolutePath("XDG_RUNTIME_DIR".into(), "some/dir".into())
        );
        assert_eq!(report, "XDG_RUNTIME_DIR=\"some/dir\" is not absolute path");
    }

    #[test]
    fn xdg_runtime_dir_valid() {
        let mut context = HashMap::new();
        context.insert("HOME", "/home/user");
        context.insert("XDG_RUNTIME_DIR", "/run/user/1000");
        let result = BaseDir::from_context(&context).unwrap();
        assert_eq!(result.runtime, Some(PathBuf::from("/run/user/1000")));
    }
}
