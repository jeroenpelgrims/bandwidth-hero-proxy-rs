use bytes::Bytes;
use image::{DynamicImage, RgbaImage, imageops};
use webp::Encoder;

pub fn compress_image(
    image_bytes: Bytes,
    quality: f32,
    monochrome: bool,
    webp: bool,
) -> Result<Bytes, anyhow::Error> {
    let img = image::load_from_memory(&image_bytes)?.to_rgba8();

    let img: RgbaImage = match monochrome {
        true => {
            let grayscale = imageops::grayscale(&img);
            let rgba = DynamicImage::ImageLuma8(grayscale).to_rgba8();
            RgbaImage::from(rgba)
        }
        false => img,
    };

    let bytes = match webp {
        true => encode_webp(img, quality),
        false => encode_jpeg(img, quality),
    };
    Ok(bytes)
}

fn encode_webp(img: RgbaImage, quality: f32) -> Bytes {
    let (width, height) = img.dimensions();
    let encoder = Encoder::from_rgba(&img, width, height);
    let webp_data = encoder.encode(quality);
    Bytes::from(webp_data.to_vec())
}

fn encode_jpeg(img: RgbaImage, quality: f32) -> Bytes {
    let mut buf = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality as u8);
    encoder.encode_image(&img).unwrap();
    Bytes::from(buf)
}
