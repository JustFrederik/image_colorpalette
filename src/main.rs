use anyhow::Result;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::ImageFormat;
use image::RgbImage;
use image::{self, DynamicImage};
use reqwest;
use std::collections::HashSet;
use std::time::Instant;

struct HandleImage {
    image: RgbImage,
    compressed_image: RgbImage,
    colors: Option<HashSet<[u8; 3]>>,
}

impl HandleImage {
    pub fn set(src: &str) -> Result<HandleImage> {
        let img = ImageReader::open(src)?.decode()?;
        Ok(Self {
            image: img.to_rgb8(),
            compressed_image: HandleImage::compressing_image(&img),
            colors: None,
        })
    }

    pub async fn set_from_web(src: &str) -> Result<HandleImage> {
        let result = reqwest::get(&src[..]).await?.bytes().await?;
        let image = image::load_from_memory_with_format(&result, ImageFormat::Jpeg).unwrap();
        Ok(Self {
            image: image.to_rgb8(),
            compressed_image: HandleImage::compressing_image(&image),
            colors: None,
        })
    }

    fn compressing_image(image: &DynamicImage) -> RgbImage {
        let width = image.width();
        let height = image.height();
        let ratio = 500 as f64 / HandleImage::smaller(width, height) as f64;
        image
            .resize(
                HandleImage::calculate(width, ratio),
                HandleImage::calculate(height, ratio),
                FilterType::Triangle,
            )
            .to_rgb8()
    }

    pub fn get_colors(&mut self) -> HashSet<[u8; 3]> {
        return match &self.colors {
            Some(value) => value.clone(),
            None => {
                let mut seen = HashSet::new();
                for pix in self.compressed_image.pixels() {
                    seen.insert([pix[0], pix[1], pix[2]]);
                }
                self.colors = Some(seen.clone());
                seen
            }
        };
    }

    pub fn check_grayscale(&mut self, threshold: u8) -> bool {
        return match &self.colors {
            Some(arr) => {
                let mut vec = vec![];
                for value in arr {
                    if HandleImage::get_difference(value[0], value[1]) < threshold
                        && HandleImage::get_difference(value[1], value[2]) < threshold
                        && HandleImage::get_difference(value[0], value[2]) < threshold
                    {
                        vec.push(true);
                    } else {
                        vec.push(false);
                    }
                }
                vec.iter().all(|&item| item == true)
            }
            None => {
                &self.get_colors();
                self.check_grayscale(threshold)
            }
        };
    }

    fn set_colors(mut self, colors: HashSet<[u8; 3]>) -> () {
        self.colors = Some(colors);
    }

    pub fn get_dimensions(&self) -> [u32; 2] {
        [self.image.width(), self.image.height()]
    }

    fn get_difference(f: u8, s: u8) -> u8 {
        if f < s {
            return s - f;
        }
        return f - s;
    }

    fn smaller(x: u32, y: u32) -> u32 {
        if x < y {
            return x;
        }
        return y;
    }

    fn calculate(u: u32, f: f64) -> u32 {
        (u as f64 * f).round() as u32
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let start = Instant::now();
    let url = "https://wallpapercave.com/wp/wp1848553.jpg";
    let mut res = HandleImage::set_from_web(&url).await.unwrap();
    println!("{}", start.elapsed().as_millis());
    let start = Instant::now();
    let _ = res.get_colors();
    println!("{}", start.elapsed().as_millis());
    Ok(())
}
