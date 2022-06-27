use super::{Camera, CameraType, Object3D};

#[derive(Debug, Clone, Copy)]
pub struct OrthographicCamera {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    z_near: f32,
    z_far: f32,
    zoom: f32,
}

impl Object3D for OrthographicCamera {}

impl Camera for OrthographicCamera {
    fn get_projection_matrix(&self) -> glam::Mat4 {
        // glam::Mat4::perspective_rh_gl(self.fov, self.aspect_ratio, self.z_near, self.z_far)
        glam::Mat4::default()
    }

    fn get_view_matrix(&self) -> glam::Mat4 {
        // let center = glam::Vec3::from(self.eye + self.target);
        // glam::Mat4::look_at_rh(self.eye.into(), center, self.up.into())
        glam::Mat4::default()
    }

    fn get_type(&self) -> CameraType {
        CameraType::Orthographic
    }

    fn get_zoom(&self) -> f32 {
        self.zoom
    }

    fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    fn set_position(&mut self, positions: glam::Vec3) {
        todo!()
    }

    fn update(&mut self) {}
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        Self {
            left: -1.0,
            right: 1.0,
            top: 1.0,
            bottom: -1.0,
            z_near: 0.1,
            z_far: 2000.0,
            zoom: 1.0,
        }
    }
}
