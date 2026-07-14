use std::sync::Arc;
use crate::renderer::render_thread::{RendererToApp};
use egui::load::SizedTexture;
use egui::{Color32, ColorImage, DragValue, Image, Layout, TextureHandle, TextureOptions};
use tokio::sync::mpsc;
use crate::renderer::RenderState;

pub struct App {
    state: Arc<RenderState>,
    rx: mpsc::Receiver<RendererToApp>,
    texture: TextureHandle,
}

impl App {
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        state: Arc<RenderState>,
        rx: mpsc::Receiver<RendererToApp>,
    ) -> Self {
        Self {
            state,
            rx,
            texture: cc.egui_ctx.load_texture(
                "image",
                ColorImage::default(),
                TextureOptions::LINEAR,
            ),
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

                        ui.with_layout(Layout::left_to_right(egui::Align::TOP), |ui| {
                            let mut pos = self.state.position.blocking_lock();
                            ui.add(
                                DragValue::new(&mut pos.x).speed(0.1)
                            );
                            ui.add(
                                DragValue::new(&mut pos.y).speed(0.1)
                            );
                            ui.add(
                                DragValue::new(&mut pos.z).speed(0.1)
                            );
                        });

                        ui.with_layout(Layout::left_to_right(egui::Align::TOP), |ui| {
                            let mut rot = self.state.rotation.blocking_lock();
                            ui.add(
                                DragValue::new(&mut rot.0).speed(0.01)
                            );
                            ui.add(
                                DragValue::new(&mut rot.1).speed(0.01)
                            );
                        });
                    });
            });
    }
}