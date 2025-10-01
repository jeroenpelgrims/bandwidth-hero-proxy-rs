use bytes::Bytes;
use webp::Encoder;

pub fn convert_bytes_to_webp_with_alpha(
    image_bytes: Bytes,
    quality: f32,
) -> Result<Bytes, anyhow::Error> {
    let img = image::load_from_memory(&image_bytes)?;

    let rgba_img = match img {
        image::DynamicImage::ImageRgba8(rgba) => rgba,
        other => other.to_rgba8(),
    };
    let (width, height) = rgba_img.dimensions();

    // Create encoder from RGBA data
    let encoder = Encoder::from_rgba(&rgba_img, width, height);
    let webp_data = encoder.encode(quality);

    Ok(Bytes::from(webp_data.to_vec()))
}
