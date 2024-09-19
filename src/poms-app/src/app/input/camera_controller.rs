use std::f32::consts::PI;

use cgmath::{InnerSpace, Matrix4, MetricSpace, Point3, Rad, SquareMatrix, Vector3};

use super::mouse_input::MouseInput;

/// A typical ArcBall camera controller, reacts to mouse input to rotate and zoom the camera.
#[derive(Debug)]
pub(crate) struct CameraController {
    /// The Z-axis offset of the camera from the target.
    pub(crate) offset: f32,
    /// The position the camera is looking at.
    pub(crate) target: Point3<f32>,
    /// The current position of the camera.
    pub(crate) position: Point3<f32>,
    /// The current screen size. Used for calculating the aspect ratio of the camera.
    pub(crate) screen_size: (u32, u32),
    /// The view matrix of the camera, used for rendering.
    pub(crate) view_matrix: Matrix4<f32>,
}

impl CameraController {
    const ZFAR: f32 = 1000.;
    const ZNEAR: f32 = 0.1;
    const FOVY: f32 = PI / 4.0;

    const ZOOM_SPEED: f32 = 2.0;
    const INITIAL_OFFSET: f32 = 100.0;
    const DISTANCE_THRESHOLD: f32 = 0.1;

    #[rustfmt::skip]
    const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );

    /// Creates a new `CameraController` based on the given surface configuration.
    pub(crate) fn from_config(config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            offset: Self::INITIAL_OFFSET,
            target: Point3::new(0.0, 0.0, 0.0),
            position: Point3::new(0.0, 0.0, Self::INITIAL_OFFSET),
            screen_size: (config.width, config.height),
            view_matrix: Matrix4::identity(),
        }
    }

    /// Returns the current direction the camera is looking at.
    pub(crate) fn look_direction(&self) -> Vector3<f32> {
        (self.position - self.target).normalize()
    }

    /// Generates and returns the projection matrix based on the current camera settings.
    pub(crate) fn projection_matrix(&self) -> Matrix4<f32> {
        Self::OPENGL_TO_WGPU_MATRIX
            * cgmath::perspective(Rad(Self::FOVY), self.get_aspect(), Self::ZNEAR, Self::ZFAR)
    }

    /// Updates the target position the camera is focusing on, adjusting the camera's position accordingly.
    pub(crate) fn set_target(&mut self, target: Point3<f32>) {
        if target.distance(self.target) > Self::DISTANCE_THRESHOLD {
            self.target = target;
            self.set_position(target + Vector3::unit_z() * self.offset);
        }
    }

    /// Updates the camera's screen size based on the new surface configuration.
    pub(crate) fn resize(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.screen_size = (config.width, config.height);
    }

    /// Updates the camera's position based on the user's mouse input. Call this every frame.
    pub(crate) fn update(&mut self, input: &MouseInput) {
        if input.mouse_pressed {
            self.update_on_mouse_drag(input.mouse_delta);
        }
        self.update_on_mouse_scroll(input.scroll);
    }

    /// Adjusts the camera's distance from the target based on the scroll input.
    fn update_on_mouse_scroll(&mut self, delta: f32) {
        if delta != 0. {
            self.offset =
                (self.offset + delta.signum() * Self::ZOOM_SPEED).clamp(0.3, Self::ZFAR / 2.);
            self.set_position(self.target + self.get_forward_vector() * self.offset);
        }
    }

    /// Rotates the camera around the target based on the mouse drag input.
    fn update_on_mouse_drag(&mut self, mouse_delta: (f64, f64)) {
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
        Vector3::new(
            self.view_matrix.x.z,
            self.view_matrix.y.z,
            self.view_matrix.z.z,
        )
    }

    fn get_right_vector(&self) -> Vector3<f32> {
        Vector3::new(
            self.view_matrix.x.x,
            self.view_matrix.y.x,
            self.view_matrix.z.x,
        )
    }

    fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
        self.view_matrix = Matrix4::look_at_rh(self.position, self.target, Vector3::unit_y());
    }
}
