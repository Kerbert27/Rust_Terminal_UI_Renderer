#![allow(non_snake_case)]

use crossterm::event::{self, Event, poll, read};
use crossterm::{QueueableCommand, cursor};
use std::cell::RefCell;
use std::f32::consts::PI;
use std::io::{self, Result, Stdout, Write};
use std::process::Command;
use std::rc::Rc;
use std::env;

use lib::*;

const TARGET_FRAMERATE: f32 = 60.0;

fn main() {
    let _args: Vec<String> = env::args().collect();

    //println!("Args: {:?}", _args);
    let emptyString: &String = &String::new();
    let cmd: &str = _args.last().unwrap_or(emptyString).as_str();
    let mut _window: Command = Command::new("cmd");
    match cmd {
        "original" => {LaunchTerminal();},
        // "hot" => {
        //     Command::new("cmd").args(["/c", "start", "cmd", "/k", "cargo watch -w lib -x 'build -p lib'"]).spawn().expect("Failed to create new Terminal");
        //     _window.args(["/c", "start", "cmd", "/k", "cargo run -- original"]).spawn().expect("Failed to create new Terminal");
        // }
        _ => {
            _window.args(["/c", "start", "cmd", "/k", "cargo run -- original"]).spawn().expect("Failed to create new Terminal");
        }
    }
    
}


pub struct Context {
    pub frame: u32,
    pub delta_time: f32,
    pub poll_event: Option<Event>,
    pub buffer: Box<StdoutBuffer>,
    pub W: u16,
    pub H: u16,
    pub elements: Vec<Element>,
    pub camera: Rc<RefCell<Camera>>,
    pub draw_time: Timer,
    pub start_time: Instant,
}


fn LaunchTerminal() {
    
    let mut previous_time: time::Instant = time::Instant::now();
    let mut stdout:Stdout  = io::stdout();
    stdout.queue(terminal::SetSize(200, 200)).unwrap();
    stdout.queue(terminal::EnterAlternateScreen).unwrap();
    stdout.queue(terminal::DisableLineWrap).unwrap();
    terminal::enable_raw_mode().unwrap();
    stdout.queue(cursor::DisableBlinking).unwrap();
    stdout.queue(cursor::Hide).unwrap();
    stdout.flush().unwrap();
    let mut context: Context = InitializeContext();
    context.draw_time = Timer::new(10);
    context.start_time = time::Instant::now();
    loop {
        Update(&mut context).unwrap();
        if context.delta_time >= 1.0 / TARGET_FRAMERATE {
            stdout.queue(terminal::BeginSynchronizedUpdate).unwrap();
            stdout.queue(cursor::Hide).unwrap();
            stdout.queue(terminal::DisableLineWrap).unwrap();
            stdout.queue(event::EnableMouseCapture).unwrap();
            stdout.queue(cursor::DisableBlinking).unwrap();
            context.buffer.fill(" ".to_string());
            Draw(&mut context);
            stdout.queue(terminal::EndSynchronizedUpdate).unwrap();
            
            
            
            context.buffer.write_to(&mut stdout);
            stdout.flush().unwrap();
            context.frame = (context.frame + 1) % TARGET_FRAMERATE as u32;
            
            previous_time = time::Instant::now();
        }
        
        
        
        context.delta_time = previous_time.elapsed().as_secs_f32();
        
    }
}



fn pixel_update(lightPositon: [f32;3], input: Fragment, time: f32) -> (String, image::Rgb<f32>) {
    let ascii = match input.normal {
        d if dot::<3>(&d, &lightPositon) >= 0.2 => String::from("██"),
        d if dot::<3>(&d, &lightPositon) >= 0.001 => String::from("▓▓"),
        d if dot::<3>(&d, &lightPositon) >= -0.5 => String::from("▒▒"),
        _ => String::from("░░")
    };

    let n = match input.normal {
        d if dot::<3>(&d, &[0.0, 1.0, 0.0]) >= 0.1 => Rgb([255.0, 0.0, 0.0]),
        d if dot::<3>(&d, &[0.0, -1.0, 0.0]) >= 0.1 => Rgb([255.0, 0.0, 0.0]),
        d if dot::<3>(&d, &[1.0, 0.0, 0.0]) >= 0.1 => Rgb([0.0, 255.0, 0.0]),
        d if dot::<3>(&d, &[-1.0, 0.0, 0.0]) >= 0.1 => Rgb([0.0, 255.0, 0.0]),
        d if dot::<3>(&d, &[0.0, 0.0, 1.0]) >= 0.1 => Rgb([0.0, 0.0, 255.0]),
        d if dot::<3>(&d, &[0.0, 0.0, -1.0]) >= 0.1 => Rgb([0.0, 0.0, 255.0]),
        _ => Rgb([255.0, 255.0, 255.0])
    };

    let c = Matrix::lerp(input.color.0, input.color.0.map(|p| -p + 255.0), (time % 1.0));
    return (ascii, input.color);
}

fn InitializeContext() -> Context {
    
    let (W, H) = terminal::size().unwrap();
    

    let buffer = Box::new(StdoutBuffer::new(W as usize, H as usize));

    let mut elements: Vec<Element> = Vec::new();

    let camera = Camera::new(Transform {position: [0.0, 0.0, 5.0], scale: [0.0,0.0,0.0], rotation: [0.0, 0.0, 0.0]});
    let cameraPtr = Rc::from(RefCell::new(camera));

    let mut pixArray: ScreenBuffer = ScreenBuffer::new(
        Rectangle { x: 0, y: 0, width: W, height: H, fill: RectFill::Empty},
        Box::new(pixel_update),
        Rc::clone(&cameraPtr)
    );


    let transform = Transform {position: [0.0, 0.0, 0.0], scale: [1.0,1.0, 1.0], rotation: [0.0, 0.0, 0.0]};


    let projectionP: Matrix<4> = Matrix::create_projection_persp(1.0 , PI/4.0, 100.0, 1.0);
    let projectionO: Matrix<4> = Matrix::create_projection_ortho(0.0, 4.0 , 4.0, 0.0, -10.0, 10.0);

    let model = Model::load_from_obj("../../obj/gourd.obj", transform, cameraPtr.borrow_mut().view.clone(), projectionP, RasterMode::AsTriangles);

    pixArray.add_model(&model);
    

    elements.push(Element::ScreenBuffer(pixArray));

    let text = TextBox::new(Rectangle{x: 0, y: 0, width: 20, height: 5, fill: RectFill::EmptyAndBoxed}, String::from("Hello"));
    elements.push(Element::TextBox(text));


    Context {frame: 0, delta_time: 0.0, poll_event: None, buffer, W, H,  elements, camera: cameraPtr, draw_time: Timer::new(1), start_time: time::Instant::now()}
}



fn Update(context: &mut Context) -> Result<bool> {
    if poll(time::Duration::ZERO).unwrap() {
        context.poll_event = read().ok();
    }

    for e in context.elements.iter_mut() {
        match e {
            Element::Box(r) => {},
            Element::Sprite(s) => {},
            Element::TextBox(t) => {
                t.text = format!("(X, Y): {}, {}", context.camera.borrow().view_angles.0, context.camera.borrow().view_angles.1);
            },
            Element::ScreenBuffer(p) => {
                p.update(context.delta_time as f32, context.poll_event.clone());
            },
            Element::Graph(g) => {
                //g.update(|g| g.set_at(0, context.frame as f32).unwrap());
            },
            Element::Timeline(t) => {
                //dbg!(context.frame);
                if context.frame % 10 == 0 {
                    //t.push_step("Plot 1", context.start_time.elapsed().as_secs_f32(), track::get_or_default("Draw").get_mean_time().as_secs_f32());
                    //t.push_step("Plot 2", context.start_time.elapsed().as_secs_f32(), track::get_or_default("draw_at").total_time.as_secs_f32());
                }
            }
            
        }
    }
    Ok(true)
}


fn Draw(context: &mut Context) {
    for e in context.elements.iter_mut() {
        match e {
            Element::Box(r) => r.draw(&mut context.buffer),
            Element::Sprite(s) => s.draw(&mut context.buffer),
            Element::TextBox(t) => t.draw(&mut context.buffer),
            Element::ScreenBuffer(p) => p.draw(&mut context.buffer, context.start_time),
            Element::Graph(g) => g.draw(&mut context.buffer),
            Element::Timeline(t) => t.draw(&mut context.buffer),
        }
    }     
}