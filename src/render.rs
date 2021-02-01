use crate::loader::{Direction, Map};
use image::{ImageBuffer, RgbaImage};

pub struct Render {
    x: f64,
    y: f64,
    cx: f64,
    cy: f64,
    fov: f64,
    buffer: RgbaImage, 
    vfov: f64,
    height: f64
}

impl Render {
    pub fn spawn(map: &Map) -> Self {

        let fov = 30f64.to_radians();
        let res = map.resolution;
        let theta = map.spawn.direction.angle();

        Render { x: map.spawn.x as f64
               , y: map.spawn.y as f64
               , cx: theta.cos()
               , cy: theta.sin()
               , fov
               , vfov: (fov.sin() * (res.1 as f64) / (res.0 as f64)).asin()
               , buffer: ImageBuffer::new(res.0 as u32, res.1 as u32)
               , height: 0.6
               }
    }



    pub fn render(&mut self, map: &Map) {

        for x in 0..self.buffer.width() {

            todo!()

        }

    }

}

