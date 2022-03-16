use std::fmt;
use std::cmp::min;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

use std::io::{stdout, Write};
use crossterm::{
    terminal,
    ExecutableCommand, QueueableCommand,
    cursor, style::{self, Stylize}, self
};
use crossterm::event::{poll, read, Event};

use std::time::Duration;
use std::thread::sleep;


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

pub type Mask = Space<()>;

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
    pub fn get_width(&self) -> usize { self.w }
    pub fn get_height(&self) -> usize { self.h }
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
    pub fn copy_in_with_bounds(&mut self, mut x: usize, mut y: usize, other: Space<T>) -> (usize, usize){
        if x+other.w > self.w{
            x -= (x+other.w)-self.w
        }
        if y+other.h> self.h{
            y -= (y+other.h)-self.h
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
        (x, y)
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

pub type Desk = Space<(Colors, bool)>;

impl Desk {
    pub fn fall(&mut self) -> bool {
        // Yeah, disgusting code
        // But I'm writing this at 3 a.m. so it's acceptable
        let mut ret = false;
        for x in 0..self.w {
            if let Some((c, s)) = self.space[self.h-1][x]{
                self.space[self.h-1][x] = Some((c, false))
            }
        }
        // Fall shapes
        for y in (0..self.h-1).rev() {
            let mut can_fall = true;
            for x in 0..self.w {
                if let Some((_, s)) = self.space[y][x]{
                    if s {
                        if let Some((_, s2)) = self.space[y+1][x]{
                            if !s2{
                                can_fall = false;
                            }
                        }
                    }
                }
            }
            if can_fall {
                for x in 0..self.w {
                    if let Some((_, s)) = self.space[y][x] {
                        if s {
                            if let None = self.space[y+1][x] {
                                self.space[y+1][x] = self.space[y][x];
                                self.space[y][x] = None;
                                ret = true;
                            }
                        }
                    }
                }
            }else{
                for yy in y..self.h {
                    for x in 0..self.w {
                        if let Some((c, s)) = self.space[yy][x] {
                            self.space[yy][x] = Some((c, false));
                            if s && yy > y {
                                self.space[yy-1][x] = self.space[yy][x];
                                self.space[yy][x] = None;
                            }
                        }
                    }
                }
            }
        }
        if ret { return true }
        // Fall all other
        for y in (1..self.h).rev() {
            let mut void = true;
            for x in 0..self.w {
                if let Some(_) = self.space[y][x] {
                    void = false;
                    break;
                }
            }
            if void {
                for x in 0..self.w {
                    if let Some(_) = self.space[y-1][x] {
                        self.space[y][x] = self.space[y-1][x];
                        self.space[y-1][x] = None;
                        ret = true;
                    }
                }
            }
        }
        ret
    }
    pub fn remove(&mut self) -> bool {
        let mut ret = false;
        for y in 0..self.h{
            let mut rm = true;
            for x in 0..self.w{
                if let None = self.space[y][x]{
                    rm = false;
                    break;
                }
            }
            if rm {
                ret = true;
                for x in 0..self.w{
                    self.space[y][x] = None;
                }
            }
        }
        ret
    }
    pub fn step(&mut self) -> bool {
        if self.fall() { return true }
        if self.remove() { return true }
        false
    }
    pub fn get_content_heigth(&self) -> usize {
        for y in 0..self.h {
            for x in 0..self.w {
                if let Some(_) = self.space[y][x]{
                    return self.h-y
                }
            }
        }
        0
    }
}

fn get_i_shape(color: Colors) -> Desk {
    let color = Some((color, true));
    let mut shape = Desk::new(1,4);
    shape.set_no_err(0, 0, color);
    shape.set_no_err(0, 1, color);
    shape.set_no_err(0, 2, color);
    shape.set_no_err(0, 3, color);
    shape
}

fn get_o_shape(color: Colors) -> Desk {
    let color = Some((color, true));
    let mut shape = Desk::new(2,2);
    shape.set_no_err(0, 0, color);
    shape.set_no_err(0, 1, color);
    shape.set_no_err(1, 0, color);
    shape.set_no_err(1, 1, color);
    shape
}

fn get_l_shape(color: Colors) -> Desk {
    let color = Some((color, true));
    let mut shape = Desk::new(2,3);
    shape.set_no_err(0, 0, color);
    shape.set_no_err(0, 1, color);
    shape.set_no_err(0, 2, color);
    shape.set_no_err(1, 2, color);
    shape
}

fn get_j_shape(color: Colors) -> Desk {
    let color = Some((color, true));
    let mut shape = Desk::new(2,3);
    shape.set_no_err(1, 0, color);
    shape.set_no_err(1, 1, color);
    shape.set_no_err(1, 2, color);
    shape.set_no_err(0, 2, color);
    shape
}

fn get_s_shape(color: Colors) -> Desk {
    let color = Some((color, true));
    let mut shape = Desk::new(3,2);
    shape.set_no_err(0, 1, color);
    shape.set_no_err(1, 1, color);
    shape.set_no_err(1, 0, color);
    shape.set_no_err(2, 0, color);
    shape
}

fn get_z_shape(color: Colors) -> Desk {
    let color = Some((color, true));
    let mut shape = Desk::new(3,2);
    shape.set_no_err(2, 1, color);
    shape.set_no_err(1, 1, color);
    shape.set_no_err(1, 0, color);
    shape.set_no_err(0, 0, color);
    shape
}

fn get_t_shape(color: Colors) -> Desk {
    let color = Some((color, true));
    let mut shape = Desk::new(3,2);
    shape.set_no_err(2, 1, color);
    shape.set_no_err(1, 1, color);
    shape.set_no_err(1, 0, color);
    shape.set_no_err(0, 1, color);
    shape
}

fn get_random_shape<R: Rng + ?Sized>(rng: &mut R, color: Colors) -> Desk {
    match rng.gen_range(0..7) {
        0 => { get_i_shape(color) }
        1 => { get_o_shape(color) }
        2 => { get_l_shape(color) }
        3 => { get_j_shape(color) }
        4 => { get_s_shape(color) }
        5 => { get_z_shape(color) }
        _ => { get_t_shape(color) }
    }
}

fn render<O: Write + ?Sized>(out: &mut O, desk: &Desk, x: usize, y: usize) -> crossterm::Result<()>{
    for row in 0..desk.get_height() {
        out.queue(cursor::MoveTo(x  as u16 ,(y+row) as u16 ))?;
        for symbol in 0..desk.get_width() {
            if let Some((c, f)) = desk.get_no_err(symbol, row) {
                let color = match c {
                    Colors::Red => { style::Color::Red }
                    Colors::Green => { style::Color::DarkGreen }
                    Colors::Blue => { style::Color::Blue }
                    Colors::Yellow => { style::Color::Yellow }
                };
                if f {
                    out.queue(style::Print("██".with(color)))?;
                }else{
                    out.queue(style::Print("▒▒".with(color)))?;
                }
            }else{
                out.queue(style::Print("░░"))?;
            }
        }
    }
    out.flush()?;
    Ok(())
}

fn main() {
    let mut rng = StdRng::seed_from_u64(10);
    let color = Colors::get_random(&mut rng);
    let ax = rng.gen_range(0..4) ;
    let shape = get_random_shape(&mut rng, color).get_rotated(Rotation::Clockwise, ax);
    let mut desk = Desk::new(10,20);
    desk.copy_in_with_bounds(3,3,shape);
    let mut shape = Desk::new_default(10,2,Some((color, true)));
    desk.copy_in_with_bounds(0,10,shape);
    //
    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    terminal::enable_raw_mode().unwrap();
    let (x, y) = cursor::position().unwrap();
    for _ in 0..desk.get_height() {
        print!("\n");
    }
    let d = Duration::from_millis(60);
    //
    'outer: loop {
        while desk.step() {
            //println!("{:?}\n{:?}", desk.get_content_heigth(), desk);
            render(&mut stdout, &desk, x as usize, y as usize - desk.get_height()).unwrap();
            if poll(d).unwrap() {
                match read().unwrap() {
                    Event::Key(_) => {break 'outer;}
                    _ => {}
                }
            }
        }
        if desk.get_content_heigth() >= desk.get_height() {
            desk = Desk::new(10,20);
        }
        let color = Colors::get_random(&mut rng);
        let ax = rng.gen_range(0..4) ;
        let shape = get_random_shape(&mut rng, color).get_rotated(Rotation::Clockwise, ax);
        desk.copy_in_with_bounds(rng.gen_range(0..10),rng.gen_range(0..4),shape);
    }
    //
    stdout.queue(cursor::MoveTo(x  as u16 ,y as u16 )).unwrap();
    terminal::disable_raw_mode().unwrap();
    stdout.execute(cursor::Show).unwrap();
}
