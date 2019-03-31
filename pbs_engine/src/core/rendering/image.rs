
pub enum ImageFormat {
    Rgb,
    Rgba,
    Luminance
}

pub struct Image<'a> {
    file_name: &'a str,
    pixels: &'a [u8],
    format: ImageFormat
}

impl<'a> Image<'a> {

    pub fn new(file_name: &str) -> Result<Image, String> {
        Ok(Image{
            file_name: "",
            pixels: &[],
            format: ImageFormat::Rgb
        })
    }

}
