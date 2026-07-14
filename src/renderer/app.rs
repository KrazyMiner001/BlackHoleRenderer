use std::sync::Arc;
use crate::renderer::render_thread::{RendererToApp};
use egui::load::SizedTexture;
use egui::{Color32, ColorImage, DragValue, Image, TextureHandle, TextureOptions};
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

                        ui.add(
                            DragValue::from_get_set(|num| {
                                match num {
                                    Some(value) => {
                                        self.state.position.blocking_lock().x = value as f32;
                                        value
                                    }
                                    None => {
                                        self.state.position.blocking_lock().x as f64
                                    }
                                }
                            }).speed(0.1)
                        );
                        ui.add(
                            DragValue::from_get_set(|num| {
                                match num {
                                    Some(value) => {
                                        self.state.position.blocking_lock().y = value as f32;
                                        value
                                    }
                                    None => {
                                        self.state.position.blocking_lock().y as f64
                                    }
                                }
                            }).speed(0.1)
                        );
                        ui.add(
                            DragValue::from_get_set(|num| {
                                match num {
                                    Some(value) => {
                                        self.state.position.blocking_lock().z = value as f32;
                                        value
                                    }
                                    None => {
                                        self.state.position.blocking_lock().z as f64
                                    }
                                }
                            }).speed(0.1)
                        );
                    });
            });
    }
}