use imagemicro::services::editor_service::{Editor, imager::{editor_service_server::EditorService, InvertImageRequest, BlendImageRequest}};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tonic::Request;

fn create_invert_request() -> InvertImageRequest {
    let dummy_image_data = vec![0u8; 100 * 100];

    InvertImageRequest {
        image: dummy_image_data,
    }
}

fn create_blend_request() -> BlendImageRequest {
    let dummy_image_data1 = vec![0u8; 100 * 100];
    let dummy_image_data2 = vec![255u8; 100 * 100];

    BlendImageRequest {
        first_image: dummy_image_data1,
        second_image: dummy_image_data2,
        alpha: 0.5,
    }
}

fn benchmark_invert(c: &mut Criterion) {
    let editor = Editor::default();
    let request = create_invert_request();

    c.bench_function("invert", |b| {
        b.iter(|| {
            let _response = black_box(editor.invert(Request::new(request.clone())));
        });
    });
}

fn benchmark_blend_images(c: &mut Criterion) {
    let editor = Editor::default();
    let request = create_blend_request();

    c.bench_function("blend_images", |b| {
        b.iter(|| {
            let _response = black_box(editor.blend_images(Request::new(request.clone())));
        });
    });
}

criterion_group!(benches, benchmark_invert, benchmark_blend_images);
criterion_main!(benches);

