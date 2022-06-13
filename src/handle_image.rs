use anyhow::Result;
use image::{
    imageops::FilterType,
    io::Reader as ImageReader,
    ImageFormat, RgbImage, {self, DynamicImage},
};
use std::collections::HashSet;

#[allow(unused)]
pub struct HandleImage {
    pub image: RgbImage,
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
        let result = reqwest::get(src).await?.bytes().await?;
        let image = image::load_from_memory_with_format(&result, ImageFormat::Jpeg)?;
        Ok(Self {
            image: image.to_rgb8(),
            compressed_image: HandleImage::compressing_image(&image),
            colors: None,
        })
    }

    fn compressing_image(image: &DynamicImage) -> RgbImage {
        let width = image.width();
        let height = image.height();
        let mut ratio = 500.0 / HandleImage::smaller(width, height) as f64;
        if ratio > 1.0 {
            ratio = 1.0;
        }
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

    pub fn get_dominant_color(&mut self) -> [u8; 3] {
        return match &self.colors {
            Some(arr) => {
                let mut f = 0;
                let mut s = 0;
                let mut t = 0;

                for value in arr {
                    f += value[0] as u64;
                    s += value[1] as u64;
                    t += value[2] as u64;
                }
                [
                    (f as f32 / arr.len() as f32).round() as u8,
                    (s as f32 / arr.len() as f32).round() as u8,
                    (t as f32 / arr.len() as f32).round() as u8,
                ]
            }
            None => {
                let mut seen = HashSet::new();
                for pix in self.compressed_image.pixels() {
                    seen.insert([pix[0], pix[1], pix[2]]);
                }
                let _ = &self.get_colors();
                self.get_dominant_color()
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
                vec.iter().all(|&item| item)
            }
            None => {
                let _ = &self.get_colors();
                self.check_grayscale(threshold)
            }
        };
    }

    pub fn get_grayscale_threshold(&mut self) -> Option<u8> {
        return match &self.colors {
            Some(arr) => {
                let mut vec = vec![];
                for value in arr {
                    vec.push(HandleImage::get_difference(value[0], value[1]));
                    vec.push(HandleImage::get_difference(value[0], value[2]));
                    vec.push(HandleImage::get_difference(value[1], value[2]));
                }
                match vec.iter().max_by_key(|x| x.clone()) {
                    Some(v) => Some(*v),
                    None => None,
                }
            }
            None => {
                let _ = &self.get_colors();
                self.get_grayscale_threshold()
            }
        };
    }

    fn set_colors(mut self, colors: HashSet<[u8; 3]>) {
        self.colors = Some(colors);
    }

    pub fn get_dimensions(&self) -> [u32; 2] {
        [self.image.width(), self.image.height()]
    }

    fn get_difference(f: u8, s: u8) -> u8 {
        if f < s {
            return s - f;
        }
        f - s
    }

    fn smaller(x: u32, y: u32) -> u32 {
        if x < y {
            return x;
        }
        y
    }

    fn calculate(u: u32, f: f64) -> u32 {
        (u as f64 * f).round() as u32
    }
}
