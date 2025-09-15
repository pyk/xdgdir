use std::{env, fmt, path::PathBuf};

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
    NotAbsolutePath(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::HomeNotSet => {
                write!(f, "HOME environment variable not set or empty")
            }
            Error::NotAbsolutePath(path) => {
                write!(f, "'{path}' is not absolute path", path = path.display())
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
    fn from_context(context: &impl Context) -> Result<Self, Error> {
        let home = context.get("HOME").ok_or(Error::HomeNotSet)?;
        if home.is_empty() {
            return Err(Error::HomeNotSet);
        }

        let home_path = PathBuf::from(home);
        if !home_path.is_absolute() {
            return Err(Error::NotAbsolutePath(home_path));
        }

        Ok(BaseDir {
            home: PathBuf::new(),
            config: PathBuf::new(),
            data: PathBuf::new(),
            state: PathBuf::new(),
            cache: PathBuf::new(),
            runtime: None,
            bin: PathBuf::new(),
        })
    }

    pub fn global() -> Result<Self, Error> {
        Self::from_context(&Env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{borrow::Borrow, collections::HashMap, hash::Hash};

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
        assert_eq!(result.unwrap_err(), Error::HomeNotSet);
    }

    #[test]
    fn home_empty() {
        let mut context = HashMap::new();
        context.insert("HOME", "");
        let result = BaseDir::from_context(&context);
        assert_eq!(result.unwrap_err(), Error::HomeNotSet);
    }

    #[test]
    fn home_not_absolute() {
        let mut context = HashMap::new();
        context.insert("HOME", "some/dir");
        let result = BaseDir::from_context(&context);
        assert_eq!(
            result.unwrap_err(),
            Error::NotAbsolutePath(PathBuf::from("some/dir"))
        );
    }
}
