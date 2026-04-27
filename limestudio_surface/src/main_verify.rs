use limestudio_surface::host_attach::standalone::run_standalone;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    println!("Starting LimeSurface Standalone Verification...");
    run_standalone().await
}
