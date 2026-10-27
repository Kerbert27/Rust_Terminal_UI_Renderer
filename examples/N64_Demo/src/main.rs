#![allow(non_snake_case)]

use crossterm::event::{poll, read};
use crossterm::terminal;
use image::Rgb;
use std::f32::consts::PI;
use std::process::Command;
use std::{env};
use std::rc::Rc;
use std::cell::RefCell;

use track::*;

use lib::*;

const TARGET_FRAMERATE: f32 = 60.0;
const WINDOW_WIDTH: usize = 160;
const WINDOW_HEIGHT: usize = 50;

fn main() {
    let _args: Vec<String> = env::args().collect();

    //println!("Args: {:?}", _args);
    let emptyString: &String = &String::new();
    let cmd: &str = _args.last().unwrap_or(emptyString).as_str();
    let mut _window: Command = Command::new("cmd");
    match cmd {
        "original" => {
            Set_Window_Size(WINDOW_WIDTH, WINDOW_HEIGHT);
            LaunchTerminal(Box::new(InitializeContext), Box::new(Update), Box::new(Draw));
        },
        // "hot" => {
        //     Command::new("cmd").args(["/c", "start", "cmd", "/k", "cargo watch -w lib -x 'build -p lib'"]).spawn().expect("Failed to create new Terminal");
        //     _window.args(["/c", "start", "cmd", "/k", "cargo run -- original"]).spawn().expect("Failed to create new Terminal");
        // }
        _ => {
            _window.args(["/c", "start", "cmd", "/k", "cargo run -- original"]).spawn().expect("Failed to create new Terminal");
        }
    }
    
}

const BLOCK: char = '█';
const SHADE_LIGHT: char = '▓';
const SHADE_MED: char = '▒';
const SHADE_DARK : char = '░';

//Fragment Shader
#[track]
fn pixel_update(lightPositon: [f32;3], input: &Fragment, time: f32) -> (char, image::Rgb<f32>) {
    let ascii = match input.normal {
        d if dot::<3>(&d, &lightPositon) >= 0.2 => BLOCK,
        d if dot::<3>(&d, &lightPositon) >= 0.001 => SHADE_LIGHT,
        d if dot::<3>(&d, &lightPositon) >= -0.5 => SHADE_MED,
        _ => SHADE_DARK
    };

    let color = match input.normal {
        d if dot::<3>(&d, &[0.0, 1.0, 0.0]) >= 0.1 => Rgb([255.0, 0.0, 0.0]),
        d if dot::<3>(&d, &[0.0, -1.0, 0.0]) >= 0.1 => Rgb([255.0, 0.0, 0.0]),
        d if dot::<3>(&d, &[1.0, 0.0, 0.0]) >= 0.1 => Rgb([0.0, 255.0, 0.0]),
        d if dot::<3>(&d, &[-1.0, 0.0, 0.0]) >= 0.1 => Rgb([0.0, 255.0, 0.0]),
        d if dot::<3>(&d, &[0.0, 0.0, 1.0]) >= 0.1 => Rgb([0.0, 0.0, 255.0]),
        d if dot::<3>(&d, &[0.0, 0.0, -1.0]) >= 0.1 => Rgb([0.0, 0.0, 255.0]),
        _ => Rgb([255.0, 255.0, 255.0])
    };
;

    return (ascii, color);
}
fn InitializeContext() -> Context {
    
    let (W, H) = terminal::size().unwrap();
    

    let buffer = Box::new(StdoutBuffer::new(W as usize, H as usize));

    let mut elements: Vec<Element> = Vec::new();

    let transform = Transform {position: [0.0, 0.0, 0.0], scale: [1.0,1.0, 1.0], rotation: [0.0, 0.0, 0.0]};
    
    let camera = Camera::new(Transform {position: [0.0, 0.0, -5.0], scale: [0.0,0.0,0.0], rotation: [0.0, 0.0, 0.0]}, transform);

    let cameraPtr = Rc::from(RefCell::new(camera));

    let mut renderer: Renderer = Renderer::new(
        Rectangle { x: 0, y: 0, width: W, height: H, fill: RectFill::Empty},
        Box::new(pixel_update),
        Rc::clone(&cameraPtr)
    );

    let projectionP: Matrix<4> = Matrix::create_projection_persp(W as f32 / H as f32 , (20.0 as f32).to_radians(), 1000.0, 1.0);
    let projectionO: Matrix<4> = Matrix::create_projection_ortho(0.0, 4.0 , 4.0, 0.0, -10.0, 10.0);

    let modelLoad = Model::load_from_obj("../../obj/N64.obj", transform, cameraPtr.borrow_mut().view.clone(), projectionP.clone(), RasterMode::AsTriangles, &mut Vec::new());

    let model: Model = if modelLoad.is_err() {
        init_model(transform.clone(), cameraPtr.borrow_mut().view.clone(), projectionP.clone())
    } else {
        modelLoad.unwrap()
    };
    renderer.add_model(&model);
    

    
    elements.push(Element::Renderer(renderer, Some(Box::new(RendererUpdate))));
    let text = TextBox::new(Rectangle{x: 0, y: 0, width: 35, height: 7, fill: RectFill::EmptyAndBoxed}, String::from("Hello"));
    elements.push(Element::TextBox(text, Some(Box::new(StatsUpdate))));

    let logger = TextBox::new(Rectangle {x: 0, y: 10, width: 70, height: 5, fill: RectFill::EmptyAndBoxed}, String::new());
    elements.push(Element::TextBox(logger, Some(Box::new(LoggerUpdate))));


    Context {frame: 0, locked_framerate: None, delta_time: 0.0, poll_event: None, buffer, W, H, camera: cameraPtr,  elements, start_time: time::Instant::now(), textures: Vec::new()}
}

fn init_model(transform: Transform, view: Matrix<4>, projection: Matrix<4>) -> Model {
    let mesh = vec![
        [0.0, 0.0, 0.0],
        [0.0, 0.0, -1.0],
        [0.0, 3.0, 0.0],
        [0.0, 3.0, -1.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, -1.0],
        [1.0, 3.0, 0.0],
        [1.0, 3.0, -1.0],
    ];

    let bridge = vec![
        [1.0, 1.5, 0.0],
        [1.0, 1.5, -1.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, -1.0],

        [2.0, 3.0, 0.0],
        [2.0, 3.0, -1.0],
        [2.0, 1.5, 0.0],
        [2.0, 1.5, -1.0],
    ];

    let mesh = mesh.iter().map(|v| VertexData::new_p(*v)).collect();
    let bridge = bridge.iter().map(|v| VertexData::new_p(*v)).collect();

    let mut model = Model::new(
        mesh,
        vec![
            0, 6, 4,    //wrong
            1, 7, 3,    //wrong
            0, 3, 2,    //wrong
            4, 7, 5,    //wrong
            0, 4, 1,    //wrong
            3, 7, 6,    //wrong
            
            0, 2, 6,
            1, 5, 7,
            0, 1, 3,
            4, 6, 7,
            1, 4, 5,
            2, 3, 6
        ]
        ,transform.clone(), view.clone(), projection.clone(), lib::RasterMode::AsTriangles
    );

    let b_model = Model::new(
        bridge,
        vec![
            0, 4, 6,    //wrong
            1, 3, 7,    //wrong
            0, 2, 3,    //wrong
            4, 5, 7,    //wrong
            0, 1, 4,    //wrong
            3, 6, 7,    //wrong
            
            0, 6, 2,
            1, 7, 5,
            0, 3, 1,
            4, 7, 6,
            1, 5, 4,
            2, 6, 3
        ]
        ,transform, view.clone(), projection.clone(), lib::RasterMode::AsTriangles
    );



    let m = model.clone();
    let mut b = b_model.clone();
    for i in 0..4 {
        let v_add = match i {
            0 => &[2.0, 0.0, 0.0],
            1 => &[0.0, 0.0, 2.0],
            2 => &[2.0, 0.0, 2.0],
            _ => &[0.0, 0.0, 0.0]
        };
        if i < 3{
            let mut v = m.vertices.iter().map(|v| VertexData::new_p(add(&v.position[0..3], v_add))).collect();
            let mut ind = m.indices.iter().map(|i| i + model.vertices.len()).collect();
            model.vertices.append(&mut v);
            model.indices.append(&mut ind);
        }
        //model.combine(&mut m.clone());
        b.vertices = b.vertices.iter().map(
            |v| 
            VertexData::new_p(Matrix::<4>::rotate_around(0.0, -PI/2.0, 0.0, [-1.5, 0.0, -0.5]).mul_v(&v.position).unwrap())
        ).collect();
        let mut b_ind = b.indices.iter().map(|i| i + model.vertices.len()).collect();
        model.vertices.append(&mut b.vertices.clone());
        model.indices.append(&mut b_ind);
    }
    
    model.vertices = model.vertices.iter_mut().map(
        |v| 
        {
            v.position = add(&v.position, &[-1.5, -1.5, -0.5, 0.0]);
            *v
    }
    ).collect();

    model.save_to_obj("../../obj/N64.obj").unwrap();
    return model;
}

fn StatsUpdate(textbox: &mut TextBox, context: &mut Context) {
    if context.frame % 30 == 0 {
        let mut out = String::new();
        out.push_str(&format!("FPS: {}\n", 1.0 / context.delta_time));
        for tracker in track::get_all() {
            out.push_str(&format!("{}: {:.4} ms/frame", tracker.function_name, tracker.total_time.as_secs_f32()).to_string());
            track::reset(tracker.function_name);
            out.push('\n');
        }
        textbox.text = out;
    }
}

fn LoggerUpdate(textbox: &mut TextBox, context: &mut Context) {
    textbox.text.clear();
    for (i, log) in LOG.write().unwrap().iter().enumerate() {
        if i >= textbox.rect.height as usize {
            break;
        }
        textbox.text.push_str(log);
        textbox.text.push_str("\n");
    }
}


fn RendererUpdate(renderer: &mut Renderer, context: &mut Context) {
    renderer.update(context.delta_time, context.poll_event.clone());
    
}

fn Update(context: &mut Context) {
    if poll(time::Duration::ZERO).unwrap() {
        context.poll_event = read().ok();
    }
    
    let mut elements = std::mem::take(&mut context.elements);
    for e in elements.iter_mut() {
        Element::Update(e, context);
    }
    context.elements = elements;
}


fn Draw(context: &mut Context) {
    for e in context.elements.iter_mut() {
        match e {
            Element::Box(r, _) => r.draw(&mut context.buffer),
            Element::Sprite(s, _) => s.draw(&mut context.buffer),
            Element::TextBox(t, _) => t.draw(&mut context.buffer),
            Element::Renderer(p, _) => p.draw(&mut context.buffer, context.start_time, &context.textures),
            Element::Graph(g, _) => g.draw(&mut context.buffer),
            Element::Timeline(t, _) => t.draw(&mut context.buffer),
        }
    }

    

    
}