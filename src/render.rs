use std::{fs::File, io::BufReader};

use crate::{geometry::{Grid, Raycaster}, loader::{Direction, Map, MapCell}};
use crate::geometry::{Vector, v};
use image::{ImageBuffer, RgbImage, Rgb};


pub struct Render {
    pub pos: Vector,
    pub cam: Vector,
    fov: f64,
    pub buffer: RgbImage, 
    vfov: f64,
    height: f64
}

impl Render {

    pub fn spawn(map: &Map) -> Self {

        let fov = 30f64.to_radians();
        let res = map.resolution;
        let theta = map.spawn.direction.angle();

        Render { pos: v(map.spawn.x as f64 + 0.5,
                        map.spawn.y as f64 + 0.5)
               , cam: Vector::angle(theta)
               , fov
               , vfov: (fov.sin() * (res.1 as f64) / (res.0 as f64)).asin()
               , buffer: ImageBuffer::new(res.0 as u32, res.1 as u32)
               , height: 0.6
               }
    }



    pub fn render(&mut self, map: &Map) {

        let grid_bounds = map.data.extents();
        let grid = Grid { width: grid_bounds[1], height: grid_bounds[0] };
        let half_width: f64 = (self.buffer.width() as f64) / 2.0;
        let half_height: f64 = (self.buffer.height() as f64) / 2.0;
        let dx: Vector = self.cam.turn() * (self.fov.sin() / half_width);


        for x in 0..self.buffer.width() {

            let ray: Vector = self.cam + (dx * (x as f64 - half_width));

            let hit = Raycaster::new(self.pos, ray, grid)
                .filter(|h| map.data[[h.y, h.x]] == MapCell::Wall)
                .next().expect("Oh no! the impossible happened, no ray hits!");
            
            let vss = hit.distance.sqrt() * self.vfov.tan();

            let ceil: u32 = (half_height * (1.0 - (1.0 - self.height) / vss)).floor() as u32;
            let floor: u32 = (half_height * (1.0 + self.height / vss)).floor() as u32;

            for y in 0..ceil {
                self.buffer.put_pixel(x, y, map.ceiling);
            }

            //TODO actual texturing
            let pixel = match hit.direction {
                Direction::N => Rgb([0,0,255]),
                Direction::S => Rgb([255,255,0]),
                Direction::E => Rgb([0,255,0]),
                Direction::W => Rgb([255,0,0]),
            };

            for y in ceil..floor {
                self.buffer.put_pixel(x, y, pixel);
            }
            for y in floor..self.buffer.height() {
                self.buffer.put_pixel(x, y, map.floor);
            }

        }

    }

}