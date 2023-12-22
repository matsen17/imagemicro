use std::{fs::{self}, env, sync::Arc};
use tracing::{info, error, Level};
use tracing_subscriber;
use services::{image_processing_service, grpc_service};

mod services;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
    .with_max_level(Level::INFO)
    .init();

    let image_paths = fs::read_dir("./images").unwrap();
    let mut current_exe_path = env::current_dir().expect("Failed to get current exe path");
    
    current_exe_path.pop();
    let current_exe_path = Arc::new(String::from(current_exe_path.to_str().unwrap()));

    // handle connect upon creation
    let mut client = grpc_service::GrpcClient::new();
    match client.connect().await {
            Ok(_) => info!("Connection established"),
            Err(e) => {
                error!("Failed to establish connection to server: {}", e);
                return;
            },
        }; 

    image_processing_service::process_images_async(image_paths, current_exe_path, Arc::new(client)).await;
}

// Test with 100 or more images (different sizes)
// Find bottlenecks and performance issues
// Benchmark functions (look for libs) 

// try rayon iter
// increase benchmarks