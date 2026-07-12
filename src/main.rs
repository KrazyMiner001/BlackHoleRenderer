use crate::renderer::render_thread::{RenderThread, RendererToApp};
use crate::renderer::App;
use render_thread::AppToRenderer;
use renderer::render_thread;
use std::thread;
use tokio::sync::mpsc;

pub mod renderer;
mod shader;

fn main() -> Result<(), eframe::Error> {
    let (app_to_renderer_tx, app_to_renderer_rx) = mpsc::channel::<AppToRenderer>(5);
    let (renderer_to_app_tx, renderer_to_app_rx) = mpsc::channel::<RendererToApp>(5);

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let render_thread = RenderThread::new(renderer_to_app_tx, app_to_renderer_rx).await;

            loop {
                render_thread.run().await;
            }
        })
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Test",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, app_to_renderer_tx, renderer_to_app_rx))))
    )
}
