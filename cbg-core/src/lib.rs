#[cfg(feature = "fun")]
pub mod fun;
#[cfg(feature = "imageops")]
pub mod imageops;
#[cfg(feature = "jobs")]
pub mod jobs;
#[cfg(feature = "osu")]
pub mod osu;
#[cfg(feature = "tetrio")]
pub mod tetrio;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
#[derive(Debug, Clone, Copy)]
// use this struct instead of serenity color
pub struct AverageColor {
    red: u8,
    green: u8,
    blue: u8,
}
impl AverageColor {
    #[cfg(feature = "discord")]
    #[must_use] 
    pub fn to_embed_color(&self) -> serenity::all::Color {
        serenity::all::Color::from_rgb(self.red, self.green, self.blue)
    }

    #[must_use] 
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub async fn from_image_url(url: &str) -> Result<AverageColor, Error> {
        use image::{GenericImageView, load_from_memory};
        let image_bytes = &reqwest::get(url).await?.bytes().await?;

        let image = load_from_memory(image_bytes)?;
        let (width, height) = image.dimensions();
        let mut red = 0u64;
        let mut green = 0u64;
        let mut blue = 0u64;

        // go through the pixels and calculate average color
        for x in 0..width {
            for y in 0..height {
                let pixel = image.get_pixel(x, y).0; // Get pixel (R, G, B, A)
                red += u64::from(pixel[0]);
                green += u64::from(pixel[1]);
                blue += u64::from(pixel[2]);
            }
        }

        // calculate the average
        let num_pixels = u64::from(width * height);
        red /= num_pixels;
        green /= num_pixels;
        blue /= num_pixels;

        Ok(AverageColor::new(red as u8, green as u8, blue as u8))
    }

    pub async fn from_bytes(image_bytes: Vec<u8>) -> Result<AverageColor, Error> {
        use image::{GenericImageView, load_from_memory};
        let image = load_from_memory(&image_bytes)?;
        let (width, height) = image.dimensions();
        let mut red = 0u64;
        let mut green = 0u64;
        let mut blue = 0u64;

        // go through the pixels and calculate average color
        for x in 0..width {
            for y in 0..height {
                let pixel = image.get_pixel(x, y).0; // get pixel (R, G, B, A)
                red += u64::from(pixel[0]);
                green += u64::from(pixel[1]);
                blue += u64::from(pixel[2]);
            }
        }

        // calculate the average
        let num_pixels = u64::from(width * height);
        red /= num_pixels;
        green /= num_pixels;
        blue /= num_pixels;

        Ok(AverageColor::new(red as u8, green as u8, blue as u8))
    }
}
