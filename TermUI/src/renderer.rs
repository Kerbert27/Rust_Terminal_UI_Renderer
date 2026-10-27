
use std::f32::MIN;
use std::f32::consts::PI;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};

use super::*;

#[derive(Clone, Copy)]
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

pub struct Quad {
    pub verts: Vec<VertexData>,
    pub indices: Vec<usize>,
}

impl Quad {
    pub fn new(position: [f32; 3], xy_scale: [f32; 2], color: Rgb<f32>, up: [f32; 3], right: [f32; 3], cullMode: CullMode) -> Self {
        let u: [f32; 3] = normalize(&cross(&normalize(&right), &normalize(&up)));
        let v: [f32; 3] = normalize(&cross(&normalize(&up), &u));
        let su: [f32; 3] = matrix::scale(&u, xy_scale[1]);
        let sv: [f32; 3] = matrix::scale(&v, xy_scale[0]);

        let p0: [f32; 3] = add(&position, &add::<3>(&su, &sv));    //bottom-right
        let p1: [f32; 3] = sub(&position, &add::<3>(&su, &sv));    //top-left
        let p2: [f32; 3] = sub(&position, &sub::<3>(&su, &sv));    //top-right
        let p3: [f32; 3] = add(&position, &sub::<3>(&su, &sv));    //bottom-left



        let verts: Vec<VertexData> = vec![
            VertexData::new_pc(p0, color), 
            VertexData::new_pc(p1, color), 
            VertexData::new_pc(p2, color), 
            VertexData::new_pc(p3, color)
        ];

        let indices: Vec<usize> = match cullMode {
            CullMode::CullFront => vec![3, 0, 2, 1, 2, 0],   //counter-clockwise rounding
            CullMode::CullBack => vec![0, 3, 2, 1, 2, 3],   //clock-wise rounding
            CullMode::CullNone => vec![3, 0, 2, 1, 2, 0, 0, 3, 2, 1, 2, 3],
        };
        Quad{verts, indices}
    }
    
    pub fn from(quads: Vec<Quad>) -> Quad {
        let mut verts = Vec::new();
        let mut indices = Vec::new();
        for mut q in quads {
            verts.append(&mut q.verts);
            indices.append(&mut q.indices);
        }
        Quad { verts, indices}
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
    pub ascii: char,
    pub normal: [f32; 3]
}

impl Default for Fragment {
    fn default() -> Self {
        Fragment {color: Rgb([255.0, 255.0, 255.0]), position: [MIN, MIN, MIN], ascii: 'X', normal: [0.0; 3]}
    }
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub rgb: [u8; 3]
}


#[derive(Clone)]
pub struct StdoutBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Vec<Cell>>,
}

impl StdoutBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::new();
        let mut row_buf = Vec::new();
        row_buf.resize(width, Cell {ch: ' ', rgb: [255, 255, 255]});
        buffer.resize(height, row_buf);


        StdoutBuffer {width, height, buffer}
    }

    pub fn write(&mut self, col: usize, row: usize, s: Cell) {
        if col < self.width && row < self.height{
            self.buffer[row][col] = s;
        }
    }

    pub fn write_all(&mut self, col: usize, row: usize, string: &String) {
        for (i, char) in string.chars().enumerate() {
            self.write(col + i, row, Cell {ch: char, rgb: [255, 255, 255]});
        }
    }

    #[track]
    pub fn write_to(&mut self, stdout: &mut Stdout) {
        //self.back_buffer = self.buffer.clone();
        let test = self.to_string();
        stdout.queue(cursor::MoveTo(0, 0)).unwrap();
        stdout.write_all(test.as_bytes()).unwrap();
    }


    #[track]
    pub fn to_string(&self) -> String {
        let mut out = String::with_capacity(self.width * self.height);
        let mut last_color: Option<[u8; 3]> = None;
        for (i, row) in self.buffer.iter().enumerate() {
            for cell in row.iter() {
                if cell.ch != ' ' && last_color != Some(cell.rgb){
                    use std::fmt::Write;
                    let [r, g, b] = cell.rgb;
                    write!(out, "\x1b[38;2;{r};{g};{b}m").unwrap();
                    last_color = Some(cell.rgb);
                }
                out.push(cell.ch);
            }
            if i + 1 != self.buffer.len() {
                out.push_str("\r\n");
            }
        }
        out.push_str("\x1b[0m");
        out
    }

    #[track]
    pub fn fill(&mut self, s: Cell) {
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
    pub distance_to_target: f32,
    pub target: [f32; 3]
}



impl Camera {
    pub fn new(transform: Transform, target: Transform) -> Self {
        let mut p = transform.position.to_vec();
        p.push(1.0);
        let view = Matrix::from(vec![
            transform.right().to_vec(),
            transform.up().to_vec(),
            transform.forward().to_vec(),
            p
        ]).transpose();
        Camera {transform, view, view_angles: (0.0, 0.0), distance_to_target: dist::<3>(&transform.position, &target.position), target: target.position}
    }

    pub fn look_at(&mut self, target: [f32; 3], up: [f32; 3]) -> Matrix::<4> {
        self.distance_to_target = dist::<3>(&self.transform.position, &target);
        let eye: [f32; 3] = 
        Matrix::<4>::create_rotation(self.view_angles.1, self.view_angles.0, 0.0)
        .mul(&Matrix::<4>::create_translation(target[0], target[1], target[2])).unwrap()
        .mul_v::<3>(&self.transform.position).unwrap();

        let zaxis: [f32; 3] = normalize(&sub::<3>(&target, &eye));
        let xaxis: [f32; 3] = normalize(&cross(&up, &zaxis));
        let yaxis: [f32; 3] = cross(&zaxis, &xaxis);

        let orientation: Matrix<4> = Matrix::<4>::from(vec![
            vec![xaxis[0], yaxis[0], zaxis[0], 0.0],
            vec![xaxis[1], yaxis[1], zaxis[1], 0.0],
            vec![xaxis[2], yaxis[2], zaxis[2], 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ]).transpose();

        let translation: Matrix<4> = Matrix::<4>::create_translation(-eye[0], -eye[1], -eye[2]);
        
        self.view = orientation.mul(&translation).unwrap();
        self.view.clone()
    }
}


pub struct Renderer{
    pub rect: Rectangle,
    pub lightPosition: [f32; 3],
    pub pixels: Vec<Option<Fragment>>,
    pixel_update: Option<Box<dyn Fn([f32; 3], &Fragment, f32) -> (char, Rgb<f32>) + Send + Sync + 'static>>,
    models: Vec<Model>,
    camera: Rc<RefCell<Camera>>
}

impl Clone for Renderer {
    fn clone(&self) -> Self {
        Self { rect: self.rect, lightPosition: self.lightPosition.clone(), pixels: self.pixels.clone(), pixel_update: None, models: self.models.clone(), camera: self.camera.clone()}
    }
}


impl Renderer {
    
    pub fn new(rect: Rectangle, pixel_update: Box<dyn Fn([f32; 3], &Fragment, f32) -> (char, Rgb<f32>) + Send + Sync +'static>, camera: Rc<RefCell<Camera>>) -> Renderer {
        let mut pixels = Vec::new();
        pixels.resize((rect.width * rect.height) as usize, None);
        
        Renderer {rect, lightPosition: [10.0, -10.0, 10.0], pixels, pixel_update: Some(pixel_update), models: Vec::new(), camera: camera}
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
                            self.camera.borrow_mut().view_angles.1 += 100.0 * PI/180.0 * delta_time;
                        },
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Down, ..} => {
                            self.camera.borrow_mut().view_angles.1 -= 100.0 * PI/180.0* delta_time;
                        },
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Left, ..} => {
                            self.camera.borrow_mut().view_angles.0 += 100.0 * PI/180.0* delta_time;
                        },
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Right, ..} => {
                            self.camera.borrow_mut().view_angles.0 -= 100.0 * PI/180.0* delta_time;
                        },
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Char('w'), ..} => {
                            self.camera.borrow_mut().transform.position[2]  += 10.0 * delta_time;
                        }, 
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Char('s'), ..} => {
                            self.camera.borrow_mut().transform.position[2] -= 10.0 * delta_time;
                        },
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Char('r'), ..} => {
                            self.camera.borrow_mut().transform.position[1]  += 10.0 * delta_time;
                        }, 
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Char('f'), ..} => {
                            self.camera.borrow_mut().transform.position[1] -= 10.0 * delta_time;
                        }, 
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Char('a'), ..} => {
                            self.camera.borrow_mut().transform.position[0] += 10.0 * delta_time;
                        }, 
                        KeyEvent {kind: KeyEventKind::Press, code: KeyCode::Char('d'), ..} => {
                            self.camera.borrow_mut().transform.position[0] -= 10.0 * delta_time;
                        }, 
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        

        for m in self.models.iter_mut() {
            m.update(&mut self.camera.borrow_mut());
        }

        self.lightPosition = Matrix::<4>::rotate_y(1.0 * delta_time).mul_v(&self.lightPosition).unwrap();
    }
    
    
    
    // #[track]
    // fn rasterize_mt(&mut self) {
    //     let pixel_lock = Arc::new(Mutex::new(self.pixels.clone()));
    //     let clipBox: Rectangle = Rectangle {x: self.rect.x, y: self.rect.y, width: self.rect.x + self.rect.width, height: self.rect.height + self.rect.y, fill: RectFill::Empty };
    //     thread::scope(|s| {
    //         for m in self.models.iter_mut() {
    //             let pixels = Arc::clone(&pixel_lock);
    //             s.spawn(move || {
    //                 match m.raster_mode {
    //                     RasterMode::AsLines => m.rasterize_as_lines(&mut pixels.lock().unwrap(), &clipBox),
    //                     RasterMode::AsPoints => m.rasterize_as_points(&mut pixels.lock().unwrap(), &clipBox),
    //                     RasterMode::AsTriangles => m.rasterize_as_triangles(&mut pixels.lock().unwrap(), &clipBox),
    //                 }   
    //                 //dbg!("Thread");
    //             });
    //         }
    //     });

    //     self.pixels = pixel_lock.lock().unwrap().to_vec();
    // }

    #[track]
    fn rasterize(&mut self, textures: &Vec<Texture>) {
        let clipBox: Rectangle = Rectangle {x: self.rect.x, y: self.rect.y, width: self.rect.x + self.rect.width, height: self.rect.height + self.rect.y, fill: RectFill::Empty };
        for m in self.models.iter_mut() {
            match m.raster_mode {
                RasterMode::AsLines => m.rasterize_as_lines(&mut self.pixels, &clipBox),
                RasterMode::AsPoints => m.rasterize_as_points(&mut self.pixels, &clipBox),
                RasterMode::AsTriangles => m.rasterize_as_triangles(&mut self.pixels, &clipBox, &textures),
            }   
        }
    }

    #[track]
    fn frag_shader(&mut self, stdout: &mut StdoutBuffer, time: Instant) {
        //Equivalent to fragment shading.
        if let Some(pixel_update) = &self.pixel_update {
            for frag in self.pixels.iter(){
                if let Some(f) = frag {
                    //Skip drawing if fragment is outside bounds of screen.
                    if self.rect.right() > f.position[0] as u16 && self.rect.bottom() > f.position[1] as u16 {
                        let x = f.position[0];
                        let y = f.position[1];
                        let (ascii, c) = (pixel_update)(self.lightPosition, f, time.elapsed().as_secs_f32());
                        let [r, g, b] = c.0;
                        stdout.write(x as usize * 2, y as usize, Cell {ch: ascii, rgb: [r as u8, g as u8, b as u8]});
                        stdout.write(x as usize * 2 + 1, y as usize, Cell {ch: ascii, rgb: [r as u8, g as u8, b as u8]});
                    }
                }
            }
        }
        self.pixels.fill(None);
    }

    pub fn draw(&mut self, stdout: &mut StdoutBuffer, time: Instant, textures: &Vec<Texture>) {

        self.rasterize(textures);
        
        self.frag_shader(stdout, time);

    }
}