
use {
    anyhow::{
        anyhow,
        Result,
    },
    multiarray::Array2D,
    std::{
        collections::HashMap,
        fmt::Debug,
        io::{
            BufRead,
            Lines,
        },
        iter::Peekable,
    },
};
#[derive(Clone,Copy,PartialEq,PartialOrd,Eq,Ord)]
pub enum MapCell {
    Space,
    Wall,
    Item,
}

#[derive(Debug)]
pub enum Direction {
    N,S,E,W
}

#[derive(Debug)]
pub struct Spawn {
    pub direction: Direction,
    pub x: usize,
    pub y: usize,
}
#[derive(Debug)]
pub struct Texture;

pub type RGB = (u8,u8,u8);
pub struct Map {
    pub resolution: (usize, usize),
    pub textures: [Texture; 4],
    pub sprite: Texture,
    pub floor: RGB,
    pub ceiling: RGB,
    pub data: Array2D<MapCell>,
    pub spawn: Spawn,
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
                Err(e) => { Err(anyhow!("io error: {}", e))? }
            };

        match line.chars().next() {
            None => { lines.next(); continue },
            Some(x) if x.is_alphabetic() => (),
            _ => break,
        }

        let (k,v) = line.split_at(line.find(' ').ok_or(anyhow!("incorrect header"))?);

        h.insert(k.to_owned(), v.trim_start().to_owned());

        lines.next();

    }

    Ok(h)
}

fn load_map<R: BufRead>(lines: Peekable<Lines<R>>) -> Result<(Array2D<MapCell>, Spawn)> {
    let lines: Result<Vec<String>> = lines
        .map(|i| i.map_err(|e| anyhow!("io error while reading map data {}",e)))
        .collect();

    let lines = lines?;

    let height = lines.len();
    let width = lines.iter().map(|s| s.len()).max().unwrap();
    let mut data = Array2D::new([height, width], MapCell::Wall);

    let mut spawn = None;

    for (y,row) in lines.iter().enumerate() {

        for (x, cell) in row.chars().enumerate() {

            let mut set_spawn = |d| {
                if spawn.replace(Spawn {x,y,direction: d}).is_none() {
                    Ok(())
                } else {
                    Err(anyhow!("More than one spawn point found"))
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
                other   => return Err(anyhow!("Invalid char in map: {}", other)),
            }
        }
    }

    let spawn = spawn.ok_or(anyhow!("map without spawn point"))?;

    Ok((data, spawn))
}

fn load_texture(_path: &str) -> Result<Texture> {
    Ok(Texture)
}

fn read_rgb(s: &str) -> Result<(u8,u8,u8)> {
    let tuple: Result<Vec<u8>> = s.split(',')
        .map(|s| s.parse().map_err(|_| anyhow!("rgb value not a u8")))
        .collect();
    let tuple = tuple?;
    if tuple.len() != 3 {
        return Err(anyhow!("Incorrect format for RGB value"));
    }

    Ok((tuple[0], tuple[1], tuple[2]))

}

impl Map {

    pub fn load<R: BufRead>(source: R) -> Result<Self> {


        let mut lines = source.lines().peekable();
        let h = headers(&mut lines)?;

        let resolution = {
            let rs = h.get("R").ok_or(anyhow!("no resolution"))?;
            let xy: Vec<_> = rs.split(' ').collect();
            if xy.len() != 2 {
                return Err(anyhow!("R line: two fields expected"));
            }
            (xy[0].parse()?, xy[1].parse()?)
        };

        let textures = [
            load_texture(h.get("NO").ok_or(anyhow!("NO texture missing"))?)?,
            load_texture(h.get("SO").ok_or(anyhow!("SO texture missing"))?)?,
            load_texture(h.get("WE").ok_or(anyhow!("WE texture missing"))?)?,
            load_texture(h.get("EA").ok_or(anyhow!("EA texture missing"))?)?,
        ];

        let sprite = load_texture(h.get("S").ok_or(anyhow!("S texture missing"))?)?;

        let floor = read_rgb(h.get("F").ok_or(anyhow!("no floor color"))?)?;
        let ceiling = read_rgb(h.get("C").ok_or(anyhow!("no ceiling color"))?)?;

        let (data, spawn) = load_map(lines)?;

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
R 1024 1024
NO nothingelse
SO nothingtoo
WE lksjldsf
EA lksjdflkasjfd

S whatever
F 220,100,0
C 225,30,0

      11111
      10001
11111110001
10020020001
10000N00001
10001111001
10001  1111
11111
";
    let m = Map::load(&data[..]);
    eprintln!("{:?}", m);
    assert!(m.is_ok());
}
