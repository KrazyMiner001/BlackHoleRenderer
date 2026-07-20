use crate::shader::Kerr;
use egui::ColorImage;
use glam::Vec3;
use std::sync::Arc;
use wgpu::{Device, DeviceDescriptor, Queue, RequestAdapterOptions};

pub struct Compute {
    device: Arc<Device>,
    queue: Queue,
    kerr_shader: Kerr,
}

impl Compute {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance.request_adapter(&RequestAdapterOptions::default()).await.unwrap();
        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await.unwrap();

        let device = Arc::new(device);

        let kerr_shader = Kerr::new(device.clone(), &queue);

        Self {
            device,
            queue,
            kerr_shader,
        }
    }

    pub async fn kerr_shader(&self, position: Vec3, width: u32, height: u32, camera_normal: Vec3, a_value: f32) -> ColorImage {
        self.kerr_shader.run(&self.queue, position, width, height, camera_normal, a_value).await
    }
}