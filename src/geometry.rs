use {
    std::{
        iter::{Peekable, Rev},
        ops::Range,
        ops::Add,
        ops::Mul
    },
    either::Either,
};


use crate::loader::Direction;

#[derive(Clone,Copy,PartialEq,Debug)]
pub struct Vector { x: f64, y: f64 }

impl Vector {
    pub fn flip(self) -> Self {
        Vector { x: self.y, y: self.x }
    }

    pub fn turn(self) -> Self {
        Vector { x: -self.y, y: self.x}
    }

    pub fn angle(theta: f64) -> Self {
        Vector { x: theta.cos(), y: theta.sin() }
    }

    pub fn squared_norm(self) -> f64 {
        self.x*self.x + self.y*self.y
    }

    pub fn squared_distance(&self, rhs: &Self) -> f64 {
        let dx = rhs.x - self.x;
        let dy = rhs.y - self.y;
        dx*dx + dy*dy
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Self) -> Self::Output {
        Vector { x: self.x + rhs.x,
                 y: self.y + rhs.y }
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        Vector {x: self.x * rhs, y: self.y * rhs }
    }
}

impl From<(f64,f64)> for Vector {
    fn from((x,y):(f64,f64)) -> Self { Vector {x,y} }
}

#[inline(always)]
pub fn v(x: f64, y: f64) -> Vector { Vector {x,y} }

#[derive(Clone,Copy,PartialEq,Eq)]
pub struct Grid { pub height: usize, pub width: usize }

impl Grid {
    fn contains(&self, x: usize, y: usize) -> bool {
        (0..self.height).contains(&y) &&
        (0..self.width).contains(&x)
    }
}

type Position = Vector;

type DynRange = Either<Range<usize>,Rev<Range<usize>>>;

/// A hit from a ray to a wall. Contains the (integer) coordinates of the 
/// map location that was hit, a direction indicating which face was hit,
/// the horizontal coordinate of the hit within the face (for texture rendering),
/// and the squared distance from the camera.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Hit {
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
    pub position: f64,
    pub distance: f64,
}

struct Interceptor {
    p: Position,
    slope: f64,
    range: DynRange,
}

/// An iterator over all integers in the interval between start and either 0 or `size`,
/// depending on the sign of `direction`. This is meant to compute all potentially 
/// intersecting grid lines for a single axis.
fn bounded_iterator(start: f64, direction: f64, size: usize) -> DynRange {
    
    if size == 0 || start < 0.0 {
        return Either::Left(1..0); // empty range
    }
    
    let ceil = start.ceil() as usize;

    if direction < 0.0 {
            Either::Right((1..ceil).rev())
    } else {
            Either::Left(ceil .. size)
    }
}

/// An iterator of all the intersection points between an arbitrary semi-line
/// given by the start point `p` and direction vector `d`, and a grid of vertical
/// lines at integer coordinates [1..(size-1)]
impl Interceptor {
    fn new(p: Position, d: Vector, size: usize) -> Self {

        let range = bounded_iterator(p.x, d.x, size);
        let slope = d.y / d.x;

        Self { p, slope, range }
    }
}

impl Iterator for Interceptor {
    type Item = (usize,f64,Position);

    fn next(&mut self) -> Option<(usize,f64,Position)> {
        self.range
            .next()
            .and_then(|xi| {
                let x = xi as f64;
                let y = self.p.y + (x - self.p.x) * self.slope;
                let v = Vector{x,y};
                Some((xi, v.squared_distance(&self.p), v))               
            })
    }
}

// An iterator of all the wall hits for a given position, direction and grid size
pub struct Raycaster {
    g: Grid,
    d: Vector,
    xint: Peekable<Interceptor>,
    yint: Peekable<Interceptor>,
}

impl Raycaster {
    pub fn new(p: Position, d: Vector, g: Grid)  -> Self {
        Raycaster {
            g,d,
            xint: Interceptor::new(p,d,g.width).peekable(),
            yint: Interceptor::new(p.flip(), d.flip(), g.height).peekable(),
        }
    }
}

impl Iterator for Raycaster {
    type Item = Hit;

    fn next(&mut self) -> Option<Self::Item> {

        let px = self.xint.peek();
        let py = self.yint.peek();

        //eprintln!("px = {:?}, py = {:?}", px, py);

        let xhit = match (px,py) {
            (None,None) => return None,
            (Some(px), Some(py)) =>
                { px.1 <= py.1 },
            (Some(_),None) => true,
            (None,Some(_)) => false,
        };

        let (x,y,position,direction, distance);

        
        if xhit {
            let (xi,d, p) = self.xint.next().unwrap();
            let fy = p.y.floor();
            
            y = fy as usize;
            position = p.y - fy;
            distance = d;
            if self.d.x < 0.0 {
                x = xi - 1; 
                direction = Direction::E 
            } else { 
                x = xi;
                direction = Direction::W
            }
            
        } else {
            let (yi,d, p) = self.yint.next().unwrap();
            let p = p.flip();
            let fx = p.x.floor();
            
            x = fx as usize;
            position = p.x - fx;
            distance = d;
            if self.d.y < 0.0 { 
                y = yi - 1;
                direction = Direction::S
            } else {
                y = yi; 
                direction = Direction::N
            };
        
        }

        if !self.g.contains(x,y) || distance.is_infinite() {
            return None;
        }
 
        Some(Hit {x,y,position,direction,distance})

    }
}

#[test]
fn test_interceptor() {
    
    let start = v(0.5, 1.5);
    let dir = v(2.0, 1.0);

    let computed: Vec<(usize, f64, Vector)> = Interceptor::new(start, dir, 6).collect();

    let expected = vec![
        (1,   5.0/16.0, v(1.0, 1.75)),
        (2,  45.0/16.0, v(2.0, 2.25)),
        (3, 125.0/16.0, v(3.0, 2.75)),
        (4, 245.0/16.0, v(4.0, 3.25))
    ];

    assert_eq!(computed, expected);

}

#[test]
fn test_raycaster() { 

    let middle = v(2.5, 2.5);
    let grid = Grid { height: 5, width: 5};

    let hits: Vec<_> = Raycaster::new(middle ,  v(0.0, -1.0) , grid).take(2).collect();
    let expected = vec![
        Hit { x: 2, y: 1, direction: Direction::S, position: 0.5, distance: 0.25 },
        Hit { x: 2, y: 0, direction: Direction::S, position: 0.5, distance: 2.25 },
    ];

    assert_eq!(hits, expected);
    
    let hits: Vec<_> = Raycaster::new(v(0.5, 1.5) ,  v(2.0, 1.0) , grid).collect();
    let expected = vec![
        Hit { x: 1, y: 1, direction: Direction::W, position: 0.75, distance: 5.0/16.0 },
        Hit { x: 1, y: 2, direction: Direction::N, position: 0.5, distance: 5.0/4.0 },
        Hit { x: 2, y: 2, direction: Direction::W, position: 0.25, distance: 45.0/16.0 },
        Hit { x: 3, y: 2, direction: Direction::W, position: 0.75, distance: 125.0/16.0 },
        Hit { x: 3, y: 3, direction: Direction::N, position: 0.5, distance: 45.0/4.0 },
    ];

    assert_eq!(hits, expected);
    assert!(false);

}