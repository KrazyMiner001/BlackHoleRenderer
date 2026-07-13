mod kernels;
pub mod render_thread;

use crate::renderer::render_thread::{AppToRenderer, RendererToApp};
use egui::load::SizedTexture;
use egui::{Color32, ColorImage, DragValue, Image, TextureHandle, TextureOptions};
use glam::{vec3, Vec3};
pub use kernels::Compute;
use tokio::sync::mpsc;

pub struct App {
    tx: mpsc::Sender<AppToRenderer>,
    rx: mpsc::Receiver<RendererToApp>,
    texture: TextureHandle,
    position: Vec3,
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
            ),
            position: vec3(0.0, 0.0, -5.0)
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

                        ui.add(
                            DragValue::from_get_set(|num| {
                                match num {
                                    Some(value) => {
                                        self.position.x = value as f32;
                                        self.tx.blocking_send(AppToRenderer::PositionChanged(self.position)).unwrap();
                                        value
                                    }
                                    None => {
                                        self.position.x as f64
                                    }
                                }
                            }).speed(0.1)
                        );
                        ui.add(
                            DragValue::from_get_set(|num| {
                                match num {
                                    Some(value) => {
                                        self.position.y = value as f32;
                                        self.tx.blocking_send(AppToRenderer::PositionChanged(self.position)).unwrap();
                                        value
                                    }
                                    None => {
                                        self.position.y as f64
                                    }
                                }
                            }).speed(0.1)
                        );
                        ui.add(
                            DragValue::from_get_set(|num| {
                                match num {
                                    Some(value) => {
                                        self.position.z = value as f32;
                                        self.tx.blocking_send(AppToRenderer::PositionChanged(self.position)).unwrap();
                                        value
                                    }
                                    None => {
                                        self.position.z as f64
                                    }
                                }
                            }).speed(0.1)
                        );
                    });
            });
    }
}