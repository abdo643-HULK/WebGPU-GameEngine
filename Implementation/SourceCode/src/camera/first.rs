use glam::{Mat3, Mat4, Vec2, Vec3, Vec3A};
use std::{
    f32::consts::PI,
    time::{SystemTime, UNIX_EPOCH},
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
};

use std::{f32, f64, time::Duration};

use crate::{app::Context, mesh::AABB};

#[derive(Debug)]
pub struct MyCamera {
    pub eye: glam::Vec3A,
    pub target: glam::Vec3A,
    pub up: glam::Vec3A,
    pub aspect: f32,
    pub fovy: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub center: glam::Vec3A,
    time: SystemTime,
}

impl MyCamera {
    pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array_2d(&[
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.5, 0.0],
        [0.0, 0.0, 0.5, 1.0],
    ]);

    pub fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        let time: SystemTime = SystemTime::now();
        MyCamera {
            eye: glam::vec3a(0.0, 0.0, 1.0),
            // eye: glam::vec3a(0.0, 0.7, 0.5),
            target: glam::vec3a(0.0, 0.0, 0.0),
            up: glam::Vec3A::Y,
            aspect: config.width as f32 / config.height as f32,
            fovy: 90.0,
            z_near: 0.1,
            // z_far: f32::INFINITY,
            z_far: 1000.0,
            center: Default::default(),
            time,
        }
    }

    pub fn animate_rotation(&self) {
        let radius = 10.0;
        let cam_x = self.time.elapsed().unwrap().as_secs_f32().sin() * radius;
        let cam_z = self.time.elapsed().unwrap().as_secs_f32().cos() * radius;
        let view = glam::Mat4::look_at_rh(
            glam::vec3(cam_x, 0.0, cam_z),
            glam::vec3(0.0, 0.0, 0.0),
            glam::vec3(0.0, 1.0, 0.0),
        );
    }

    // pub fn reset(&mut self, aabb: &AABB) -> Mat4 {
    //     let radius = aabb.radius();
    //     self.center = aabb.center;

    //     self.eye.x = aabb.center.x;
    //     self.eye.y = aabb.center.y;
    //     self.eye.z = aabb.center.z + radius;

    //     self.z_near = radius / 100.0;

    //     return glam::Mat4::perspective_infinite_rh_gl(self.fovy, self.aspect, self.z_near);
    // }

    pub fn build_view_projection_matrix(&self) -> (Mat4, Mat4) {
        // let angle: f32 = -55.0;
        // let axis = glam::vec3(1.0, 0.0, 0.0);
        // let model =
        //     Mat4::from_axis_angle(axis, angle.to_radians()) * Mat4::from_cols_array(&[1.0; 16]);

        let view = glam::Mat4::look_at_rh(self.eye.into(), self.target.into(), self.up.into());
        // let proj = glam::Mat4::perspective_infinite_rh_gl(self.fovy, self.aspect, self.z_near);
        let mut proj = glam::Mat4::perspective_rh_gl(
            self.fovy.to_radians(),
            self.aspect,
            self.z_near,
            self.z_far,
        );

        // proj.col_mut(1).y *= -1.; // Vulkan’s projected Y is inverted from OpenGL’s

        return (proj, view);
    }
}

impl Default for MyCamera {
    fn default() -> Self {
        Self {
            eye: Default::default(),
            target: Default::default(),
            up: Default::default(),
            aspect: Default::default(),
            fovy: Default::default(),
            z_near: Default::default(),
            z_far: Default::default(),
            center: Default::default(),
            time: SystemTime::now(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CameraController {
    pub camera: MyCamera,
    pub radius: f32,
    speed: f32,
    theta: f32,
    phi: f32,
    mouse_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    last_position: Option<PhysicalPosition<f64>>,
}

impl CameraController {
    pub fn new(camera: MyCamera, speed: f32) -> Self {
        Self {
            speed,
            camera,
            ..Default::default()
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent, context: &Context) -> bool {
        match event {
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    // println!("LineDelta: {x} - {y}");
                    false
                }
                winit::event::MouseScrollDelta::PixelDelta(PhysicalPosition { y, x }) => {
                    self.radius = clamp(
                        self.radius + *y as f32 * 0.001 * self.radius,
                        self.radius / 16.0,
                        f32::INFINITY,
                    );

                    // true
                    false
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                // let last_position = self.last_position.clone();
                // self.last_position = Some(position.clone());
                // if self.mouse_pressed {
                //     let movement_x = (position.x - last_position.unwrap_or(*position).x) as i32;
                //     let movement_y = (position.y - last_position.unwrap_or(*position).y) as i32;

                //     let PhysicalSize { width, height } = context.window().inner_size();
                //     self.theta -= (movement_x / width as i32) as f32 * f32::consts::PI * 2.0;

                //     self.phi = clamp(
                //         self.phi - (movement_y / height as i32) as f32 * f32::consts::PI,
                //         -f32::consts::PI / 2.0 + 0.1,
                //         f32::consts::PI / 2.0 - 0.1,
                //     );

                //     return true;
                // }

                false
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_pressed = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                false
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn animate(&mut self) {
        let forward = self.camera.target - self.camera.eye;
        let forward_norm = forward.normalize();

        let right = forward_norm.cross(self.camera.up);
        let forward = self.camera.target - self.camera.eye;
        let forward_mag = forward.length();

        self.camera.eye = self.camera.target - (forward - right * 0.1).normalize() * forward_mag;
    }

    pub fn update_camera(&mut self) -> Mat4 {
        let forward = self.camera.target - self.camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            self.camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            self.camera.eye -= forward_norm * self.speed;
        }

        // if self.is_left_pressed {
        //     let left = self.camera.target.cross(self.camera.up);
        //     let left = left.normalize();
        //     let left = left * self.speed;
        //     self.camera.eye += left;
        // }
        // if self.is_right_pressed {
        //     let right = self.camera.up.cross(self.camera.target);
        //     let right = right.normalize();
        //     let right = right * self.speed;
        //     self.camera.eye += right;
        // }

        let right = forward_norm.cross(self.camera.up);
        // Redo radius calc in case the fowrard/backward is pressed.
        let forward = self.camera.target - self.camera.eye;
        let forward_mag = forward.length();

        if self.is_right_pressed {
            self.camera.eye =
                self.camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            self.camera.eye =
                self.camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }

        let camera = &mut self.camera;
        // let translation = camera.eye - camera.center;
        // let rotation = Mat3::from_rotation_x(self.phi) * translation;
        // camera.eye = rotation + camera.center;
        // // println!("{translation}");

        // let translation = camera.eye - camera.center;
        // let rotation = Mat3::from_rotation_y(self.theta) * translation;
        // camera.eye = rotation + camera.center;
        // // println!("{translation}");

        return glam::Mat4::look_at_rh(camera.eye.into(), camera.center.into(), camera.up.into());
    }
}

pub fn clamp<T: PartialOrd>(n: T, min: T, max: T) -> T {
    if n < min {
        min
    } else if n > max {
        max
    } else {
        n
    }
}

pub trait PerspectiveGL {
    #[inline]
    fn perspective_infinite_rh_gl(
        fov_y_radians: f32,
        aspect_ratio: f32,
        z_near: f32,
    ) -> glam::Mat4 {
        use glam::{Mat4, Vec4};

        let f = 1.0 / (0.5 * fov_y_radians).tan();
        let a = f / aspect_ratio;
        let b = -1.0;
        let c = -2.0 * z_near;
        Mat4::from_cols(
            Vec4::new(a, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, b, -1.0),
            Vec4::new(0.0, 0.0, c, 0.0),
        )
    }
}

impl PerspectiveGL for Mat4 {}

struct WorldTrans {
    scale: f32,
    rotation: glam::Vec3A,
    position: glam::Vec3A,
}

impl WorldTrans {
    fn set_scale(&mut self, scale: f32) {
        self.scale = scale
    }

    fn set_position(&mut self, position: glam::Vec3A) {
        self.position = position
    }

    fn set_rotation(&mut self, rotation: glam::Vec3A) {
        self.rotation = rotation
    }

    fn rotate(&mut self, rotation: glam::Vec3A) {
        self.rotation += rotation
    }

    fn get_matrix(&self) -> glam::Mat4 {
        let scale_matrix = glam::Mat4::from_scale(glam::Vec3::from([self.scale; 3]));
        // let rotation_matrix =
        //     glam::Mat4::from_scale_rotation_translation(glam::Vec3::from([self.scale; 3]));
        let translate_matrix = glam::Mat4::from_translation(self.position.into());
        translate_matrix * scale_matrix
    }
}
