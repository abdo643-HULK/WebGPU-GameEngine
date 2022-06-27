use crate::camera::{self, perspective::PerspectiveCamera, Camera, Object3D};

use std::f32;

use super::Controls;

#[derive(Debug, Clone, Copy)]
pub struct Spherical {
    pub radius: f32,
    pub phi: f32,
    pub theta: f32,
}

impl Spherical {
    pub fn new() -> Self {
        Self {
            radius: 1.0,
            phi: 0.0,   // polar angle
            theta: 0.0, // azimuthal angle
        }
    }
}

pub enum SphericalSetter {
    Vec(glam::Vec3),
    Values(f32, f32, f32),
    CartesianCoords(f32, f32, f32),
}

impl Spherical {
    pub fn set(&mut self, input: SphericalSetter) -> &mut Self {
        let (radius, phi, theta) = match input {
            SphericalSetter::Vec(input) => {
                let Spherical { radius, phi, theta } = Spherical::from(input);
                (radius, phi, theta)
            }
            SphericalSetter::CartesianCoords(x, y, z) => {
                let Spherical { radius, phi, theta } = Spherical::from(glam::vec3(x, y, z));
                (radius, phi, theta)
            }
            SphericalSetter::Values(radius, phi, theta) => (radius, phi, theta),
        };

        self.radius = radius;
        self.phi = phi;
        self.theta = theta;

        self
    }

    pub fn make_safe(&mut self) -> &mut Self {
        let eps = 0.000001_f32;
        self.phi = eps.max((f32::consts::PI - eps).min(self.phi));

        self
    }
}

impl From<glam::Vec3> for Spherical {
    fn from(input: glam::Vec3) -> Self {
        let radius = (input.x * input.x + input.y * input.y + input.z * input.z).sqrt();

        let (theta, phi) = if radius == 0.0 {
            (0.0, 0.0)
        } else {
            let theta = input.x.atan2(input.z);
            let phi = (input.y / radius).clamp(-1.0, 1.0).acos();
            (theta, phi)
        };

        Self { radius, phi, theta }
    }
}

///
///
///
/// OrbitControls
///
///
///
#[derive(Debug, Clone, PartialEq)]
pub enum AutoRotation {
    Disable,
    Enable(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pan {
    Disable,
    Enable(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Zoom {
    Disable,
    Enable(f32),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Damping {
    Disable,
    Enable(f32),
}

#[derive(Debug, Clone, PartialEq)]
enum ControllerState {
    None,
    Rotate,
    Dolly,
    Pan,
    TouchRotate,
    TouchPan,
    TouchDollyPan,
    TouchDollyRotate,
}

impl Default for ControllerState {
    fn default() -> Self {
        ControllerState::None
    }
}

#[derive(Debug, Clone)]
struct Controller {
    scale: f32,
    state: ControllerState,
    pan_offset: glam::Vec3A,

    rotate_start: glam::Vec2,
    rotate_end: glam::Vec2,
    rotate_delta: glam::Vec2,

    pan_start: glam::Vec2,
    pan_end: glam::Vec2,
    pan_delta: glam::Vec2,

    dolly_start: glam::Vec2,
    dolly_end: glam::Vec2,
    dolly_delta: glam::Vec2,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            scale: 1.0,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    target: glam::Vec3A,
    position: glam::Vec3A,
    zoom: f32,
}

#[derive(Debug, Clone)]
pub struct OrbitControls<T: camera::Camera = PerspectiveCamera> {
    pub object: T,
    target: glam::Vec3A,
    // How far you can dolly in and out ( PerspectiveCamera only )
    min_distance: f32,
    max_distance: f32,
    // How far you can zoom in and out ( OrthographicCamera only )
    min_zoom: f32,
    max_zoom: f32,
    // How far you can orbit vertically, upper and lower limits.
    // Range is 0 to Math.PI radians.
    min_polar_angle: f32,
    max_polar_angle: f32,
    // How far you can orbit horizontally, upper and lower limits.
    // If set, must be a sub-interval of the interval [ - Math.PI, Math.PI ].
    min_azimuth_angle: f32,
    max_azimuth_angle: f32,

    scale: f32,

    zoom: Zoom,
    auto_rotate: AutoRotation,
    panning: Pan,
    damping: Damping,

    spherical: Spherical,
    spherical_delta: Spherical,

    last_state: State,
    controller: Controller,
}

impl OrbitControls {
    const EPS: f32 = 0.000001;
    const TWO_PI: f32 = f32::consts::PI * 2.0;
}

impl<T: camera::Camera> OrbitControls<T> {
    fn get_distance(&self) -> f32 {
        glam::Vec3A::from(*self.object.position()).distance(self.target)
    }
}

impl<T: camera::Camera> OrbitControls<T> {
    fn get_auto_rotation_angle(&self) -> Option<f32> {
        match self.auto_rotate {
            AutoRotation::Disable => None,
            AutoRotation::Enable(speed) => Some(2.0 * f32::consts::PI / 60.0 / 60.0 * speed),
        }
    }

    fn get_zoom_scale(&self) -> Option<f32> {
        match self.zoom {
            Zoom::Disable => None,
            Zoom::Enable(speed) => Some(0.95_f32.powf(speed)),
        }
    }

    fn rotate_up(&mut self, angle: f32) {
        self.spherical_delta.phi -= angle;
    }

    fn rotate_left(&mut self, angle: f32) {
        self.spherical_delta.theta -= angle;
    }
}

impl<T: camera::Camera> OrbitControls<T> {
    pub fn get_polar_angle(&self) -> f32 {
        self.spherical.phi
    }

    pub fn get_azimuth_angle(&self) -> f32 {
        self.spherical.theta
    }
}

impl<T: camera::Camera> Controls for OrbitControls<T> {
    fn save(&mut self) {
        self.last_state.target = self.target.clone();
        self.last_state.position = (*self.object.position()).into();
        self.last_state.zoom = self.object.get_zoom();
    }

    fn reset(&mut self) {
        self.target = self.last_state.target.clone();
        self.object.set_position(self.last_state.position.into());
        self.object.set_zoom(self.last_state.zoom);
        self.object.update();
        // scope.dispatchEvent( _changeEvent );

        self.update();
        self.controller.state = ControllerState::None;
    }

    fn update(&mut self) {
        let OrbitControls {
            spherical,
            spherical_delta,
            controller,
            ..
        } = self;

        match self.damping {
            Damping::Disable => {
                spherical.theta += spherical_delta.theta;
                spherical.phi += spherical_delta.phi;
            }
            Damping::Enable(factor) => {
                spherical.theta += spherical_delta.theta * factor;
                spherical.phi += spherical_delta.phi * factor;
            }
        }

        // restrict phi to be between desired limits
        spherical.phi = self
            .min_polar_angle
            .max(self.max_polar_angle.min(spherical.phi));

        spherical.make_safe();

        spherical.radius *= controller.scale;

        // restrict radius to be between desired limits
        spherical.radius = self
            .min_distance
            .max(self.max_distance.min(spherical.radius));

        // move target to panned location
        match self.damping {
            Damping::Disable => {
                self.target += controller.pan_offset;
            }
            Damping::Enable(factor) => {
                self.target += controller.pan_offset * factor;
            }
        }
    }
}

impl Default for OrbitControls {
    fn default() -> Self {
        let camera = PerspectiveCamera::new(1920.0, 1080.0);
        let position = (*camera.position()).into();
        let zoom = camera.get_zoom();
        let target = glam::Vec3A::ZERO;

        Self {
            object: camera,
            target: target.clone(),

            min_distance: 0.0,
            max_distance: f32::INFINITY,
            min_zoom: 0.0,
            max_zoom: f32::INFINITY,
            min_polar_angle: 0.0,
            max_polar_angle: f32::consts::PI,
            min_azimuth_angle: f32::NEG_INFINITY,
            max_azimuth_angle: f32::INFINITY,

            zoom: Zoom::Enable(1.0),
            auto_rotate: AutoRotation::Enable(1.0),
            panning: Pan::Enable(1.0),
            damping: Damping::Disable,

            spherical: Spherical::new(),
            spherical_delta: Spherical::new(),

            scale: 1.9,
            last_state: State {
                target: glam::Vec3A::ZERO,
                position,
                zoom,
            },
            controller: Controller::default(),
        }
    }
}
