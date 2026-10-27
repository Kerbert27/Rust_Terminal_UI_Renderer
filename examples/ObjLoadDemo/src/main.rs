#![allow(non_snake_case)]

use crossterm::event::{poll, read};
use std::process::Command;
use std::env;
use lib::*;

const WIDTH: usize = 200;
const HEIGHT: usize = 50;

const TARGET_FRAMERATE: f32 = 60.0;


fn main() {
    let _args: Vec<String> = env::args().collect();

    let emptyString: &String = &String::new();
    let cmd: &str = _args.last().unwrap_or(emptyString).as_str();
    let mut _window: Command = Command::new("cmd");
    match cmd {
        "original" => {
            Set_Window_Size(WIDTH, HEIGHT);
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

    // let color = Rgb([
    //     Matrix::lerp([0.0], [255.0], (dot::<3>(&input.normal, &[1.0, 0.0, 0.0]) + 1.0) * 0.5)[0],
    //     Matrix::lerp([0.0], [255.0], (dot::<3>(&input.normal, &[0.0, 1.0, 0.0]) + 1.0) * 0.5)[0],
    //     Matrix::lerp([0.0], [255.0], (dot::<3>(&input.normal, &[0.0, 0.0, 1.0]) + 1.0) * 0.5)[0],
    // ]);
;

    return (ascii, input.color);
}

fn InitializeContext() -> Context {
    
    let (W, H) = terminal::size().unwrap();
    

    let buffer = Box::new(StdoutBuffer::new(W as usize, H as usize));

    let mut elements: Vec<Element> = Vec::new();
    let transform = Transform {position: [0.0; 3], scale: [0.1; 3], rotation: [0.0; 3]};
    let transform2 = Transform {position: [0.0; 3], scale: [1.1; 3], rotation: [0.0;3 ]};

    let camera = Camera::new(Transform {position: [0.0, 10.0, -10.0], scale: [0.0; 3], rotation: [0.0; 3]}, transform2);
    let cameraPtr = Rc::from(RefCell::new(camera));

    let mut renderer: Renderer = Renderer::new(
        Rectangle { x: 0, y: 0, width: W, height: H, fill: RectFill::Empty},
        Box::new(pixel_update),
        Rc::clone(&cameraPtr)
    );




    let projectionP: Matrix<4> = Matrix::create_projection_persp(W as f32/ H as f32 , (20.0 as f32).to_radians(), 1000.0, 0.01);
    let projectionO: Matrix<4> = Matrix::create_projection_ortho(0.0, 0.1 * (W as f32/ H as f32), 0.1, 0.0, 10.0, 0.1);

    let mut materials = Vec::new();
    //let model = Model::load_from_obj("../../obj/teapot.obj", transform, cameraPtr.borrow_mut().view.clone(), projectionP.clone(), RasterMode::AsTriangles, &mut materials).unwrap();
    let model2 = Model::load_from_obj("../../obj/link/source/pose.obj", transform2, cameraPtr.borrow_mut().view.clone(), projectionP.clone(), RasterMode::AsTriangles, &mut materials).unwrap();
    //let mut model3 = Model::load_from_obj("../../obj/oot-link/source/Link (Adult)/Link (Adult)/Link Adult.obj", transform, cameraPtr.borrow_mut().view.clone(), projectionP.clone(), RasterMode::AsTriangles, &mut materials).unwrap();
    materials.iter_mut().find(|m| *m == "Hair").and_then(|m| Some(*m = "BodyB".to_string()));
    let textures = load_materials("../../obj/link/textures", &mut materials);
    

    let center = Model::point([0.0, 19.0, 0.0], cameraPtr.borrow_mut().view.clone(), projectionP.clone());
    cameraPtr.borrow_mut().target = model2.transform.position;
    let planeTransform = Transform {position: [0.0; 3], scale: [50.0;3], rotation: [0.0; 3]};
    //let plane = Model::plane(planeTransform, cameraPtr.borrow_mut().view.clone(), projectionP.clone(), CullMode::CullBack);

    renderer.add_model(&model2);    
    //renderer.add_model(&plane);




    elements.push(Element::Renderer(renderer, Some(Box::new(RendererUpdate))));

    let text = TextBox::new(Rectangle{x: 0, y: 0, width: 30, height: 6, fill: RectFill::EmptyAndBoxed}, String::from("Hello"));
    elements.push(Element::TextBox(text, Some(Box::new(StatsUpdate))));

    let logger = TextBox::new(Rectangle {x: 0, y: 42, width: 70, height: 7, fill: RectFill::EmptyAndBoxed}, String::new());
    elements.push(Element::TextBox(logger, Some(Box::new(LoggerUpdate))));

    let mut timeline = Timeline::new(Rectangle{x: 10, y: 20, width: 25, height: 20, fill: RectFill::EmptyAndBoxed}, 25, RangeType::Average, RangeType::MinMax);
    timeline.add_time_plot("FPS", Variable {name: "FPS", range: (0.00, 60.00)}, Rgb([0, 0, 255]));
    timeline.add_time_plot("Raster", Variable {name: "Raster", range: (0.00, 60.00)}, Rgb([255, 0, 0]));
    timeline.add_time_plot("Draw", Variable {name: "Draw", range: (0.00, 60.00)}, Rgb([0, 255, 0]));

    //elements.push(Element::Timeline(timeline, Some(Box::new(TimelineUpdate))));


    Context {
        frame: 0, 
        locked_framerate: None,
        delta_time: 0.0, 
        poll_event: None, 
        buffer, 
        W, 
        H,  
        elements, 
        camera: cameraPtr, 
        start_time: time::Instant::now(), 
        textures
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

fn StatsUpdate(textbox: &mut TextBox, context: &mut Context) {
    let mut out = String::new();
    out.push_str(&format!("FPS: {:.2}\n", 1.0 / context.delta_time));
    for tracker in track::get_all() {
        out.push_str(&format!("{}: {:.4} ms/frame", tracker.function_name, tracker.total_time.as_secs_f32()).to_string());
        track::reset(tracker.function_name);
        out.push('\n');
    }
    textbox.text = out;
}

fn TimelineUpdate(timeline: &mut Timeline, context: &mut Context) {
    if context.start_time.elapsed().as_millis() % 100 <= 5 {
        timeline.push_step("FPS", context.start_time.elapsed().as_secs_f32(), 1.0 / context.delta_time);
        let mut rt = track::get_or_default("rasterize").last_time.as_secs_f32();
        let mut dt = track::get_or_default("Draw").last_time.as_secs_f32();
        if rt == 0.0 {
            rt = 1.0;
        }
        if dt == 0.0 {
            dt = 1.0;
        }
        timeline.push_step("Raster", context.start_time.elapsed().as_secs_f32(), 1.0 / rt);
        timeline.push_step("Draw", context.start_time.elapsed().as_secs_f32(), 1.0 / dt);
    }
}

fn RendererUpdate(renderer: &mut Renderer, context: &mut Context) {
    //renderer.update(context.delta_time, context.poll_event.clone());
    for x in 0..context.buffer.width {
        for y in 0..context.buffer.height {
            context.buffer.write(x, y, Cell {ch: 'X', rgb: [x as u8, y as u8, 255]})
        }
    }
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
            Element::TextBox(t, _) => {
                t.draw(&mut context.buffer);
            },
            Element::Renderer(p, _) => p.draw(&mut context.buffer, context.start_time, &mut context.textures),
            Element::Graph(g, _) => g.draw(&mut context.buffer),
            Element::Timeline(t, _) => t.draw(&mut context.buffer),
        }
    }     
}