use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::BufferDescriptor;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BufferAddress, BufferUsages, ComputePipeline, ComputePipelineDescriptor, Device, DeviceDescriptor, MapMode, Queue, RequestAdapterOptions, ShaderModule, ShaderModuleDescriptor};

pub struct Compute {
    device: Arc<Device>,
    queue: Queue,
    test_shader: Shader,
}

impl Compute {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance.request_adapter(&RequestAdapterOptions::default()).await.unwrap();
        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await.unwrap();

        let device = Arc::new(device);

        let test_shader = Shader::new(wgpu::include_wgsl!("simple.wgsl"), device.clone());

        Self {
            device, queue, test_shader
        }
    }

    pub async fn test_shader(&self, data: u32) -> u32 {
        u32::from_le_bytes(
            self.test_shader.run(data.to_le_bytes(), &self.queue).await
        )
    }
}

struct Shader {
    module: ShaderModule,
    pipeline: ComputePipeline,
    device: Arc<Device>,
}

impl Shader {
    pub fn new(descriptor: ShaderModuleDescriptor, device: Arc<Device>) -> Self {
        let module = device.create_shader_module(descriptor);
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &module,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            module,
            pipeline,
            device,
        }
    }

    pub async fn run<const INPUT_SIZE: usize, const OUTPUT_SIZE: usize>(&self, input: [u8; INPUT_SIZE], queue: &Queue) -> [u8; OUTPUT_SIZE] {
        let input_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("input"),
            contents: &input,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE
        });
        let output_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("output"),
            size: OUTPUT_SIZE as BufferAddress,
            usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let temp_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("temp"),
            size: output_buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding()
                },
                BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding()
                }
            ]
        });


        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }

        encoder.copy_buffer_to_buffer(&output_buffer, 0, &temp_buffer, 0, output_buffer.size());

        queue.submit([encoder.finish()]);

        let output_data = {
            let (tx, mut rx) = tokio::sync::broadcast::channel(1);

            temp_buffer.map_async(MapMode::Read, .., move |result| {
                tx.send(result).unwrap();
            });

            self.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

            rx.recv().await.unwrap().unwrap();

            let data = temp_buffer.get_mapped_range(..).unwrap();
            data.to_vec()
        };

        temp_buffer.unmap();

        *output_data.as_array().unwrap()
    }
}