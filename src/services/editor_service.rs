use image::{load_from_memory, ImageOutputFormat, DynamicImage, RgbaImage, Rgba};
use std::io::Cursor;
use tonic::{Request, Status, Response};
use tracing::info;

use imager::{InvertImageRequest, InvertImageResponse};
use imager::{BlendImageRequest, BlendImageResponse};
use imager::editor_service_server::EditorService;

pub mod imager {
    tonic::include_proto!("imager");
}

#[derive(Debug, Default)]
pub struct Editor {}

#[tonic::async_trait]
impl EditorService for Editor {
    async fn invert (
        &self,
        request: Request<InvertImageRequest>
    ) -> Result<Response<InvertImageResponse>, Status> {
        info!("Got a request: {:?}", request);

        let request_image_buffer = request.into_inner().image;

        let mut image = match load_from_memory(&request_image_buffer) {
            Ok(img) => img,
            Err(_) => return Err(Status::new(tonic::Code::InvalidArgument, "Invalid image data")),
        };

        invert_image_colors(&mut image);
        let mut response_image = Cursor::new(Vec::new());
        
        if let Err(_) = image.write_to(&mut response_image, ImageOutputFormat::Png) {
            return Err(Status::new(tonic::Code::Internal, "Failed to process the image"));
        }

        Ok(Response::new(InvertImageResponse { image: response_image.into_inner() }))
    }

    async fn blend_images(
        &self,
        request: Request<BlendImageRequest>) 
        -> Result<Response<BlendImageResponse>, Status> {
            let BlendImageRequest {
                first_image,
                second_image,
                alpha,
            } = request.into_inner();
        
            let first_image_buffer = load_and_convert_image(&first_image)?;
            let second_image_buffer = load_and_convert_image(&second_image)?;

            if first_image_buffer.dimensions() != second_image_buffer.dimensions() {
                return Err(Status::new(tonic::Code::InvalidArgument, "Image dimension mismatch"))
            }

            let blended = RgbaImage::from_fn(first_image_buffer.width(), first_image_buffer.height(), |x, y| {
                let pixel1 = first_image_buffer.get_pixel(x, y).0;
                let pixel2 = second_image_buffer.get_pixel(x, y).0;
                
                let blend_channel = |channel: usize| {
                    ((pixel1[channel] as f32 * alpha) + (pixel2[channel] as f32 * (1.0 - alpha))) as u8
                };
        
                Rgba([blend_channel(0), blend_channel(1), blend_channel(2), 255])
            });

            let mut response_image = Cursor::new(Vec::new());
            DynamicImage::ImageRgba8(blended)
            .write_to(&mut response_image, ImageOutputFormat::Png)
             .map_err(|_| Status::new(tonic::Code::Internal, "Failed to process the image"))?;
    
            Ok(Response::new(BlendImageResponse { image: response_image.into_inner() }))
    }
}

fn load_and_convert_image(image_data: &[u8]) -> Result<RgbaImage, Status> {
    load_from_memory(image_data)
        .map_err(|_| Status::new(tonic::Code::InvalidArgument, "Invalid image data"))
        .map(|img| img.into_rgba8())
}

fn invert_image_colors(image: &mut DynamicImage) {
    image.invert();
}