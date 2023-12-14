use tonic::transport::Server;
use tracing::{info, error, Level};
use tracing_subscriber;

pub mod services;
pub mod enums;

use crate::services::editor_service::{Editor, imager::editor_service_server::EditorServiceServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let addr = "[::1]:50051".parse()?;
    let img_service = Editor::default();

    info!("Starting server at {}", addr);

    Server::builder()
        .add_service(EditorServiceServer::new(img_service))
        .serve(addr)
        .await
        .map_err(|e| {
            error!("Server error: {}", e);
            e
        })?;
    Ok(())
}
