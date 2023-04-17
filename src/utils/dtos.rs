#[derive(Default)]
pub struct LightData {
    pub follow_camera: Option<bool>,
    pub direction: Option<[f32; 3]>,
    pub color: Option<[f32; 3]>,
}
