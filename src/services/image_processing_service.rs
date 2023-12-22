use std::{fs::{ReadDir, self, DirEntry}, sync::Arc, path::PathBuf};
use tracing::{info, error};
use crate::services::grpc_service::GrpcImagerService;
use super::grpc_service::GrpcClient;

pub async fn process_images_async(image_paths: ReadDir, result_path: Arc<String>, grpc_client: Arc<GrpcClient>) {
    let handles = image_paths.map(|path_result| {
        let result_path = result_path.clone();
        let client = grpc_client.clone();

        tokio::spawn(async move {
            let image_entry = match path_result {
                Ok(path) => path,
                Err(e) => {
                    error!("Failed to read directory entry: {}", e);
                    return;
                }
            };
            
            info!("Succesfully read image entry: {}", image_entry.file_name().to_str().unwrap());
            invert_image(&image_entry, &result_path, &client).await;
            blend_image(&image_entry, &result_path, &client).await
        })
    }).collect::<Vec<_>>();

    for handle in handles {
        handle.await.expect("Task panicked or failed");
    }
}

async fn blend_image(image_entry: &DirEntry, result_path: &str, client: &GrpcClient) {
    let image_bytes = match fs::read(image_entry.path()) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read image bytes: {}", e);
            return;
        }
    };

    let blend_bytes = match fs::read("./blend_image/blend-image.jpg") {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read blend image bytes: {}", e);
            return;
        }
    };

    let image_result = client.blend_images(image_bytes, blend_bytes, 123.2).await;

    match image_result {
        Ok(image) => {
            let result_path: PathBuf = [
                result_path.as_ref(),
                "Blend results",
                image_entry.file_name().to_str().unwrap(),
            ].iter().collect();
    
            match image.save(result_path) {
                Ok(_) => info!("Image saved successfully"),
                Err(e) => error!("Failed to save image: {}", e),
            }
        },
        Err(e) => {
            error!("Failed to process image: {}", e);
        },
    }
}

async fn invert_image(image_entry: &DirEntry, result_path: &str, client: &GrpcClient) {
    let image_bytes = match fs::read(image_entry.path()) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read image bytes: {}", e);
            return;
        }
    };

    let image_result = client.invert_image(image_bytes).await;

    match image_result {
        Ok(image) => {
            let result_path: PathBuf = [
                result_path.as_ref(),
                "Results",
                image_entry.file_name().to_str().unwrap(),
            ].iter().collect();
    
            match image.save(result_path) {
                Ok(_) => info!("Image saved successfully"),
                Err(e) => error!("Failed to save image: {}", e),
            }
        },
        Err(e) => {
            error!("Failed to process image: {}", e);
        },
    }
}