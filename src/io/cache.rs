use std::fs;
use std::io as std_io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use app;
use io::{serialize, Read, Write};

pub struct Cache {
    name: String,
    dir: PathBuf,
}

pub static CACHE_DIRNAME: &'static str = "cache";

impl Cache {
    pub fn new<S: Into<String>>(name: S) -> Self {
        let mut dir = app::app_dir().unwrap();
        debug_assert!(dir.exists());
        dir.push(CACHE_DIRNAME);
        Cache::with_dir(name, dir, true).unwrap()
    }

    pub fn with_dir<S: Into<String>, P: AsRef<Path>>(
        name: S,
        dir: P,
        mkdir: bool,
    ) -> std_io::Result<Self> {
        let path = dir.as_ref().to_path_buf();
        if !path.exists() {
            if mkdir {
                fs::create_dir(&dir)?;
            } else {
                return Err(std_io::Error::new(
                    std_io::ErrorKind::NotFound,
                    format!("file `{}` is not found", path.to_str().unwrap()),
                ));
            }
        } else if !path.is_dir() {
            return Err(std_io::Error::new(
                std_io::ErrorKind::Other,
                format!(
                    "path `{}` is not a directory",
                    path.to_str().unwrap()
                ),
            ));
        }
        Ok(Cache {
            name: name.into(),
            dir: path,
        })
    }

    pub fn read<T: DeserializeOwned>(&mut self, name: &str) -> std_io::Result<T> {
        let path = self.resolve(name);
        self.read_from(path)
    }

    pub fn read_from<P: AsRef<Path>, T: DeserializeOwned>(&mut self, path: P) -> std_io::Result<T> {
        let mut serializer = serialize::Serializer::new(
            std_io::BufReader::new(fs::File::open(path)?),
            serialize::Format::Msgpack,
        );
        let mut buf = Vec::with_capacity(1);
        serializer.read_upto(1, &mut buf)?;
        if buf.len() != 1 {
            Err(std_io::Error::new(
                std_io::ErrorKind::InvalidData,
                "broken data",
            ))
        } else {
            Ok(buf.pop().unwrap())
        }
    }

    pub fn write<T: Serialize>(&mut self, name: &str, data: &T) -> std_io::Result<()> {
        let path = self.resolve(name);
        self.write_to(path, data)
    }

    pub fn write_to<P: AsRef<Path>, T: Serialize>(
        &mut self,
        path: P,
        data: &T,
    ) -> std_io::Result<()> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let mut serializer =
            serialize::Serializer::new(std_io::BufWriter::new(file), serialize::Format::Msgpack);
        serializer.write(&[data])?;
        serializer.flush()
    }

    pub fn exists(&self, name: &str) -> bool {
        self.resolve(name).exists()
    }

    pub fn resolve(&self, name: &str) -> PathBuf {
        let mut path = self.dir.clone();
        path.push(format!("{}-{}.mpac", self.name, name));
        path
    }
}

#[inline]
pub fn read_cache<T: DeserializeOwned>(prefix: &str, name: &str) -> std_io::Result<T> {
    Cache::new(prefix).read(name)
}

#[inline]
pub fn write_cache<T: Serialize>(prefix: &str, name: &str, data: &T) -> std_io::Result<()> {
    Cache::new(prefix).write(name, data)
}

#[inline]
pub fn cache_exists(prefix: &str, name: &str) -> bool {
    Cache::new(prefix).exists(name)
}

pub trait FromCache: Serialize + Sized {
    fn from_cache(name: &str) -> std_io::Result<Self>;

    fn has_cache(name: &str) -> bool;
}

pub trait IntoCache: DeserializeOwned {
    fn into_cache(&self, name: &str) -> std_io::Result<()>;
}

#[macro_export]
macro_rules! impl_cache {
    ($name:ident) => {
        use std::io::Result as IOResult;

        impl cache::FromCache for $name {
            fn from_cache(name: &str) -> IOResult<Self> {
                let prefix = stringify!($name).to_lowercase();
                cache::read_cache(&prefix, name)
            }

            fn has_cache(name: &str) -> bool {
                let prefix = stringify!($name).to_lowercase();
                cache::cache_exists(&prefix, name)
            }
        }

        impl cache::IntoCache for $name {
            fn into_cache(&self, name: &str) -> IOResult<()> {
                let prefix = stringify!($name).to_lowercase();
                cache::write_cache(&prefix, name, self)
            }
        }
    }
}