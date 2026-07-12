mod kernels;
pub mod render_thread;

use crate::renderer::render_thread::{AppToRenderer, RendererToApp};
use egui::load::SizedTexture;
use egui::{Color32, ColorImage, Image, TextureHandle, TextureOptions};
pub use kernels::Compute;
use tokio::sync::mpsc;

pub struct App {
    tx: mpsc::Sender<AppToRenderer>,
    rx: mpsc::Receiver<RendererToApp>,
    texture: TextureHandle
}

impl App {
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        tx: mpsc::Sender<AppToRenderer>,
        rx: mpsc::Receiver<RendererToApp>,
    ) -> Self {
        Self {
            tx,
            rx,
            texture: cc.egui_ctx.load_texture(
                "image",
                ColorImage::default(),
                TextureOptions::LINEAR,
            )
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if let Ok(message) = self.rx.try_recv() {
            match message {
                RendererToApp::NewImage(image) => {
                    self.texture.set(image, TextureOptions::LINEAR)
                }
            }
        }

        ui.add(
            Image::from_texture(
                SizedTexture::from_handle(&self.texture)
            ).fit_to_exact_size(ui.available_size())
        );

        egui::Area::new(egui::Id::new("settings"))
            .movable(true)
            .show(ui, |ui| {
                egui::Frame::NONE
                    .fill(Color32::DARK_GRAY)
                    .stroke(egui::Stroke::new(3f32, Color32::GRAY))
                    .inner_margin(5)
                    .show(ui, |ui| {
                        let _ = ui.button("Test");
                        ui.label("Meow");
                    });
            });
    }
}