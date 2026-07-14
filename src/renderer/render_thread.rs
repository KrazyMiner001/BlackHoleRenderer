use egui::ColorImage;
use std::sync::Arc;
use glam::{vec3, Mat3};
use tokio::sync::mpsc::{Sender};
use crate::renderer::kernels::Compute;
use crate::renderer::RenderState;

pub enum RendererToApp {
    NewImage(ColorImage),
}

pub enum AppToRenderer {
    PositionChanged(glam::Vec3),
    ResolutionChanged(u32, u32),
}

pub struct RenderThread {
    transmitter: Sender<RendererToApp>,
    compute: Compute,
    state: Arc<RenderState>
}

impl RenderThread {
    pub async fn new(tx: Sender<RendererToApp>, state: Arc<RenderState>) -> Self {
        Self {
            transmitter: tx,
            compute: Compute::new().await,
            state,
        }
    }

    pub async fn run(&self) {
        let pos = *self.state.position.lock().await;
        let (width, height) = *self.state.resolution.lock().await;
        let (pitch, yaw) = *self.state.rotation.lock().await;
        
        let normal = Mat3::from_rotation_x(pitch) * Mat3::from_rotation_y(yaw) * vec3(0.0, 0.0, 1.0);

        let image = self.compute.sphere_shader(pos, width, height, normal).await;

        let _ = self.transmitter.send(RendererToApp::NewImage(image)).await;
    }
}