use std::error::Error;
use image::{load_from_memory, DynamicImage};
use tonic::transport::Channel;
use tracing::info;

use grpc_imager::{InvertImageRequest, BlendImageRequest};
use grpc_imager::editor_service_client::EditorServiceClient; 

pub mod grpc_imager {
    tonic::include_proto!("imager");
}
pub struct GrpcClient {
    pub client: Option<EditorServiceClient<Channel>>
}

impl GrpcClient {
    pub fn new() -> Self {
        Self { client: None}
    }

    // dont await
    // also check if connection drops 
    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        self.client = Some(EditorServiceClient::connect("http://[::1]:50051").await?);
        Ok(())
    }
}

#[tonic::async_trait]
pub trait GrpcImagerService {
    async fn invert_image(&self, image_bytes: Vec<u8>) -> Result<DynamicImage, Box<dyn Error>>;
    async fn blend_images(&self, first_image_bytes: Vec<u8>, second_image_bytes: Vec<u8>, image_alpha: f32) -> Result<DynamicImage, Box<dyn Error>>;
}

#[tonic::async_trait]
impl GrpcImagerService for GrpcClient {
    async fn invert_image(&self, image_bytes: Vec<u8>) ->  Result<DynamicImage, Box<dyn Error>>  {
        if let Some(client) = &self.client {
            let request = tonic::Request::new(InvertImageRequest {
                image: image_bytes
            });

            info!("Sending request: invert");
            let response = client.clone().invert(request).await?;
            info!("Received response: invert");

            let response_buffer = response.into_inner().image;
            
            return match load_from_memory(&response_buffer) {
                Ok(img) => Ok(img),
                Err(e) => Err(Box::new(e) as Box<dyn Error>)
            };
        } else {
            Err("Client not connected".into())
        }
    }

    async fn blend_images(&self, first_image_bytes: Vec<u8>, second_image_bytes: Vec<u8>, image_alpha: f32)  ->  Result<DynamicImage, Box<dyn Error>> {
        if let Some(client) = &self.client {
            let request = tonic::Request::new(BlendImageRequest { 
                first_image: first_image_bytes, 
                second_image: second_image_bytes, 
                alpha: image_alpha
            });

            info!("Sending request: blend");
            let response = client.clone().blend_images(request).await?;
            info!("Received response: blend");

            let response_buffer = response.into_inner().image;
            
            return match load_from_memory(&response_buffer) {
                Ok(img) => Ok(img),
                Err(e) => Err(Box::new(e) as Box<dyn Error>)
            };
        } else {
            Err("Client not connected".into())
        }
    }
}
