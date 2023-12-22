use image::{load_from_memory, ImageOutputFormat, DynamicImage, RgbaImage, Rgba};
use std::io::Cursor;
use tonic::{Request, Status, Response};
use tracing::info;
use rayon::prelude::*;

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
        let InvertImageRequest {
            image
        } = request.into_inner();

        let mut loaded_image = load_image(&image)?;

        invert_image_colors(&mut loaded_image);
        let mut response_image = Cursor::new(Vec::new());
        
        if let Err(_) = loaded_image.write_to(&mut response_image, ImageOutputFormat::Png) {
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
        
            let first_image_buffer = load_image(&first_image)?.into_rgba8();
            let second_image_buffer = load_image(&second_image)?.into_rgba8();

            if first_image_buffer.dimensions() != second_image_buffer.dimensions() {
                return Err(Status::new(tonic::Code::InvalidArgument, "Image dimension mismatch"))
            }

            let blended = blend_images_parallel(&first_image_buffer, &second_image_buffer, alpha);

            let mut response_image = Cursor::new(Vec::new());
            DynamicImage::ImageRgba8(blended)
            .write_to(&mut response_image, ImageOutputFormat::Png)
             .map_err(|_| Status::new(tonic::Code::Internal, "Failed to process the image"))?;
    
            Ok(Response::new(BlendImageResponse { image: response_image.into_inner() }))
    }
}

fn load_image(image_data: &[u8]) -> Result<DynamicImage, Status> {
    load_from_memory(image_data)
        .map_err(|_| Status::new(tonic::Code::InvalidArgument, "Invalid image data"))
        .map(|img| img)
}

fn invert_image_colors(image: &mut DynamicImage) {
    image.invert();
}

fn blend_images_parallel(first_image: &RgbaImage, second_image: &RgbaImage, alpha: f32) -> RgbaImage {
    let (width, height) = first_image.dimensions();

    let blended_pixels: Vec<_> = 
    first_image
        .pixels()
        .zip(second_image.pixels())
        .par_bridge()
        .map(|(pixel1, pixel2)| {
            blend_pixel(pixel1, pixel2, alpha)
        })
        .collect();

    RgbaImage::from_raw(width, height, blended_pixels.into_iter().flatten().collect()).unwrap()
}

fn blend_pixel(pixel1: &Rgba<u8>, pixel2: &Rgba<u8>, alpha: f32) -> [u8; 4] {
    let blend_channel = |channel: usize| {
        ((pixel1.0[channel] as f32 * alpha) + (pixel2.0[channel] as f32 * (1.0 - alpha))) as u8
    };

    [blend_channel(0), blend_channel(1), blend_channel(2), pixel1[3]]
}