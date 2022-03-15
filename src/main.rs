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
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

fn main() {
    let mut seed_max: u64 = 0;
    let mut ln_max: usize = 0;
    for seed in 0..u64::MAX {
        let mut ln: usize = 0;
        let mut rng = StdRng::seed_from_u64(seed);
        loop{
            let shape = ShapeType::get_random(&mut rng);
            let color = Colors::get_random(&mut rng);
            if let ShapeType::O = shape{
                ln += 1;
            }else{
                break
            }
        }
        if ln > ln_max {
            ln_max = ln;
            seed_max = seed;
        }
        println!("Iteration: {:?}", seed);
        println!("Max: {:?} Seed: {:?} \n", ln_max, seed_max);
    }
    /*let mut rng = StdRng::seed_from_u64(29563);
    for _ in 0..10 {
        println!("{:?}", ShapeType::get_random(&mut rng));
    }*/
    /*let mut space = Space::<()>::new_default(20, 11, Some(()));
    println!("{:?}", space);
    println!("{:?}", space.get_no_err(0,0));
    for x in 0..30 {
        if let Err(err) = space.set(x,0, None){
            println!("Err '{}' when set {}, 0", err, x);
            break
        }
    }
    println!("{:?}", space);
    let mut figure = Space::<()>::new_default(3, 4, None);
    figure.set_no_err(1,1, Some(()));
    figure.set_no_err(1,2, Some(()));
    let mut mask = Space::<()>::new_default(3, 4, Some(()));
    mask.set_no_err(0,3, None);
    mask.set_no_err(1,3, None);
    mask.set_no_err(2,3, None);
    println!("{:?}{:?}", figure, mask);
    println!("{:?}", figure == figure.clone());
    println!("{:?}", space.copy_in(1,2, figure.clone()));
    println!("{:?}", space.copy_in_with_mask(5,2, figure.clone(), mask.clone()));
    println!("{:?}", space.copy_in_with_mask(9,2, figure.clone(), mask.get_invert()));
    println!("{:?}", space);
    println!("{:?}", space.get_rotated(Rotation::Clockwise, 1));
    println!("{:?}", space.get_rotated(Rotation::Counterclockwise, 1));*/
}
