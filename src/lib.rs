use image::ImageEncoder;
use image::PixelWithColorType;
use once_cell::sync::Lazy;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use regex::Regex;
use std::borrow::Cow;

static WHITESPACES_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

trait SvgDataUriUtils: AsRef<str> {
    fn trim_byte_order_mark(&self) -> &str {
        let string = self.as_ref();
        match string.chars().next() {
            Some('\u{FEFF}') => &string[1..],
            _ => string,
        }
    }

    fn collapse_whitespace(&self) -> Cow<str> {
        WHITESPACES_REGEX.replace_all(self.as_ref(), " ")
    }

    fn encode_uri_components(&self) -> Cow<str> {
        let string = self.as_ref();
        utf8_percent_encode(string, NON_ALPHANUMERIC).collect()
    }
}

impl<T: AsRef<str>> SvgDataUriUtils for T {}

pub fn svg_str_to_data_uri(svg: impl AsRef<str>) -> String {
    format!(
        "data:image/svg+xml,{}",
        svg.trim_byte_order_mark()
            .trim()
            .collapse_whitespace()
            .encode_uri_components()
    )
}

pub fn image_to_png_data_uri<T>(image: &T) -> image::ImageResult<String>
where
    T: image::GenericImageView + image::EncodableLayout,
    <T as image::GenericImageView>::Pixel: image::PixelWithColorType,
{
    let mut buffer = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
    encoder.write_image(
        image.as_bytes(),
        image.width(),
        image.height(),
        <T as image::GenericImageView>::Pixel::COLOR_TYPE,
    )?;
    Ok([
        "data:",
        mime::IMAGE_PNG.as_ref(),
        ";base64,",
        base64::encode(&buffer).as_ref(),
    ]
    .iter()
    .cloned()
    .collect())
}

pub fn image_to_jpeg_data_uri<T>(image: &T, quality: u8) -> image::ImageResult<String>
where
    T: image::GenericImageView,
    <T as image::GenericImageView>::Pixel: image::PixelWithColorType,
{
    let mut buffer = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);
    encoder.encode_image(image)?;
    Ok([
        "data:",
        mime::IMAGE_JPEG.as_ref(),
        ";base64,",
        base64::encode(&buffer).as_ref(),
    ]
    .iter()
    .cloned()
    .collect())
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn full_test() {
        let svg = r##"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 50 50">
                <path d="M22 38V51L32 32l19-19v12C44 26 43 10 38 0 52 15 49 39 22 38z"/>
            </svg>"##;
        let expected = r#"data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww%2Ew3%2Eorg%2F2000%2Fsvg%22%20viewBox%3D%220%200%2050%2050%22%3E%20%3Cpath%20d%3D%22M22%2038V51L32%2032l19%2D19v12C44%2026%2043%2010%2038%200%2052%2015%2049%2039%2022%2038z%22%2F%3E%20%3C%2Fsvg%3E"#;
        let result = svg_str_to_data_uri(svg);
        assert_eq!(result, expected);
    }
}
