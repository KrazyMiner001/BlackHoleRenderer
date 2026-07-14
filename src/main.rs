use std::sync::{Arc};
use crate::renderer::render_thread::{RenderThread, RendererToApp};
use std::thread;
use glam::vec3;
use tokio::sync::{mpsc, Mutex};
use crate::renderer::app::App;
use crate::renderer::RenderState;

pub mod renderer;
mod shader;

fn main() -> Result<(), eframe::Error> {
    let (renderer_to_app_tx, renderer_to_app_rx) = mpsc::channel::<RendererToApp>(5);
    let state = Arc::new(
        RenderState {
            position: Mutex::new(vec3(0.0, 0.0, -5.0)),
            resolution: Mutex::new((100, 100))
        }
    );
    let state_clone = state.clone();

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let render_thread = RenderThread::new(renderer_to_app_tx, state_clone).await;

            loop {
                render_thread.run().await;
            }
        })
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Test",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, state.clone(), renderer_to_app_rx))))
    )
}
