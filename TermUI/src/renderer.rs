use image::{Pixel, Rgb};
use crossterm::{event::*, terminal};
use crossterm::style::{Color, Stylize};
use crossterm::{QueueableCommand, cursor};
use std::any::Any;
use std::cell::RefCell;
use std::f32::MIN;
use std::f32::consts::PI;
use std::io::{Stdout, Write};
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Instant;

use super::matrix::*;
use super::*;

#[derive(Clone)]
pub struct Transform {
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub rotation: [f32; 3]
}

impl Transform {
    pub fn right(&self) -> [f32; 4] {
        Matrix::<4>::create_rotation(self.rotation[0], self.rotation[1], self.rotation[2]).mul_v(&[1.0, 0.0, 0.0, 0.0]).unwrap()
    }

    pub fn up(&self) -> [f32; 4] {
        Matrix::<4>::create_rotation(self.rotation[0], self.rotation[1], self.rotation[2]).mul_v(&[0.0, 1.0, 0.0, 0.0]).unwrap()
    }

    pub fn forward(&self) -> [f32; 4] {
        Matrix::<4>::create_rotation(self.rotation[0], self.rotation[1], self.rotation[2]).mul_v(&[0.0, 0.0, 1.0, 0.0]).unwrap()
    }

    pub fn left(&self) -> [f32; 4] {
        Matrix::<4>::create_rotation(self.rotation[0], self.rotation[1], self.rotation[2]).mul_v(&[-1.0, 0.0, 0.0, 0.0]).unwrap()
    }

    pub fn down(&self) -> [f32; 4] {
        Matrix::<4>::create_rotation(self.rotation[0], self.rotation[1], self.rotation[2]).mul_v(&[0.0, -1.0, 0.0, 0.0]).unwrap()
    }

    pub fn back(&self) -> [f32; 4] {
        Matrix::<4>::create_rotation(self.rotation[0], self.rotation[1], self.rotation[2]).mul_v(&[0.0, 0.0, -1.0, 0.0]).unwrap()
    }

    pub fn get_world(&self) -> Matrix<4> {
        Matrix::<4>::create_world(
            Matrix::<4>::create_translation(self.position[0], self.position[1], self.position[2]), 
            Matrix::<4>::create_rotation(self.rotation[0], self.rotation[1], self.rotation[2]),
            Matrix::<4>::create_scale(self.scale[0], self.scale[1], self.scale[2]),
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform{position: [0.0, 0.0, 0.0], scale: [1.0, 1.0, 1.0], rotation: [0.0, 0.0, 0.0]}    
    }

    
}

#[derive(Clone)]
pub enum RasterMode {
    AsTriangles,
    AsPoints,
    AsLines
}

pub enum CullMode {
    CullNone,
    CullFront,
    CullBack
}

pub struct VertexData {
    pub position: [f32; 3],
    pub color: Rgb<f32>,
    pub normal: [f32; 3],
    pub params: Vec<Box<dyn Any>>
}

impl VertexData {
    pub fn new_p(position: [f32; 3]) -> Self{
        VertexData { position, color: Rgb([0.0, 0.0, 0.0]), normal: [0.0, 0.0, 0.0], params: Vec::new() }
    }

    pub fn new_pc(position: [f32; 3], color: [f32; 3]) -> Self{
        VertexData { position, color: Rgb(color), normal: [0.0, 0.0, 0.0], params: Vec::new() }
    }

    pub fn new_pcn(position: [f32; 3], color: [f32; 3], normal: [f32; 3]) -> Self{
        VertexData { position, color: Rgb(color), normal, params: Vec::new() }
    }

    pub fn add_param<T>(&mut self, param: T) where T: 'static{
        self.params.push(Box::from(param));
    }
}

pub struct Quad {
    pub verts: Vec<([f32; 3], Rgb<f32>)>,
    pub indices: Vec<usize>,
    normal: [f32; 3]
}

impl Quad {
    pub fn new(position: [f32; 3], xy_scale: [f32; 2], color: Rgb<f32>, up: [f32; 3], right: [f32; 3], cullMode: CullMode) -> Self {
        let u: [f32; 3] = normalize(&cross(&normalize(&right), &normalize(&up)));
        let v: [f32; 3] = normalize(&cross(&normalize(&up), &u));
        let su: [f32; 3] = matrix::scale(&u, xy_scale[1]);
        let sv: [f32; 3] = matrix::scale(&v, xy_scale[0]);

        let p0: [f32; 3] = add(&position, &add(&su, &sv));    //bottom-right
        let p1: [f32; 3] = sub(&position, &add(&su, &sv));    //top-left
        let p2: [f32; 3] = sub(&position, &sub(&su, &sv));    //top-right
        let p3: [f32; 3] = add(&position, &sub(&su, &sv));    //bottom-left



        let verts: Vec<([f32; 3], Rgb<f32>)> = vec![(p0, color), (p1, color), (p2, color), (p3, color)];

        let indices: Vec<usize> = match cullMode {
            CullMode::CullFront => vec![3, 0, 2, 1, 2, 0],   //counter-clockwise rounding
            CullMode::CullBack => vec![0, 3, 2, 1, 2, 3],   //clock-wise rounding
            CullMode::CullNone => vec![3, 0, 2, 1, 2, 0, 3, 2, 0, 1, 0, 2],
        };
        Quad{verts, indices, normal: up}
    }
    
    pub fn from(quads: Vec<Quad>) -> Quad {
        let mut verts = Vec::new();
        let mut indices = Vec::new();
        for mut q in quads {
            verts.append(&mut q.verts);
            indices.append(&mut q.indices);
        }
        Quad { verts, indices, normal: [0.0, 1.0, 1.0] }
    }

    pub fn as_model(&self, transform: Transform, view:Matrix<4>, projection:Matrix<4>) -> Model {
        Model::new(self.verts.clone(), self.indices.clone(),transform, view, projection, RasterMode::AsTriangles)
    }

    pub fn roll(&mut self, other: &mut Quad) -> &mut Self {
        self.verts.append(&mut other.verts);
        self.indices.append(&mut other.indices);
        self
    }
    
}


#[derive(Clone, PartialEq)]
pub struct Fragment {
    pub color: Rgb<f32>,
    pub position: [f32; 3],
    pub ascii: String,
    pub normal: [f32; 3]
}

impl Default for Fragment {
    fn default() -> Self {
        Fragment {color: Rgb([255.0, 255.0, 255.0]), position: [MIN, MIN, MIN], ascii: String::from("XX"), normal: [0.0; 3]}
    }
}

pub struct StdoutBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Vec<String>>
}

impl StdoutBuffer {
    const cell_width: usize = 2;
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::new();
        let mut row_buf = Vec::new();
        row_buf.resize(width, String::new());
        buffer.resize(height, row_buf);


        StdoutBuffer {width, height, buffer}
    }

    pub fn write(&mut self, col: usize, row: usize, s: String) {
        //println!("[len: {}, col: {}, row: {}]: {}", s.len(), col, row, s.clone());
        if col < self.width && row < self.height{
            self.buffer[row][col] = s.to_string();
        }
    }

    pub fn write_all(&mut self, col: usize, row: usize, string: String) {
        for (i, s) in string.chars().enumerate() {
            self.write(col + i, row, s.to_string());
        }
    }

    pub fn write_to(&self, stdout: &mut Stdout) {
        let test = self.to_string();
        stdout.queue(cursor::MoveTo(0, 0)).unwrap();
        stdout.write_all(test.as_bytes()).unwrap();
    }

    pub fn to_string(&self) -> String {
        self.buffer.join(&"\n".to_string()).concat()
    }

    pub fn fill(&mut self, s: String) {
        let mut r = Vec::new();
        r.resize(self.width, s);
        self.buffer.fill(r);
    }
}

#[derive(Clone)]
pub struct Camera {
    pub transform: Transform,
    pub view: Matrix<4>,
    pub view_angles: (f32, f32),
    pub distance_to_target: f32
}



impl Camera {
    pub fn new(transform: Transform) -> Self {
        let mut p = transform.position.to_vec();
        p.push(1.0);
        let view = Matrix::from(vec![
            transform.right().to_vec(),
            transform.up().to_vec(),
            transform.forward().to_vec(),
            p
        ]).transpose();
        Camera {transform, view, view_angles: (0.0, 0.0), distance_to_target: 1.0}
    }

    pub fn look_at(&mut self, target: [f32; 3], up: [f32; 3]) -> Matrix::<4> {
        let eye = 
        Matrix::<4>::rotate_around(self.view_angles.1, self.view_angles.0, 0.0, target).mul(&Matrix::<4>::create_translation(target[0], target[1], target[2])).unwrap().mul_v::<3>(&[0.0, 0.0, self.distance_to_target, 0.0]).unwrap();

        let zaxis = normalize(&sub::<3>(&eye, &target));
        let xaxis = normalize(&cross(&up, &zaxis));
        let yaxis = cross(&zaxis, &xaxis);

        let orientation = Matrix::<4>::from(vec![
            vec![xaxis[0], yaxis[0], zaxis[0], 0.0],
            vec![xaxis[1], yaxis[1], zaxis[1], 0.0],
            vec![xaxis[2], yaxis[2], zaxis[2], 0.0],
            vec![0.0, 0.0, 0.0, self.distance_to_target],
        ]).transpose();

        let translation = Matrix::<4>::create_translation(-dot(&xaxis, &eye), -dot(&yaxis, &eye), -dot(&zaxis, &eye));
        
        orientation.mul(&translation).unwrap()

    }

    pub fn update(&mut self) {
        let mut p = self.transform.position.to_vec();
        p.push(1.0);
        self.view = Matrix::from(vec![
            self.transform.right().to_vec(),
            self.transform.up().to_vec(),
            self.transform.forward().to_vec(),
            p
        ]).transpose();     
    }
}


pub struct ScreenBuffer{
    pub rect: Rectangle,
    pub lightPosition: [f32; 3],
    pixels: Vec<Option<Fragment>>,
    pixel_update: Box<dyn Fn([f32; 3], Fragment, f32) -> (String, Rgb<f32>) + Send + Sync + 'static>,
    models: Vec<Model>,
    camera: Rc<RefCell<Camera>>
}


impl ScreenBuffer {
    
    pub fn new(rect: Rectangle, pixel_update: Box<dyn Fn([f32; 3], Fragment, f32) -> (String, Rgb<f32>) + Send + Sync +'static>, camera: Rc<RefCell<Camera>>) -> ScreenBuffer {
        let mut pixels = Vec::new();
        pixels.resize((rect.width * rect.height) as usize, None);
        
        ScreenBuffer {rect, lightPosition: [10.0, -10.0, 10.0], pixels, pixel_update, models: Vec::new(), camera: camera}
    }
    
    pub fn add_model(&mut self, m: &Model) {
        self.models.push(m.clone());
    }
    
    pub fn update(&mut self, delta_time: f32, poll_event: Option<Event>) {
        if poll_event.is_some() {
            match poll_event.unwrap() {
                Event::Mouse(m) => {
                    match m {
                        MouseEvent {kind: MouseEventKind::Down(MouseButton::Left),column: c, row: r, ..} => {
                            let px: usize = c.abs_diff(self.rect.x) as usize;
                            let py: usize = r.abs_diff(self.rect.y) as usize;
                        }
                        _ => {}
                    }
                },
                Event::Key(k) => {
                    match k {
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Up, ..} => {
                            self.camera.borrow_mut().view_angles.1 += 10.0 * PI/180.0 * delta_time;
                            //self.camera.update();
                            // let p = self.models[0].view.get_cell(3, 3);
                            // self.models[0].view.set_cell(3, 3,  p + 0.5);
                            
                            //self.models[0].transform.rotation.1 -= 50.0 * delta_time* PI/180.0;
                        },
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Down, ..} => {
                            self.camera.borrow_mut().view_angles.1 -= 10.0 * PI/180.0* delta_time;
                            //self.camera.update();
                            // let p = self.models[0].view.get_cell(3, 3);
                            // self.models[0].view.set_cell(3, 3,  p - 0.5);

                            //self.models[0].transform.rotation.1 += 50.0 * delta_time* PI/180.0;
                        },
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Left, ..} => {
                            self.camera.borrow_mut().view_angles.0 += 5.0 * PI/180.0* delta_time;
                            //self.camera.update();
                            //self.models[0].transform.rotation.0 -= 50.0 * delta_time* PI/180.0;//Angle::Radians(self.models[0].transform.rotation.1.to() + 0.001);
                        },
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Right, ..} => {
                            self.camera.borrow_mut().view_angles.0 -= 5.0 * PI/180.0* delta_time;
                            //self.camera.update();
                            //self.models[0].transform.rotation.0 += 50.0 * delta_time* PI/180.0; //Angle::Radians(self.models[0].transform.rotation.1.to() - 0.001);
                        },
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Char('w'), ..} => {
                            self.camera.borrow_mut().distance_to_target += 0.1* delta_time;
                            //self.camera.update();
                            //self.models[0].transform.rotation.0 += 50.0 * delta_time* PI/180.0; //Angle::Radians(self.models[0].transform.rotation.1.to() - 0.001);
                        }, 
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Char('s'), ..} => {
                            self.camera.borrow_mut().distance_to_target -= 0.1* delta_time;
                            //self.camera.update();
                            //self.models[0].transform.rotation.0 += 50.0 * delta_time* PI/180.0; //Angle::Radians(self.models[0].transform.rotation.1.to() - 0.001);
                        }, 
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        

        for m in self.models.iter_mut() {
            //m.transform.rotation.2 += 0.0001;
            //m.transform.rotation.1 += 0.0001;
            //m.transform.rotation.1 += 50.0 * delta_time * PI/180.0;
            //m.transform.rotation.0 += 50.0 * delta_time * PI/180.0;
            m.update((self.rect.x as f32 * 2.0, self.rect.y as f32), &mut self.camera.borrow_mut());
        }

        self.lightPosition = Matrix::<4>::rotate_y(0.1 * delta_time).mul_v(&self.lightPosition).unwrap();
    }
    
    
    
    
    pub const TILE_RES: usize = 5;
    pub fn draw(&mut self, stdout: &mut StdoutBuffer, time: Instant) {
        thread::scope(|s: &thread::Scope<'_, '_>| {
            let buffer: Arc<Mutex<Vec<Option<Fragment>>>> = Arc::new(Mutex::new(self.pixels.clone()));
            let mut handles: Vec<thread::ScopedJoinHandle<'_, ()>> = Vec::new();
            for m in self.models.iter_mut() {
                let buffer_clone: Arc<Mutex<Vec<Option<Fragment>>>> = Arc::clone(&buffer);
                let clipBox: Rectangle = Rectangle { 
                    x: self.rect.x, y: self.rect.y, width: self.rect.x + self.rect.width, height: self.rect.height + self.rect.y, fill: RectFill::Empty };
                let (t, p) = mpsc::channel();
                
                let handle: thread::ScopedJoinHandle<'_, ()> = s.spawn(move || {
                    let mut buf: Vec<Option<Fragment>> = buffer_clone.lock().unwrap().clone();
                    match m.raster_mode {
                        RasterMode::AsLines => m.rasterize_as_lines(&mut buf, &clipBox),
                        RasterMode::AsPoints => m.rasterize_as_points(&mut buf, &clipBox),
                        RasterMode::AsTriangles => m.rasterize_as_triangles(&mut buf, clipBox),
                    }
                    
                    t.send(buf).unwrap();
                });

                handles.push(handle);
                
                self.pixels = p.recv().unwrap();
                
                // self.pixels = r.iter()
                //     .zip(self.pixels.iter())
                //     .map(|(rf, pf )| 
                //     if rf.is_some() && pf.is_none() {rf.clone()}
                //     else {pf.clone()})
                //     .collect();
            }
        });
    


        let mut output: Vec<_> = Vec::with_capacity((self.rect.width * self.rect.height) as usize);
        output.resize((self.rect.width * self.rect.height) as usize, String::from(" "));

        let mut strings = Vec::<String>::new();
        let mut newline = false;
        let (max_width, max_height) = terminal::size().unwrap();
        for (i , frag) in self.pixels.iter().enumerate() {
            let row = i / self.rect.width as usize;
            let col = i % self.rect.width as usize;
            // if (row >= self.rect.height as usize - 1) || (row >= max_height as usize){
            //     break;
            // }

            // if (self.rect.x as usize > col) || (self.rect.y as usize > row) || (2 * col >= max_width as usize - 1){
            //     continue;
            // }

            // if (col >= self.rect.width as usize - 1) {
            //     newline = true;
            //     continue;
            // }
            // if newline {

            //     //strings.push(String::from("\n"));
            //     newline = false;
            // }
            
            if frag.is_none() {
                continue;
            }
            let frag = frag.clone().unwrap();
            let (ascii, c) = (self.pixel_update)(self.lightPosition, frag.clone(), time.elapsed().as_secs_f32());
            let [r, g, b] = c.0;
            
            //let out = ascii.with(Color::Rgb {r: r as u8, g: g as u8, b: b as u8}).to_string();
            
            if self.rect.x + self.rect.width > frag.position[0] as u16 && self.rect.y + self.rect.height > frag.position[1] as u16 {
                for (i, char) in ascii.chars().enumerate() {
                    stdout.write(frag.position[0] as usize * 2 + i, frag.position[1] as usize, char.with(Color::Rgb {r: r as u8, g: g as u8, b: b as u8}).to_string());
                    //stdout.write(frag.position[0] as usize * 2 + 1, frag.position[1] as usize, "X".to_string());
                }
            }

        }

        let test = String::from_iter(strings.into_iter());

        //stdout.write(self.rect.x as usize, self.rect.y as usize, test);
        // stdout.queue(cursor::MoveTo(self.rect.x, self.rect.y)).unwrap();
        //stdout.write_all(0, 0, test);



        self.pixels.fill(None);

    }
}