#![allow(non_snake_case)]

use crossterm::event::{poll, read};
use crossterm::terminal;
use std::process::Command;
use std::{env};

use track::*;

use lib::*;

const WINDOW_WIDTH: usize = 160;
const WINDOW_HEIGHT: usize = 50;

fn main() {
    let _args: Vec<String> = env::args().collect();
    let empty_string: &String = &String::new();
    let cmd: &str = _args.last().unwrap_or(empty_string).as_str();
    let mut _window: Command = Command::new("cmd");
    match cmd {
        "original" => {
            Set_Window_Size(WINDOW_WIDTH, WINDOW_HEIGHT);
            LaunchTerminal(Box::new(InitializeContext), Box::new(Update), Box::new(Draw));
        },
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
fn pixel_update(light_positon: [f32;3], input: &Fragment, time: f32) -> (char, image::Rgb<f32>) {
    let ascii = match input.normal {
        d if dot::<3>(&d, &light_positon) >= 0.2 => BLOCK,
        d if dot::<3>(&d, &light_positon) >= 0.001 => SHADE_LIGHT,
        d if dot::<3>(&d, &light_positon) >= -0.5 => SHADE_MED,
        _ => SHADE_DARK
    };

    return (ascii, input.color);
}
fn InitializeContext() -> Context {
    
    let (W, H) = terminal::size().unwrap();
    

    let buffer = Box::new(StdoutBuffer::new(W as usize, H as usize));

    let mut elements: Vec<Element> = Vec::new();

    let transform = Transform {position: [0.0; 3], scale: [0.1; 3], rotation: [0.0; 3]};
    
    let camera = Camera::new(Transform {position: [0.0, 0.0, -20.0], scale: [0.0; 3], rotation: [0.0; 3]}, transform);

    let cameraPtr = Rc::from(RefCell::new(camera));

    let mut renderer: Renderer = Renderer::new(
        Rectangle { x: 0, y: 0, width: W, height: H, fill: RectFill::Empty},
        Box::new(pixel_update),
        Rc::clone(&cameraPtr)
    );

    let projectionP: Matrix<4> = Matrix::create_projection_persp(W as f32 / H as f32 , (20.0 as f32).to_radians(), 1000.0, 1.0);
    let projectionO: Matrix<4> = Matrix::create_projection_ortho(0.0, 4.0 , 4.0, 0.0, -10.0, 10.0);

    let model = Model::load_from_obj("../../obj/teapot.obj", transform, cameraPtr.borrow_mut().view.clone(), projectionP.clone(), RasterMode::AsTriangles, &mut Vec::new()).unwrap();
    renderer.add_model(&model);
    

    
    elements.push(Element::Renderer(renderer, Some(Box::new(RendererUpdate))));


    let text = TextBox::new(Rectangle{x: 0, y: 0, width: 35, height: 7, fill: RectFill::EmptyAndBoxed}, String::from("Hello World"));
    elements.push(Element::TextBox(text, Some(Box::new(TextboxUpdate))));



    Context {
        frame: 0, 
        locked_framerate: None,
        delta_time: 0.0, 
        poll_event: None, 
        buffer, 
        W, 
        H, 
        camera: cameraPtr,  
        elements, 
        start_time: time::Instant::now(), 
        textures: Vec::new()
    }
}


fn TextboxUpdate(textbox: &mut TextBox, context: &mut Context) {
    //...
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