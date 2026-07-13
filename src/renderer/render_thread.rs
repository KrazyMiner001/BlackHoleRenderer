use crate::renderer::Compute;
use egui::ColorImage;
use glam::vec3;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task;

pub enum RendererToApp {
    NewImage(ColorImage),
}

pub enum AppToRenderer {
    PositionChanged(glam::Vec3),
    ResolutionChanged(u32, u32),
}

struct RenderThreadState {
    transmitter: Sender<RendererToApp>,
    compute: Compute,
    position: Mutex<glam::Vec3>,
    resolution: Mutex<(u32, u32)>,
}

pub struct RenderThread {
    state: Arc<RenderThreadState>
}

impl RenderThread {
    pub async fn new(tx: Sender<RendererToApp>, rx: Receiver<AppToRenderer>) -> Self {
        let state = Arc::new(RenderThreadState {
            transmitter: tx,
            compute: Compute::new().await,
            position: Mutex::new(vec3(0f32, 0f32, -5f32)),
            resolution: Mutex::new((100, 100))
        });
        let state_clone = state.clone();

        task::spawn(async move {
            let state = state_clone;
            let mut rx = rx;
            loop {
                let message = rx.recv().await;
                match message {
                    Some(AppToRenderer::PositionChanged(pos)) => {
                        *state.position.lock().await = pos;
                    },
                    Some(AppToRenderer::ResolutionChanged(x, y)) => {
                        *state.resolution.lock().await = (x, y);
                    },
                    _ => {}
                };
            };
        });

        Self {
            state
        }
    }

    pub async fn run(&self) {
        let pos = *self.state.position.lock().await;
        let (width, height) = *self.state.resolution.lock().await;

        let image = self.state.compute.sphere_shader(pos, width, height).await;

        let _ = self.state.transmitter.send(RendererToApp::NewImage(image)).await;
    }
}