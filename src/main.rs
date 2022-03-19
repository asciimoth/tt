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
use crossterm::event::{poll, read, Event, KeyCode};

use std::time::Duration;
use clap::Parser;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Rotation{
    Clockwise,
    Counterclockwise,
}

pub struct Space<T>{
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
        let mut result = self.clone();
        for _ in 0..count{
            let mut rotated = Self::new(result.h,result.w);
            for x in 0..result.w {
                for y in 0..result.h {
                    if let Rotation::Clockwise = rotation {
                        rotated.space[x][result.h-1-y] = result.space[y][x].clone();
                    }else{
                        rotated.space[result.w-1-x][y] = result.space[y][x].clone();
                    }
                }
            }
            result = rotated;
        }
        result
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
            if let Some((c, _)) = self.space[self.h-1][x]{
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

fn render<O: Write + ?Sized>(out: &mut O, desk: &Desk, x: usize, y: usize, extended: bool) -> crossterm::Result<()>{
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
                    if extended {
                        out.queue(style::Print("▒▒".with(color)))?;
                    }else{
                        out.queue(style::Print("██".with(color)))?;
                    }
                }
            }else{
                if extended {
                    out.queue(style::Print("░░"))?;
                }else{
                    out.queue(style::Print("  "))?;
                }
            }
        }
    }
    Ok(())
}

struct Field{
    x: u16,
    y: u16,
    desk: Desk,
}

fn render_fields<O: Write + ?Sized>(out: &mut O, fields: &Vec<Field>, extended: bool) -> crossterm::Result<()>{
    let mut ax = 0;
    for field in fields {
        render(out, &field.desk.get_rotated(Rotation::Clockwise, ax), field.x as usize, field.y as usize, extended)?;
        ax += 1;
    }
    Ok(())
}

fn run(mut fields_count: u8, mut width: u16, height: u16, seed: u64, extended_render: bool, delay: u64)  -> crossterm::Result<usize> {
    if fields_count > 4 { fields_count = 4; }
    if fields_count < 1 { return Ok(0) }
    if width > height { width = height }
    let real_width = width;
    let real_height = height + width;
    let mut fields: Vec::<Field> = Vec::new();
    let total_h = if fields_count > 2 { real_height + height } else { real_height };
    let mut rng = StdRng::seed_from_u64(seed);
    let duration = Duration::from_millis(delay);
    let mut score: usize = 0;
    let mut max_score: usize = 0;
    //
    let mut stdout = stdout();
    stdout.execute(cursor::Hide)?;
    terminal::enable_raw_mode()?;
    let (_, cy) = cursor::position()?;
    let sy = if total_h > cy { 0 } else { cy - total_h };
    for _ in 0..total_h { print!("\n"); }
    //
    for i in 0..fields_count {
        let x: u16 = if i < 3 && fields_count > 3 { height*2 } else { 0 };
        let y: u16 = sy + match i {
            0 => { if fields_count > 2 { height } else { 0 } }
            1 => { if fields_count > 2 { height } else { 0 } }
            2 => { 0 }
            _ => { height }
        };
        fields.push(Field{x, y, desk: Desk::new(real_width as usize, real_height as usize)});
    }
    let score_x = fields[0].x+width*2+1;
    let score_y = fields[0].y+if fields_count > 1 { width+1 } else { 0 };
    //
    render_fields(&mut stdout, &fields, extended_render)?;
    stdout.queue(cursor::MoveTo(score_x  as u16 , score_y as u16 ))?;
    stdout.queue(style::Print(format!("score: {:?} max: {:?}", score, max_score)))?;
    stdout.flush()?;
    'outer: loop {
        let color = Colors::get_random(&mut rng);
        let mut shape = get_random_shape(&mut rng, color);
        let (mut shape_x, mut shape_y) = (rng.gen_range(0..width as usize),rng.gen_range(0..width as usize));
        let mut change = true;
        let mut select_desk = Desk::new(width as usize, width as usize);
        loop{
            if change {
                select_desk = Desk::new(width as usize, width as usize);
                let a = select_desk.copy_in_with_bounds(shape_x, shape_y, shape.clone());
                shape_x = a.0;
                shape_y = a.1;
                render(&mut stdout, &select_desk, fields[0].x as usize, fields[0].y as usize, extended_render)?;
                stdout.flush()?;
                change = false;
            }
            if poll(duration)? {
                match read()? {
                    Event::Key(event) => {
                        match event.code {
                            KeyCode::Esc => { break 'outer; }
                            KeyCode::Char(' ') => { break }
                            KeyCode::Char('a') => {
                                if shape_x > 0 {
                                    shape_x -= 1;
                                    change = true;
                                }
                            }
                            KeyCode::Char('d') => {
                                shape_x += 1;
                                change = true;
                            }
                            KeyCode::Char('w') => {
                                if shape_y > 0 {
                                    shape_y -= 1;
                                    change = true;
                                }
                            }
                            KeyCode::Char('s') => {
                                shape_y += 1;
                                change = true;
                            }
                            KeyCode::Char('e') => {
                                shape = shape.get_rotated(Rotation::Clockwise, 1);
                                change = true;
                            }
                            KeyCode::Char('q') => {
                                shape = shape.get_rotated(Rotation::Counterclockwise, 1);
                                change = true;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        //
        let mut ax = 0;
        let mut end = false;
        for field in &mut fields {
            let lax = match ax {
                1 => { 3 }
                3 => { 1 }
                _ => { ax }
            };
            field.desk.copy_in_with_bounds(0,0, select_desk.get_rotated(Rotation::Counterclockwise, lax));
            while field.desk.step() {
                render(&mut stdout, &field.desk.get_rotated(Rotation::Clockwise, lax), field.x as usize, field.y as usize, extended_render)?;
                stdout.flush()?;
                if poll(duration)? {
                    match read().unwrap() {
                        Event::Key(event) => {
                            match event.code {
                                KeyCode::Esc => { break 'outer; }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            if field.desk.get_content_heigth() >= height as usize {
                end = true;
            }
            ax += 1;
        }
        if end {
            score = 0;
            for i in 0..fields.len() {
                fields[i].desk = Desk::new(fields[i].desk.w, fields[i].desk.h);
            }
            render_fields(&mut stdout, &fields, extended_render)?;
        }else{
            score += 1;
            if score > max_score {
                max_score = score;
            }
        }
        stdout.queue(cursor::MoveTo(score_x  as u16 , score_y as u16 ))?;
        stdout.queue(style::Print("                                                 "))?;
        stdout.queue(cursor::MoveTo(score_x  as u16 , score_y as u16 ))?;
        stdout.queue(style::Print(format!("score: {:?} max: {:?}", score, max_score)))?;
        stdout.flush()?;
    }
    //
    stdout.queue(cursor::MoveTo(0 , cy))?;
    terminal::disable_raw_mode()?;
    stdout.execute(cursor::Show)?;
    Ok(score)
}

#[derive(Parser, Debug)]
#[clap(author, version, about = include_str!("about.txt"), long_about = None)]
struct Args {
    /// Count of feelds
    #[clap(short, long, default_value_t = 4)]
    fields: u8,

    /// Field width
    #[clap(short, long, default_value_t = 10)]
    width: u16,

    /// Field height
    #[clap(short, long, default_value_t = 10)]
    height: u16,

    /// PRNG seed
    #[clap(short, long, default_value_t = 12345)]
    seed: u64,

    /// Render delay
    #[clap(short, long, default_value_t = 10)]
    delay: u64,
}

fn main() {
    let args = Args::parse();
    if let Err(err) = run(
        args.fields,
        args.width,
        args.height,
        args.seed,
        true,
        args.delay) {
        print!("{:?}", err);
    }
}
