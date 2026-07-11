mod kernels;

use std::sync::Arc;
use egui::{Color32, ColorImage, Image, ImageData, TextureHandle, TextureOptions};
use egui::load::SizedTexture;
pub use kernels::Compute;

pub struct App {
    image_handle: TextureHandle
}

impl App {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            image_handle: cc.egui_ctx.load_texture(
                "tex",
                ImageData::Color(Arc::new(ColorImage::filled([100, 100], Color32::BLACK))),
                TextureOptions::default()
            )
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.add(Image::from_texture(SizedTexture::from_handle(&self.image_handle)));

        egui::Area::new(egui::Id::new("settings"))
            .movable(true)
            .show(ui, |ui| {
                egui::Frame::NONE
                    .fill(egui::Color32::DARK_GRAY)
                    .stroke(egui::Stroke::new(3f32, Color32::GRAY))
                    .inner_margin(5)
                    .show(ui, |ui| {
                        let _ = ui.button("Test");
                        ui.label("Meow");
                    });
            });
    }
}