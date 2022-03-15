use std::fmt;
use std::cmp::min;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Rotation{
    Clockwise, // По часовой
    Counterclockwise, // Против часовой
}

struct Space<T>{
    space: Vec<Vec<Option<T>>>,
    w: usize,
    h: usize,
}

type Mask = Space<()>;

impl<T> Space<T>{
    pub fn new(w: usize, h: usize) -> Self{
        let mut space = Vec::new();
        for _ in 0..h {
            let mut row = Vec::new();
            for _ in 0..w{
                row.push(None);
            }
            space.push(row);
        }
        Self{space, w, h}
    }
    pub fn get_width(self) -> usize { self.w }
    pub fn get_height(self) -> usize { self.h }
    pub fn set(&mut self, x: usize, y: usize, value: Option<T>) -> Result<(), &'static str> {
        if x >= self.w || y >= self.h {
            Err("Out of space")
        }else{
            self.space[y][x] = value;
            Ok(())
        }
    }
    pub fn set_no_err(&mut self, x: usize, y: usize, value: Option<T>) {
        self.space[y][x] = value;
    }
    pub fn copy_in(&mut self,  x: usize, y: usize, other: Space<T>) -> Result<(), &'static str> {
        if x+other.w-1 > self.w || y+other.h-1 > self.h {
            return Err("Out of space")
        }
        let mut yc = 0;
        for raw in other.space {
            let mut xc = 0;
            for v in raw {
                self.space[yc+y][xc+x] = v;
                xc += 1;
            }
            yc += 1;
        }
        Ok(())
    }
    pub fn copy_in_with_mask(&mut self,  x: usize, y: usize, other: Space<T>, mask: Mask) -> Result<(), &'static str> {
        if other.w != mask.w {
            return Err("Mask width incorrect")
        }
        if other.h != mask.h {
            return Err("Mask height incorrect")
        }
        if x+other.w-1 > self.w || y+other.h-1 > self.h {
            return Err("Out of space")
        }
        let mut yc = 0;
        for raw in other.space {
            let mut xc = 0;
            for v in raw {
                if let Some(_) = mask.space[yc][xc] {
                    self.space[yc+y][xc+x] = v;
                }
                xc += 1;
            }
            yc += 1;
        }
        Ok(())
    }
}

impl<T: PartialEq> PartialEq for Space<T>{
    fn eq(&self, other: &Self) -> bool {
        if self.w != other.w || self.h != other.h { return false }
        for x in 0..self.w {
            for y in 0..self.h {
                if self.space[y][x] != other.space[y][x] { return false }
            }
        }
        true
    }
}

impl<T: Copy> Space<T>{
    pub fn get(&self, x: usize, y: usize) -> Result<Option<T>, &'static str> {
        if x >= self.w || y >= self.h {
            Err("Out of space")
        }else{
            Ok(self.space[y][x])
        }
    }
    pub fn get_no_err(&self, x: usize, y: usize) -> Option<T> {
        self.space[y][x]
    }
}

impl<T: std::clone::Clone> Space<T>{
    pub fn new_default(w: usize, h: usize, d: Option<T>) -> Self{
        let mut space = Vec::<Vec::<Option<T>>>::new();
        for _ in 0..h {
            space.push(vec![d.clone(); w]);
        }
        Self{space, w, h}
    }
    pub fn get_clone(&self, x: usize, y: usize) -> Result<Option<T>, &'static str> {
        if x >= self.w || y >= self.h {
            Err("Out of space")
        }else{
            Ok(self.space[y][x].clone())
        }
    }
    pub fn get_clone_no_err(&self, x: usize, y: usize) -> Option<T> {
        self.space[y][x].clone()
    }
    pub fn get_rotated(&self, rotation: Rotation, count: usize) -> Self{
        if count % 4 == 0 {return self.clone()}
        let mut rotated = Self::new(self.h,self.w);
        for x in 0..self.w {
            for y in 0..self.h {
                if let Rotation::Clockwise = rotation {
                    rotated.space[x][self.h-1-y] = self.space[y][x].clone();
                }else{
                    rotated.space[self.w-1-x][y] = self.space[y][x].clone();
                }
            }
        }
        rotated
    }
}

impl<T: std::clone::Clone> Clone for Space<T>{
    fn clone(&self) -> Self {
        let mut result = Space::<T>::new(self.w, self.h);
        for x in 0..self.w {
            for y in 0..self.h {
                result.set_no_err(x,y, self.get_clone_no_err(x,y))
            }
        }
        result
    }
}

impl<T> fmt::Debug for Space<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " ")?;
        for i in 0..min(10, self.w) {
            write!(f, "{} ", i)?;
        }
        write!(f, "\n")?;
        for h in 0..self.h {
            if h < 10 {
                write!(f, "{}", h)?;
            }else{
                write!(f, " ")?;
            }
            for point in &self.space[h] {
                if let None = point {
                    write!(f, "░░")?;
                }else{
                    write!(f, "██")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Mask{
    pub fn invert(&mut self){
        for x in 0..self.w {
            for y in 0..self.h {
                if let None = self.space[y][x]{
                    self.space[y][x] = Some(());
                }else{
                    self.space[y][x] = None;
                }
            }
        }
    }
    pub fn get_invert(self) -> Self{
        let mut clone = self.clone();
        clone.invert();
        clone
    }
    pub fn from_space<T>(space: &Space<T>) -> Self{
        let mut ret = Mask::new(space.w, space.h);
        for x in 0..space.w {
            for y in 0..space.h {
                if let Some(_) = space.space[y][x] {
                    ret.space[y][x] = Some(())
                }
            }
        }
        ret
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Colors{
    Red,
    Green,
    Blue,
    Yellow,
}

impl Colors{
    pub fn get_random<R: Rng + ?Sized>(rng: &mut R) -> Colors{
        let rand = rng.gen_range(0..4);
        match rand {
            0 => Colors::Red,
            1 => Colors::Green,
            2 => Colors::Blue,
            _ => Colors::Yellow,
        }
    }
}

impl fmt::Debug for Colors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self{
            Colors::Red => { write!(f, "R") }
            Colors::Green => { write!(f, "G") }
            Colors::Blue => { write!(f, "B") }
            Colors::Yellow => { write!(f, "Y") }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ShapeType{
    I,
    O,
    L,
    J,
    S,
    Z,
    T,
}

impl ShapeType{
    pub fn get_random<R: Rng + ?Sized>(rng: &mut R) -> ShapeType{
        let rand = rng.gen_range(0..7);
        match rand{
            0 => ShapeType::I,
            1 => ShapeType::O,
            2 => ShapeType::L,
            3 => ShapeType::J,
            4 => ShapeType::S,
            5 => ShapeType::Z,
            _ => ShapeType::T,
        }
    }
}

pub struct Shape{
    shape: Space<Colors>,
    mask: Mask,
}

impl Shape{
    pub fn new(tp: ShapeType, color: Colors) -> Self{
        let mut shape = Space::<Colors>::new(5,5);
        match tp {
            ShapeType::I => {
                shape.set_no_err(2,1, Some(color));
                shape.set_no_err(2,2, Some(color));
                shape.set_no_err(2,3, Some(color));
                shape.set_no_err(2,4, Some(color));
            }
            ShapeType::O => {
                shape.set_no_err(2,2, Some(color));
                shape.set_no_err(2,3, Some(color));
                shape.set_no_err(3,2, Some(color));
                shape.set_no_err(3,3, Some(color));
            }
            ShapeType::L => {
                shape.set_no_err(2,1, Some(color));
                shape.set_no_err(2,2, Some(color));
                shape.set_no_err(2,3, Some(color));
                shape.set_no_err(3,3, Some(color));
            }
            ShapeType::J => {
                shape.set_no_err(2,1, Some(color));
                shape.set_no_err(2,2, Some(color));
                shape.set_no_err(2,3, Some(color));
                shape.set_no_err(1,3, Some(color));
            }
            ShapeType::S => {
                shape.set_no_err(2,1, Some(color));
                shape.set_no_err(3,1, Some(color));
                shape.set_no_err(1,2, Some(color));
                shape.set_no_err(2,2, Some(color));
            }
            ShapeType::Z => {
                shape.set_no_err(1,1, Some(color));
                shape.set_no_err(2,1, Some(color));
                shape.set_no_err(2,2, Some(color));
                shape.set_no_err(3,2, Some(color));
            }
            ShapeType::T => {
                shape.set_no_err(2,2, Some(color));
                shape.set_no_err(1,3, Some(color));
                shape.set_no_err(2,3, Some(color));
                shape.set_no_err(3,3, Some(color));
            }
        }
        let mask = Mask::from_space(&shape);
        Shape{shape, mask}
    }
}

impl fmt::Debug for Shape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut minx: usize = self.mask.w;
        let mut maxx: usize = 0;
        let mut miny: usize = self.mask.h;
        let mut maxy: usize = 0;
        for x in 0..self.mask.w {
            for y in 0..self.mask.h {
                if let Some(()) = self.mask.get_no_err(x,y){
                    if x > maxx { maxx = x; }
                    if x < minx { minx = x; }
                    if y > maxy { maxy = y; }
                    if y < miny { miny = y; }
                }
            }
        }
        for y in miny..maxy+1 {
            for x in minx..maxx+1 {
                if let Some(color) = self.shape.space[y][x] {
                    write!(f, "{:?}", color)?;
                }else{
                    write!(f, " ")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

fn main() {
    let mut rng = StdRng::seed_from_u64(1);
    for _ in 0..10 {
        let shape_type = ShapeType::get_random(&mut rng);
        let color = Colors::get_random(&mut rng);
        let shape = Shape::new(shape_type, color);
        println!("{:?}", shape);
    }
}
