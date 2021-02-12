use crate::texture::Texture;

use {
    anyhow::{
        anyhow,
        bail,
        Result,
        Context,
    },
    multiarray::{MultiArray, Array2D},
    std::{
        collections::HashMap,
        convert::TryInto,
        fmt::Debug,
        io::{
            BufRead,
            Lines,
        },
        iter::Peekable,
    },
    image::Rgb,
};
#[derive(Clone,Copy,PartialEq,PartialOrd,Eq,Ord)]
pub enum MapCell {
    Space,
    Wall,
    Item,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Direction {
    N,S,E,W
}

impl Direction {
    pub fn pointer(self) -> char {
        use Direction::*;
        match self {
            N => '^',
            S => 'v',
            W => '<',
            E => '>',
        }
    }

    pub fn angle(self) -> f64 {
        use Direction::*;
        match self {
            E => 0.0f64,
            S => 90.0,
            W => 180.0,
            N => 270.0,
        }.to_radians()
    }
}

#[derive(Debug,PartialEq,Eq)]
pub struct Spawn {
    pub direction: Direction,
    pub x: usize,
    pub y: usize,
}

pub type RGB = Rgb<u8>;

pub struct Map {
    pub resolution: (usize, usize),
    pub textures: [Texture; 4],
    pub sprite: Texture,
    pub floor: RGB,
    pub ceiling: RGB,
    pub data: Array2D<MapCell>,
    pub spawn: Spawn,
}

impl Map {
    pub fn texture(&self, d: Direction) -> &Texture {
        match d {
            Direction::N => &self.textures[0],
            Direction::S => &self.textures[1],
            Direction::W => &self.textures[2],
            Direction::E => &self.textures[3],
        }
    }
}

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "resolution {}x{}\n", self.resolution.0, self.resolution.1)?;
        write!(f, "floor: {:?}\n", self.floor)?;
        write!(f, "ceiling: {:?}\n", self.ceiling)?;
        write!(f, "spawn: {:?}\n", self.spawn)?;

        let (h,w) = (self.data.extents()[0], self.data.extents()[1]);

        write!(f, "map layout: {}x{}\n", h, w)?;

        for y in 0..h {
            for x in 0..w {
                let c = match self.data[[y,x]] {
                    _ if self.spawn.x == x && self.spawn.y == y => self.spawn.direction.pointer(),
                    MapCell::Space => '.',
                    MapCell::Item => '*',
                    MapCell::Wall => '#',
                };
                write!(f, "{}", c)?;
            }
            write!(f, "\n")?;
        }
        Ok(())

    }
}

fn headers<R: BufRead>(lines: &mut Peekable<Lines<R>>) -> Result<HashMap<String,String>> {
    let mut h = HashMap::new();

    loop{
        let line = match lines.peek()
            .ok_or(anyhow!("eof while reading headers"))?
            {
                Ok(s) => { s },
                Err(e) => { bail!("io error: {}", e) }
            };

        match line.chars().next() {
            None => { lines.next(); continue },
            Some(x) if x.is_alphabetic() => (),
            _ => break,
        }

        let (k,v) = line.split_at(line.find(' ').ok_or(anyhow!("incorrect header line format"))?);

        h.insert(k.to_owned(), v.trim().to_owned());

        lines.next();

    }

    Ok(h)
}

fn load_map<R: BufRead>(lines: Peekable<Lines<R>>) -> Result<(Array2D<MapCell>, Spawn)> {
    let lines: Result<Vec<String>> = lines
        .map(|i| i.map_err(|e| anyhow!("io error while reading map data {}",e)))
        .collect();

    let lines = lines.context("processing file header")?;

    let height = lines.len();
    let width = lines.iter().map(|s| s.len()).max().unwrap();
    let mut data = Array2D::new([height, width], MapCell::Wall);

    let mut spawn = None;

    for (y,row) in lines.iter().enumerate() {

        for (x, cell) in row.chars().enumerate() {

            let mut set_spawn = |d| {
                match spawn.replace(Spawn {x,y,direction: d}) {
                    None => Ok(()),
                    Some(s) =>
                        Err(anyhow!("More than one spawn point found ({:?} and {:?})", (s.y,s.x), (y, x)))
                }
            };

            data[[y,x]] = match cell {
                '0'     => MapCell::Space,
                '1'|' ' => MapCell::Wall,
                '2' => MapCell::Item,
                'N' => { set_spawn(Direction::N)?; MapCell::Space }
                'S' => { set_spawn(Direction::S)?; MapCell::Space }
                'E' => { set_spawn(Direction::E)?; MapCell::Space }
                'W' => { set_spawn(Direction::W)?; MapCell::Space }
                other   => bail!("Invalid char '{:?}' at location {:?}", other, (y,x)),
            }
        }
    }

    let spawn = spawn.ok_or(anyhow!("map without spawn point"))?;

    Ok((data, spawn))
}

fn read_rgb(s: &str) -> Result<RGB> {
    let pixel= s.split(',')
        .map(|s| Ok(s.parse()?))
        .collect::<Result<Vec<u8>>>()?
        .try_into()
        .map_err(|e| anyhow!("Unreadable pixel: {:?}", e))?;
    

    Ok(Rgb(pixel))
}

fn check_borders(data: &Array2D<MapCell>) -> Result<()> {

    let (h,w) = (data.extents()[0], data.extents()[1]);

    for y in &[0, h-1] {
        for x in 0..w {
            if data[[*y,x]] != MapCell::Wall {
                bail!("Edge cell isn't a wall at ({},{})", y, x);
            }
        }
    }

    for x in &[0, w-1] {
        for y in 0..h {
            if data[[y,*x]] != MapCell::Wall {
                bail!("Edge cell isn't a wall at ({},{})", y, x);
            }
        }
    }

    Ok(())
}

impl Map {

    pub fn load<R: BufRead>(source: R) -> Result<Self> {


        let mut lines = source.lines().peekable();
        let h = headers(&mut lines)?;

        let resolution = {
            let rs = h.get("R").ok_or(anyhow!("R header missing"))?;
            let xy: Vec<_> = rs.split(' ').collect();
            if xy.len() != 2 {
                return Err(anyhow!("R header: two fields expected"));
            }
            (xy[0].parse()?, xy[1].parse()?)
        };

        let textures = [
            Texture::load(h.get("NO").ok_or(anyhow!("NO texture missing"))?).context("loading NO texture")?,
            Texture::load(h.get("SO").ok_or(anyhow!("SO texture missing"))?).context("loading SO texture")?,
            Texture::load(h.get("WE").ok_or(anyhow!("WE texture missing"))?).context("loading WE texture")?,
            Texture::load(h.get("EA").ok_or(anyhow!("EA texture missing"))?).context("loading EA texture")?,
        ];

        let sprite = Texture::load(h.get("S").ok_or(anyhow!("S texture missing"))?)?;

        let floor = read_rgb(h.get("F").ok_or(anyhow!("no floor color"))?)?;
        let ceiling = read_rgb(h.get("C").ok_or(anyhow!("no ceiling color"))?)?;

        let (data, spawn) = load_map(lines)?;

        check_borders(&data)?;
        

        Ok(Self {
            resolution,
            textures,
            sprite,
            floor,
            ceiling,
            data, spawn
        })
    }
}

#[test]
fn test_loader() {
    let data = b"
R 640 480
NO tex/north.png
SO tex/south.png
WE tex/west.png
EA tex/east.png

S tex/sprite.png
F 220,100,0
C 225,30,0

 111
1101
12N1
1
";
    let m = Map::load(&data[..]).unwrap();

    let mut expected_data = Array2D::new([4,4], MapCell::Wall);

    expected_data[[1,2]] = MapCell::Space;
    expected_data[[2,1]] = MapCell::Item;
    expected_data[[2,2]] = MapCell::Space;

    assert_eq!(m.resolution, (640, 480));
    assert_eq!(m.floor, Rgb([220, 100, 0]));
    assert_eq!(m.ceiling, Rgb([225, 30, 0]));
    assert_eq!(m.spawn, Spawn { direction: Direction::N, x: 2, y: 2});
    assert!(m.data == expected_data);


}

#[test]
fn test_map_edge() {
    let data = b"
R 1024 1024
NO tex/north.png
SO tex/south.png
WE tex/west.png
EA tex/east.png

S tex/sprite.png
F 220,100,0
C 225,30,0

111
1N0
111

";
    let m = Map::load(&data[..]);
    assert!(m.is_err());
}