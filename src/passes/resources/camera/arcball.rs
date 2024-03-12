use std::f32::consts::PI;

use cgmath::{InnerSpace, Matrix4, MetricSpace, Point3, Rad, SquareMatrix, Vector3};

use crate::utils::input::Input;

#[derive(Debug)]
pub struct ArcballCameraController {
    pub offset: f32,
    pub target: Point3<f32>,
    pub position: Point3<f32>,
    pub screen_size: (u32, u32),
    pub view: Matrix4<f32>,
}

impl ArcballCameraController {
    const ZFAR: f32 = 1000.;
    const ZNEAR: f32 = 0.1;
    const FOVY: f32 = PI / 4.0;

    const INITIAL_OFFSET: f32 = 100.0;
    const DISTANCE_THRESHOLD: f32 = 0.1;

    #[rustfmt::skip]
    const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );

    pub fn from_config(config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            offset: Self::INITIAL_OFFSET,
            target: Point3::new(0.0, 0.0, 0.0),
            position: Point3::new(0.0, 0.0, Self::INITIAL_OFFSET),
            screen_size: (config.width, config.height),
            view: Matrix4::identity(),
        }
    }

    pub fn look_direction(&self) -> Vector3<f32> {
        (self.position - self.target).normalize()
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        Self::OPENGL_TO_WGPU_MATRIX
            * cgmath::perspective(Rad(Self::FOVY), self.get_aspect(), Self::ZNEAR, Self::ZFAR)
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        if target.distance(self.target) > Self::DISTANCE_THRESHOLD {
            self.target = target;
            self.set_position(target + Vector3::unit_z() * self.offset);
        }
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.screen_size = (config.width, config.height);
    }

    pub fn update(&mut self, input: &Input) {
        if input.mouse_pressed {
            self.update_on_mouse_drag(input.mouse_delta);
        }
        self.update_on_mouse_scroll(input.scroll);
    }

    pub fn update_on_mouse_scroll(&mut self, delta: f32) {
        self.offset = (self.offset + delta).clamp(0.3, Self::ZFAR / 2.);
        self.set_position(self.target + self.get_forward_vector() * self.offset);
    }

    pub fn update_on_mouse_drag(&mut self, mouse_delta: (f64, f64)) {
        let (dx, dy) = mouse_delta;
        let max_angle_x = PI / self.screen_size.0 as f32;
        let max_angle_y = 0.5 * PI / self.screen_size.1 as f32;

        let angle_x = -dx as f32 * max_angle_x;
        let angle_y = -dy as f32 * max_angle_y;

        let position = self.position.to_homogeneous();
        let target = self.target.to_homogeneous();

        let rotation_x = Matrix4::from_axis_angle(Vector3::unit_y(), Rad(angle_x));
        let rotation_y = Matrix4::from_axis_angle(self.get_right_vector(), Rad(angle_y));
        let new_position = rotation_x * rotation_y * (position - target) + target;

        self.set_position(Point3::from_homogeneous(new_position));

        // Handle the problem when the camera direction is the same as the up vector.
        let cos_angle = self.get_forward_vector().dot(Vector3::unit_y());
        if cos_angle.abs() > 0.99 {
            self.set_position(Point3::from_homogeneous(position));
        }
    }

    fn get_aspect(&self) -> f32 {
        self.screen_size.0 as f32 / self.screen_size.1 as f32
    }

    fn get_forward_vector(&self) -> Vector3<f32> {
        Vector3::new(self.view.x.z, self.view.y.z, self.view.z.z)
    }

    fn get_right_vector(&self) -> Vector3<f32> {
        Vector3::new(self.view.x.x, self.view.y.x, self.view.z.x)
    }

    fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
        self.view = Matrix4::look_at_rh(self.position, self.target, Vector3::unit_y());
    }
}
