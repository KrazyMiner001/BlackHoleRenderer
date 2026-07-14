use tokio::sync::Mutex;

mod kernels;
pub mod render_thread;
pub mod app;

pub struct RenderState {
    pub(crate) position: Mutex<glam::Vec3>,
    pub(crate) resolution: Mutex<(u32, u32)>,
    pub(crate) rotation: Mutex<(f32, f32)>,
}