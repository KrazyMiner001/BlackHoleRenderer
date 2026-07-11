use std::thread;
use crate::renderer::App;

pub mod renderer;

fn main() -> Result<(), eframe::Error> {
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let compute = renderer::Compute::new().await;
            let num = compute.test_shader(5).await;
            println!("{num}")
        })
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Test",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc))))
    )
}
