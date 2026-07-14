use egui::ColorImage;
use encase::UniformBuffer;
use std::sync::Arc;
use glam::{vec2, Vec3};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::BufferDescriptor;
use wgpu::{BufferAddress, BufferUsages, ComputePipeline, Device, Extent3d, MapMode, PollType, Queue, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension};

pub mod basic_sphere;
pub mod simple;

pub struct BasicSphere {
    device: Arc<Device>,
    pipeline: ComputePipeline,
}

impl BasicSphere {
    pub fn new(device: Arc<Device>) -> Self {
        use basic_sphere::compute::*;
        let pipeline = create_main_pipeline(&device);

        Self {
            device,
            pipeline,
        }
    }

    pub async fn run(&self, queue: &Queue, position: glam::Vec3, width: u32, height: u32, camera_normal: Vec3) -> ColorImage {
        let mut uniform_buffer_bytes = UniformBuffer::new(Vec::new());

        uniform_buffer_bytes
            .write(&basic_sphere::Uniforms {
                position,
                radius: 1.0,
                camera_size: vec2(3.0, 3.0),
                camera_normal,
            })
            .unwrap();

        let uniforms_buffer = &self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniforms"),
            contents: &uniform_buffer_bytes.into_inner(),
            usage: BufferUsages::UNIFORM | BufferUsages::STORAGE,
        });
        let texture = &self.device.create_texture(&TextureDescriptor {
            label: Some("texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Uint,
            usage: TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING,
            view_formats: &[TextureFormat::Rgba8Uint],
        });
        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: None,
            format: Some(TextureFormat::Rgba8Uint),
            dimension: Some(TextureViewDimension::D2),
            usage: None,
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let bind_group = basic_sphere::bind_groups::BindGroup0::from_bindings(
            &self.device,
            basic_sphere::bind_groups::BindGroupLayout0 {
                uniforms: uniforms_buffer.as_entire_buffer_binding(),
                out_data: &texture_view
            }
        );

        let mut encoder = self.device.create_command_encoder(&Default::default());
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        compute_pass.set_pipeline(&self.pipeline);
        basic_sphere::set_bind_groups(&mut compute_pass, &bind_group);
        compute_pass.dispatch_workgroups(
            width.div_ceil(basic_sphere::compute::MAIN_WORKGROUP_SIZE[0]),
            height.div_ceil(basic_sphere::compute::MAIN_WORKGROUP_SIZE[1]),
            1
        );
        drop(compute_pass);

        let bytes_per_row = (width * 4).div_ceil(256) * 256;

        let temp_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("temp"),
            size: (bytes_per_row * height) as BufferAddress,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &temp_buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            }
        );

        queue.submit([encoder.finish()]);

        let output_data = {
            let (tx, mut rx) = tokio::sync::broadcast::channel(1);

            temp_buffer.map_async(MapMode::Read, .., move |result| {
                tx.send(result).unwrap();
            });

            self.device.poll(PollType::wait_indefinitely()).unwrap();

            rx.recv().await.unwrap().unwrap();

            let raw_data = temp_buffer.get_mapped_range(..).unwrap();
            raw_data
                .to_vec()
                .chunks(bytes_per_row as usize)
                .flat_map(|row| { &row[0..(width * 4) as usize]})
                .copied()
                .collect::<Vec<u8>>()
        };

        ColorImage::from_rgba_unmultiplied([width as usize, height as usize], output_data.as_slice())
    }
}