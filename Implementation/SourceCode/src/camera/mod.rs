pub mod first;
pub mod orthographic;
pub mod perspective;

pub enum CameraType {
    Perspective,
    Orthographic,
}

pub trait Object3D {
    fn position(&self) -> &glam::Vec3 {
        &glam::Vec3::ZERO
    }
}

pub trait Camera: Object3D {
    fn get_type(&self) -> CameraType;
    fn get_zoom(&self) -> f32;
    fn get_view_matrix(&self) -> glam::Mat4;
    fn get_projection_matrix(&self) -> glam::Mat4;

    fn set_position(&mut self, position: glam::Vec3);
    fn set_zoom(&mut self, zoom: f32);

    fn update(&mut self);
}

pub struct GameCamera<T: Camera> {
    camera: T,
}

impl<T: Camera> GameCamera<T> {
    pub fn set_camera(&mut self, camera: T) {
        self.camera = camera;
    }
}
