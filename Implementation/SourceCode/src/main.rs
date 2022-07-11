// #![feature(dropck_eyepatch)]

pub mod app;
pub mod assets;
pub mod camera;
pub mod controls;
pub mod loader;
pub mod mesh;
pub mod model;
pub mod triangle;
pub mod triangle_simple;
pub mod vertex;

use app::{App, AppOptions};
use loader::GLTFLoader;

pub const WINDOW_TITLE: &str = "WebGPU Game";
pub const ASSETS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let (title, path) = (
        "Armadillo",
        format!("{ASSETS_PATH}/models/armadillo/Armadillo.glb"),
    );
    // let (title, path) = (
    //     "Dragon XYZRGB",
    //     format!("{ASSETS_PATH}/models/dragon-xyzrgb/Dragon_xyzrgb.glb"),
    // );
    let (title, path) = ("Dragon", format!("{ASSETS_PATH}/models/dragon/Dragon.glb"));
    // let (title, path) = ("Cube", format!("{ASSETS_PATH}/models/cube/Cube.glb"));
    // let (title, path) = (
    //     "Charizard",
    //     format!("{ASSETS_PATH}/models/charizard/Charizard.gltf"),
    // );

    let loader = GLTFLoader::new(path, loader::AnimationLoop::Disable);

    // let mut controls = controls::orbital::OrbitControls::default();
    // controls.object.update();

    App::init(AppOptions {
        title: Some(title),
        ..Default::default()
    })
    .add_system(Box::new(loader))
    .run();

    // triangle::run();
    // triangle_simple::run();

    return Ok(());
}

// pub struct Inspector<'a, T: std::fmt::Debug>(&'a T);

// unsafe impl<'a, #[may_dangle] T: std::fmt::Debug> Drop for Inspector<'a, T> {
//     fn drop(&mut self) {
//         println!("Inspector({:#?})", self.0);
//     }
// }
