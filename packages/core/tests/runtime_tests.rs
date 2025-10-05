use action_items_core::runtime::{DenoRuntime, RuntimeChannels, RuntimeConfig};

#[tokio::test]
async fn test_deno_runtime_initialization() {
    let config = RuntimeConfig::default();
    let channels = RuntimeChannels::default();

    let result = DenoRuntime::new(config, channels);
    match result {
        Ok(_runtime) => {
            println!("✅ Deno runtime initialized successfully!");
        },
        Err(e) => {
            panic!("❌ Failed to initialize Deno runtime: {e}");
        },
    }
}
