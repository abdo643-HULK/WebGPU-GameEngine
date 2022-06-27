use std::{
    borrow::Cow,
    collections::HashMap,
    io::Read,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use core::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

fn get_hasher() -> impl Hasher {
    DefaultHasher::new()
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AssetPath<'a>(Cow<'a, Path>);

impl<'a> From<&'a str> for AssetPath<'a> {
    fn from(asset_path: &'a str) -> Self {
        let path = Path::new(asset_path);
        AssetPath(Cow::Borrowed(path))
    }
}

#[derive(Debug)]
pub struct Assets<T: core::fmt::Debug> {
    pub assets: HashMap<HandleId, T>,
}

#[derive(Debug)]
struct AssetsFolder(&'static str);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct HandleId(u64);

impl<'a> From<AssetPath<'a>> for HandleId {
    fn from(value: AssetPath<'a>) -> Self {
        let mut hasher = get_hasher();
        hasher.write(value.0.to_str().unwrap().as_bytes());
        Self(hasher.finish())
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct AssetHandle<T> {
    pub id: HandleId,
    marker: PhantomData<fn() -> T>,
}

#[derive(Debug)]
struct AssetLoader {
    assets_path: PathBuf,
}

impl AssetLoader {
    pub fn new(root: &'static str) -> Self {
        Self {
            assets_path: Path::new(env!("CARGO_MANIFEST_DIR")).join(root),
        }
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self {
            assets_path: Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"),
        }
    }
}

#[derive(Debug, Default)]
pub struct AssetServer {
    server: AssetLoader,
}

impl AssetServer {
    const PNG_MAGIC_NUMBER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    pub fn new(root: &'static str) -> Self {
        Self {
            server: AssetLoader::new(root),
        }
    }

    pub fn load<'a, T, P: Into<AssetPath<'a>>>(&self, path: &str) -> AssetHandle<T> {
        use std::{borrow, fs};

        let file_path = self.server.assets_path.join(path);
        let mut file = fs::File::open(&file_path).unwrap();
        let metadata = file.metadata().expect("Unable to read metadata");

        let mut buffer = vec![0; metadata.len() as usize];
        let read_bytes = file.read(&mut buffer).expect("buffer overflow");

        assert_eq!(
            read_bytes,
            metadata.len() as usize,
            "File: {:?} is corrupted",
            file_path
        );
        let magic_number = &buffer[0..8];
        assert_eq!(magic_number, AssetServer::PNG_MAGIC_NUMBER);

        AssetHandle {
            id: HandleId::from(AssetPath(borrow::Cow::Borrowed(&file_path))),
            marker: PhantomData::default(),
        }
    }
}
