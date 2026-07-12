use crate::renderer::App;
use glam::vec3;
use std::thread;

pub mod renderer;
mod shader;

fn main() -> Result<(), eframe::Error> {
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let compute = renderer::Compute::new().await;
            let num = compute.test_shader(5).await;
            let image = compute.sphere_shader::<8, 8>(vec3(0f32, 0f32, 0f32)).await;
            println!("{num}");
            println!("{:?}", image.pixels);
        })
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Test",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc))))
    )
}
