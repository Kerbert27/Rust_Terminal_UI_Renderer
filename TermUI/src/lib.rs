#![allow(non_snake_case)]
use crossterm::style::{self, Color, StyledContent, Stylize};
use crossterm::terminal::{self};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader, Rgb};
use std::collections::VecDeque;
use std::time::{self, Duration, Instant};

const BAR_H: &str = "─";
const BAR_V: &str = "┃";
const BAR_TL: &str = "┎";
const BAR_TR: &str = "┒";
const BAR_BL: &str = "┖";
const BAR_BR: &str = "┛";
const DOT_H: &str = "┅";
const DOT_V: &str = "┇";

const GRID_INTERVAL: u16 = 10;

mod matrix;
pub use matrix::*;

mod renderer;
pub use renderer::*;

mod graph;
pub use graph::*;

use track_macros::*;



pub enum Element {
    Box(Rectangle),
    TextBox(TextBox),
    Sprite(Sprite),
    ScreenBuffer(ScreenBuffer),
    Graph(Graph),
    Timeline(Timeline)
}



#[derive(Clone, Copy)]
pub enum RectFill {
    FilledAndBoxed(StyledContent<char>),
    FilledAndDotted(StyledContent<char>),
    Filled(StyledContent<char>),
    EmptyAndBoxed,
    EmptyAndDotted,
    Empty
}

impl RectFill {
    pub fn isEmpty(&self) -> bool {
        match self {
            RectFill::Empty | RectFill::EmptyAndBoxed | RectFill::EmptyAndDotted => return true,
            _ => return false
        }
    }

    pub fn isBoxed(&self) -> bool {
        match self {
            RectFill::EmptyAndBoxed | RectFill::FilledAndBoxed(_) => return true,
            _ => return false
        } 
    }

    pub fn isDotted(&self) -> bool {
        match self {
            RectFill::EmptyAndDotted | RectFill::FilledAndDotted(_) => return true,
            _ => return false
        } 
    }

    pub fn isBounded(&self) -> bool {
        self.isBoxed() | self.isDotted()
    }

    pub fn get_lines(&self) -> (String, String) {
        match self {
            _b if self.isDotted() => (DOT_H.to_string(), DOT_V.to_string()),
            _b if self.isBoxed() => (BAR_H.to_string(), BAR_V.to_string()),
            _b if self.isEmpty() => (String::new(), String::new()),
            _ => (String::from(" "), String::from(""))
        }
    }

    pub fn get_fill(&self) -> String {
        match self {
            Self::Filled(c) | Self::FilledAndDotted(c) | Self::FilledAndBoxed(c)=> c.to_string(),
            _ => String::from(" ")
        }
    }
}


#[derive(Clone, Copy)]
pub struct Rectangle {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub fill: RectFill
}

impl Rectangle {
    pub fn draw(&self, stdout: &mut StdoutBuffer) {
        let (line_h, line_v) = self.fill.get_lines();
        
        if self.fill.isBounded() {
            let s = format!("{}{}{}",BAR_TL, line_h.repeat(self.width as usize), BAR_TR);
            draw_at(stdout, self.x.saturating_sub(1) as usize, self.y.saturating_sub(1) as usize, s);
            let s = format!("{}{}{}", BAR_BL, line_h.repeat(self.width as usize), BAR_BR);
            draw_at(stdout, self.x.saturating_sub(1) as usize, self.y as usize + self.height as usize + 1, s);
        }

        let spacing: String = self.fill.get_fill().repeat(self.width as usize);
        for i in 0..self.height {
            stdout.write(self.x.saturating_sub(1) as usize, (self.y + i + 1)as usize, line_v.clone());
            stdout.write((self.x + self.width) as usize, (self.y + i + 1) as usize, line_v.clone());
        }
    }

    pub fn right(&self) -> u16 {
        self.x + self.width
    }

    pub fn bottom(&self) -> u16 {
        self.y + self.height
    }

    pub fn top_right(&self) -> [u16;2] {
        [self.x + self.width, self.y]
    }

    pub fn top_left(&self) -> [u16;2] {
        [self.x, self.y]
    }

    pub fn bottom_right(&self) -> [u16;2] {
        [self.x + self.width, self.y + self.height]
    }

    pub fn bottom_left(&self) -> [u16;2] {
        [self.x + self.width, self.y + self.height]
    }

    pub fn span_x(&self) -> [u16; 2]{
        [self.x, self.x + self.width]
    }

    pub fn span_y(&self) -> [u16; 2]{
        [self.y, self.y + self.height]
    }



}

pub enum OverflowType {
    Wrap,
    Truncate,
    None
}
pub struct TextBox {
    pub rect: Rectangle,
    pub text: String,
    pub animation_time: Option<Duration>,
    pub overflow: OverflowType,
    pub showBorder: bool,
    reset: bool,
    start_time: Option<time::Instant>

}

impl TextBox {
    pub fn new(rect: Rectangle, text: String) -> TextBox{
        TextBox{rect, text, animation_time: None, overflow: OverflowType::Wrap, showBorder: true, reset: true, start_time: None}
    }

    pub fn new_animation(rect: Rectangle, text: String, t: Duration) -> TextBox {
        TextBox{rect, text, animation_time: Some(t), overflow: OverflowType::Wrap, showBorder: true, reset: true, start_time: None}
    }

    pub fn reset(&mut self) {
        self.reset = true;
    }
    
    

    fn draw_x(&mut self, stdout: &mut StdoutBuffer, chars: String) {
        if self.showBorder {self.rect.draw(stdout);}
        let mut pos = (self.rect.x + self.showBorder as u16, self.rect.y + self.showBorder as u16);
        //stdout.queue(MoveTo(self.rect.x + self.showBorder as u16, self.rect.y + self.showBorder as u16)).unwrap();
        let mut i: i32 = 0;
        for c in chars.chars() {
            pos.0 += 1;
            if i as u16>= self.rect.width - (self.showBorder as u16 * 2) {
                pos.1 += 1;
                pos.0 = self.rect.x + self.showBorder as u16;  
                //stdout.queue(cursor::MoveDown(1)).unwrap();
                //stdout.queue(cursor::MoveToColumn(self.rect.x + self.showBorder as u16)).unwrap();
                i = 0;
            }
            stdout.write(pos.0 as usize, pos.1 as usize, c.to_string());
            i += 1;
        }
    }


    fn animate(&mut self, stdout: &mut StdoutBuffer) {
        if self.reset {
            self.start_time = Some(time::Instant::now());
            self.reset = false;
        }

        if self.animation_time.unwrap().as_millis() > self.start_time.unwrap().elapsed().as_millis() {
            let i: usize  = ((self.start_time.unwrap().elapsed().as_millis() as f32 / self.animation_time.unwrap().as_millis() as f32) * self.text.len() as f32) as usize;
            self.draw_x(stdout, self.text[..i].to_string());
        } else {
            self.draw_x(stdout, self.text.to_string());
        }
    }

    pub fn draw(&mut self, stdout: &mut StdoutBuffer) {
        if self.animation_time.is_some() {
            self.animate(stdout);
        }
        else {
            self.draw_x(stdout, self.text.to_string());
        }
    }
}

#[derive(Copy, Clone)]
pub enum PixelMark {
    Filled(Rgb<u8>),
    Empty(Rgb<u8>),
    X(Rgb<u8>)
}

impl PixelMark {
    pub const FILL_COLOR: PixelMark = PixelMark::Filled(Rgb{0: [0,1,1]});
    pub const EMPTY_COLOR_A: Rgb<u8> = Rgb{0: [0,1,1]};
    pub const EMPTY_COLOR_B: Rgb<u8> = Rgb{0: [0,1,1]};
    pub const X_COLOR: Rgb<u8> = Rgb{0: [0,1,1]};
}

pub struct Sprite {
    pub rect: Rectangle,
    pub pixel_data: DynamicImage,
    pub grid: Vec<Vec<PixelMark>>
}

impl Sprite {
    

    pub fn new(filename: &str, x: u16, y: u16, width: u32, height: u32) -> Sprite {
        let mut raw_sprite: DynamicImage = ImageReader::open(filename).unwrap().decode().unwrap();
        raw_sprite = raw_sprite.resize(width, height, FilterType::Nearest);
        //raw_sprite = DynamicImage::from(canny(&raw_sprite.into_luma8(), 60.0, 100.0));

        let width: u32 = raw_sprite.width();
        let height: u32 = raw_sprite.height();

        

        let rect: Rectangle = Rectangle{x, y, width: width as u16, height: height as u16, fill: RectFill::Empty};
        let mut grid: Vec<Vec<PixelMark>> = Vec::new();
        grid.resize_with(height as usize, || 
           {vec![PixelMark::Empty(Rgb{0: [0, 0, 0]})].repeat(width as usize)}
        );
        Sprite{rect, pixel_data: raw_sprite, grid}
    }

    pub fn draw_empty(&mut self, stdout: &mut StdoutBuffer) {
        for (x, col) in self.grid.iter().enumerate() {
            let mut row_str: String = String::new();
            //stdout.queue(terminal::SetTitle(col.len()));
            let pos = (self.rect.x as u16, (self.rect.y + x as u16) as u16);
            //stdout.queue(cursor::MoveTo(self.rect.x as u16, (self.rect.y + x as u16) as u16)).unwrap();
            for (y, pixel) in col.iter().enumerate() {
                match *pixel {
                    PixelMark::Filled(color) => {
                        let [r, g, b] = color.0;
                        row_str.push_str("██".with(Color::Rgb{r, g, b}).to_string().as_str());
                    }
                    PixelMark::Empty(color) => {
                        let c = if (y + x) % 2 == 0 {Color::DarkGrey} else {Color::Rgb { r: 80, g: 80, b: 80 }};
                        row_str.push_str("██".with(c).to_string().as_str());
                    }
                    PixelMark::X(color) => {
                        row_str.push_str("><".with(style::Color::Red).to_string().as_str());
                    }
                }
                
            }
            stdout.write(pos.0 as usize, pos.1 as usize, row_str);
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize) {
        if let Some(p) = self.grid.get_mut(y).unwrap().get_mut(x/2) {
            *p = match *p {
                PixelMark::Filled(c) => PixelMark::Empty(PixelMark::EMPTY_COLOR_A),
                PixelMark::Empty(c) => PixelMark::FILL_COLOR,
                PixelMark::X(c) => PixelMark::Empty(PixelMark::EMPTY_COLOR_A)
            }
        }
    }

    pub fn mark_pixel(&mut self, x: usize, y: usize) {
        if let Some(p) = self.grid.get_mut(y).unwrap().get_mut(x/2) {
            *p = match *p {
                PixelMark::X(c) => PixelMark::Empty(PixelMark::EMPTY_COLOR_A),
                _ => PixelMark::X(PixelMark::X_COLOR)
            }
        }
    }

    pub fn draw(&mut self, stdout: &mut StdoutBuffer) {
        let pixels: Vec<(u32, u32, image::Rgba<u8>)> = self.pixel_data.pixels().collect();
        for y in 0..self.rect.height {
            let mut row: String = String::new();
            let pos = (self.rect.x as u16, (self.rect.y + y) as u16);
            //stdout.queue(cursor::MoveTo(self.rect.x as u16, (self.rect.y + y) as u16)).unwrap();
            for x in 0..self.rect.width * 2 {
                let index: usize = (y * self.rect.width + (x/2)) as usize;
                
                let [r,g,b, _] = pixels[index].2.0;
                let brightness: f32 = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) / 255.0;
                let shade: &str = match brightness {
                    n if n < 0.7 => "█",
                    // n if n > 0.5 => "█",
                    // n if n > 0.3 => "▓",
                    // n if n > 0.1 => "▒",
                    // n if n > 0.0 => "░",
                    _ => " "
                };
                row.push_str(shade.with(style::Color::Rgb { r, g, b }).to_string().as_str());
            }
            stdout.write(pos.0 as usize, pos.1 as usize, row);
            //stdout.queue(Clear)).unwrap();
        }
    }
}




pub struct Timer {
    pub size: usize,
    times: VecDeque<f32>,
    instant: Instant
}

impl Timer {
    pub fn new(size: usize) -> Timer {
        Timer {size, times: VecDeque::new(), instant: Instant::now()}
    }
    pub fn start(&mut self) {
        self.instant = Instant::now();
    }

    pub fn end(&mut self) {
        let drawtime = self.instant.elapsed();
        if self.times.len() == self.times.capacity() {
            self.times.pop_front();
        }
        self.times.push_back(drawtime.as_secs_f32());
        //self.times.iter().sum::<u128>() as f32 / self.times.len() as f32
    }

    pub fn mean(&mut self) -> f32 {
        let m = self.times.iter().sum::<f32>() / self.times.len() as f32;
        if m.is_nan() {return 0.0};
        m
    }
    
    pub fn total(&mut self) -> f32 {
        self.times.iter().sum::<f32>()
    }

    pub fn inc(&mut self) {
        let drawtime = self.instant.elapsed();
        if self.times.is_empty() {
            self.times.push_back(drawtime.as_micros() as f32);
        }
        else {
            *self.times.back_mut().unwrap() = drawtime.as_micros() as f32;
        }
    }
}

///Draw Bezier curve from start to end in screen coordinates with control point.
pub fn draw_curve(stdout: &mut StdoutBuffer, start: [f32; 2], end: [f32; 2], control: [f32; 2]) {
    for t in 0..=10 {
        let q0 = Matrix::lerp(start, control, t as f32 / 10.0);
        let q1 = Matrix::lerp(control, end, t as f32 / 10.0);
        let b = Matrix::lerp(q0, q1, t as f32 / 10.0);

        let pos = (b[0] as usize, b[1] as usize);
        stdout.write(pos.0, pos.1, "█".to_string());
    }
}

pub fn draw_line(stdout: &mut StdoutBuffer, start: [f32; 2], end: [f32; 2], glyph: String) {
    let dist = dist::<2>(&start[..], &end[..]);

    for t in 0..=dist as usize {
        let xy = Matrix::lerp(start, end, t as f32 / dist);
        let x = xy[0];
        let y = xy[1];
        draw_at(stdout, x as usize + 1, y as usize, glyph.clone());
    }
}

///Safer version of [`Stdout.queue(MoveTo)`] and [`Stdout.write()`]. Only writes to stdout if row and col are within bounds of terminal size.
pub fn draw_at(stdout: &mut StdoutBuffer, col: usize, row: usize, bytes: String) -> bool {
    let (max_width, max_height) = terminal::size().unwrap();
    if (col > max_width as usize) || (row > max_height as usize) {return false}
    stdout.write_all(col, row, bytes);

    return true;
}
