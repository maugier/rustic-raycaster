use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Result, bail};
use image::{DynamicImage, Rgb, RgbImage};


pub struct Texture {
    inner: RgbImage
}


impl Texture {

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let source = BufReader::new(File::open(path)?);
        let img  = image::load(source, image::ImageFormat::Png)?;
        match img {
            DynamicImage::ImageRgb8(inner) => Ok(Texture { inner }),
            _ => bail!("Unsupported texture format"),
        }
    } 

    pub fn get(&self, (x,y): (f64, f64)) -> Rgb<u8> {
        let x = (x * (self.inner.width() as f64).floor()) as u32;
        let y = (y * (self.inner.height() as f64).floor()) as u32;

        let x = x % self.inner.width();
        let y = y % self.inner.height();


        *self.inner.get_pixel(x, y)
    }

}