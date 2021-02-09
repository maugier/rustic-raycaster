use crate::{geometry::{Grid, Raycaster}, loader::{Direction, Map, MapCell}};
use crate::geometry::{Vector, v};
use image::{ImageBuffer, RgbaImage, Rgb};


pub struct Render {
    pos: Vector,
    cam: Vector,
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

        Render { pos: v(map.spawn.x as f64 + 0.5,
                        map.spawn.y as f64 + 0.5)
               , cam: v(theta.cos(), theta.sin())
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
            
            let ceil: usize = todo!();
            let floor: usize = todo!();

            for y in 0..self.buffer.height() {
                let pixel = todo!();
                self.buffer.put_pixel(x, y, pixel);
            }


        }

    }

}

