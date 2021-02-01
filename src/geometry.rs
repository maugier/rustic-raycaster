use std::{iter::{Peekable, Rev}, ops::Range};
use either::Either;

use crate::loader::Direction;

#[derive(Clone,Copy,PartialEq,Debug)]
pub struct Vector { x: f64, y: f64 }

impl Vector {
    fn flip(&self) -> Self {
        Vector { x: self.y, y: self.x }
    }
    fn squared_distance(&self, rhs: &Self) -> f64 {
        let dx = rhs.x - self.x;
        let dy = rhs.y - self.y;
        dx*dx + dy*dy
    }
}

impl From<(f64,f64)> for Vector {
    fn from((x,y):(f64,f64)) -> Self { Vector {x,y} }
}

#[inline(always)]
fn v(x: f64, y: f64) -> Vector { Vector {x,y} }

#[derive(Clone,Copy,PartialEq,Eq)]
pub struct Grid { height: usize, width: usize }

type Position = Vector;

type DynRange = Either<Range<usize>,Rev<Range<usize>>>;

#[derive(Debug)]
pub struct Hit {
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
    pub position: f64,
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
            Either::Left(ceil .. size-1)
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
    type Item = (usize,Position);

    fn next(&mut self) -> Option<(usize,Position)> {
        self.range
            .next()
            .and_then(|xi| {
                let x = xi as f64;
                let y = self.p.y + (x - self.p.x) * self.slope;
                Some((xi, Vector{x,y}))               
            })
    }
}

// An iterator of all the 
pub struct Raycaster {
    p: Position,
    d: Vector,
    xint: Peekable<Interceptor>,
    yint: Peekable<Interceptor>,
}

impl Raycaster {
    pub fn new(p: Position, d: Vector, g: Grid)  -> Self {
        Raycaster {
            p,d,
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

        let xhit = match (px,py) {
            (None,None) => return None,
            (Some(px), Some(py)) =>
                { px.1.squared_distance(&self.p) <= py.1.squared_distance(&self.p.flip()) },
            (Some(_),None) => true,
            (None,Some(_)) => false,
        };

        let (x,y,position,direction);

        
        if xhit {
            let (xi, p) = self.xint.next().unwrap();
            let fy = p.y.floor();
            
            y = fy as usize;
            position = p.y - fy;
            if self.d.x < 0.0 {
                x = xi - 1; 
                direction = Direction::E 
            } else { 
                x = xi;
                direction = Direction::W
            }
            
        } else {
            let (yi, p) = self.yint.next().unwrap();
            let p = p.flip();
            let fx = p.x.floor();
            
            x = fx as usize;
            position = p.x - fx;
            if self.d.y < 0.0 { 
                y = yi - 1;
                direction = Direction::S
            } else {
                y = yi; 
                direction = Direction::N
            };
        
        }
 
        Some(Hit {x,y,position,direction})

    }
}

#[test]
fn test_interceptor() {
    
    let start = v(0.5, 1.5);
    let dir = v(2.0, 1.0);

    let expected = vec![
        (1,v(1.0, 1.75)),
        (2,v(2.0, 2.25)),
        (3,v(3.0, 2.75)),
        (4,v(4.0, 3.25))
    ];
    let computed: Vec<(usize, Vector)> = Interceptor::new(start, dir, 6).collect();

    assert_eq!(expected, computed);

}

#[test]
fn test_raycaster() { 
    let middle = v(2.5, 2.5);
    let grid = Grid { height: 5, width: 5};

    let r = Raycaster::new(middle ,  v(0.0, -1.0) , grid);
    eprintln!("{:?}", r.collect::<Vec<_>>());
    
    let r = Raycaster::new(v(0.1, 2.1),  v(1.0, 0.3) , grid);
    eprintln!("{:?}", r.collect::<Vec<_>>());
}