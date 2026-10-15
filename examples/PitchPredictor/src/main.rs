use std::{collections::HashMap, hash::RandomState, iter::Map};
use ndarray::*;
use nnl::prelude::*;
pub mod model;
pub mod data;
pub mod training;

#[derive(Clone, Copy)]
pub enum Call {
    Fair,
    Strike,
    Ball,
    Foul
}

impl Call {
    pub fn rand() -> Self {
        match rand::random_range(0..4) {
            0 => Call::Ball,
            1 => Call::Fair,
            2 => Call::Foul,
            3 => Call::Strike,
            _ => Call::Fair
        }
    }
}

#[derive(Clone, Copy)]
pub enum Handedness {
    Left,
    Right
}

#[derive(Clone, Copy)]
pub enum Result {
    Walk,
    Strikeout,
    HitOut,
    Single, //integer value is RBI
    Double,
    Triple,
    HomeRun,
    Sac         //Batter was HitOut but advanced runner(s)
}

#[derive(Clone, Copy)]
pub enum PitchType {
    ///4-Seam Fastball
    FS,     
    ///2-Seam Fastball
    TS,    
    ///Cut-Fastball
    CF, 
    ///Change-up
    CH, 
    ///Slider
    SL, 
    ///Curveball
    CB,     
    ///Screwball
    SB,    
}

#[derive(Clone, Copy)]
pub struct Pitch {

    pub pitchType: PitchType,
    pub velocity: f32,
    pub call: Option<Call>,
    pub result: Option<Result>,
    pub zone: i32,
}

impl Pitch {
    pub fn to_array(&self) -> Array1<f32> {
        Array1::from_vec(vec![
            self.pitchType as i64 as f32,
            self.velocity as f32,
            self.zone as f32,
            match self.call {
                None => 0.0,
                Some(c) => c as i64 as f32
            },
            match self.result {
                None => 0.0,
                Some(r) => r as i64 as f32
            }
        ])
    }
}

#[derive(Clone, Copy)]
pub struct GameState {
    pub inning: i8,
    pub score: (i32, i32),   //Own, Opponent
    pub loaded_bases: [bool; 3],
    pub balls: i8,      //balls before pitch is thrown
    pub strikes: i8,    //strikes before pitch is thrown
    pub out: i8,        //0-2
}


impl GameState {
    pub fn init() -> Self {
        GameState {
            inning: 0,
            score: (0, 0),
            loaded_bases: [false; 3],
            balls: 0,
            strikes: 0,
            out: 0,
        }
    }

    pub fn strike(&mut self) -> Option<Result> {
        self.strikes += 1;
        if self.strikes >= 3 {
            self.strikes = 0;
            self.out();
            return Some(Result::Strikeout);
        }
        return None
    }

    pub fn ball(&mut self) -> Option<Result> {
        self.balls += 1;
        if self.balls >= 4 {
            self.balls = 0;
            self.advance_all(1);
            return Some(Result::Walk);
        }
        return None;
    }

    pub fn advance_all(&mut self, inc: i32) {
        //self.loaded_bases
    }

    pub fn out(&mut self) {
        self.out += 1;
        if self.out >= 3 {
            self.out = 0;
            self.inning += 1;
        }
    }
}

#[derive(Clone, Copy)]
pub struct Batter {
    pub avg: f32,
    pub total_ABS: i32,
    pub at_bat: i8,
    pub running_speed: f32,
    pub handedness: Handedness,
    pub prev_at_bat: Option<Pitch>
}

impl Batter {
    pub fn recieve(&self, gamestate: &mut GameState, pitch: &mut Pitch) {
        pitch.call = Some(Call::Strike);//Some(Call::rand());

        
    } 
}

#[derive(Clone)]
pub struct Pitcher {
    pub total_pitches_thrown: i32,
    pub total_strikes_thrown: i32,
    pub total_balls_thrown: i32,
    pub pitch_types_thrown: HashMap<PitchType, i32>,
    pub handedness: Handedness,
    pub avg_velocity: f32,
}

impl Pitcher {
    pub fn new(handedness: Handedness) -> Self {
        Pitcher {
            total_pitches_thrown: 0,
            total_strikes_thrown: 0,
            total_balls_thrown: 0,
            pitch_types_thrown: HashMap::new(),
            handedness,
            avg_velocity: 0.0,
        }
    }
}

#[derive(Clone)]
pub struct Inputs {
    pub pitcher: Pitcher,
    pub batter: Batter,
    //GameState info
    pub gamestate: GameState,
    pub previous_pitch: Option<Pitch>
}

impl Inputs {
    pub fn to_array(&self) -> Array1<f32> {
        //let pitch_types_thrown = self.pitcher.pitch_types_thrown.iter().map(|p| *p.1 as f32).collect();
        let pitch = match self.batter.prev_at_bat {
            None => Array1::from_vec(vec![0.0, 0.0, 0.0, 0.0, 0.0]),
            Some(p) => p.to_array()
        };
        //let pArray: Array1<f32> = Array1::from_vec(pitch_types_thrown);
    
        let mut a: Array1<f32> = ArrayBase::from_vec(vec![
            self.pitcher.total_balls_thrown as f32,
            self.pitcher.total_pitches_thrown as f32,
            self.pitcher.total_strikes_thrown as f32,
            self.pitcher.handedness as i32 as f32,
            self.batter.avg as f32,
            self.batter.at_bat as f32,
            self.batter.total_ABS as f32,
            self.batter.running_speed as f32,
            self.batter.handedness as i32 as f32,
            self.gamestate.inning as f32,
            self.gamestate.score.0 as f32,
            self.gamestate.score.1 as f32,
            self.gamestate.strikes as f32,
            self.gamestate.balls as f32,
            self.gamestate.out as f32,
            self.gamestate.loaded_bases[0] as i64 as f32,
            self.gamestate.loaded_bases[1] as i64 as f32,
            self.gamestate.loaded_bases[2] as i64 as f32,
        ]);
        //a.append(Axis(0), Array1::from_vec(pitch_types_thrown).view());
        //a.append(Axis(0), pitch.view());
        a
    }
}


use crate::{Call::*, PitchType::*};


use crate::{model::ModelConfig, training::TrainingConfig};
use burn::{
    backend::{Autodiff, Wgpu},
    optim::AdamConfig,
};

fn run() {
    type MyBackend = Wgpu<f32, i32>;
    type MyAutodiffBackend = Autodiff<MyBackend>;

    let device = burn::backend::wgpu::WgpuDevice::default();
    let artifact_dir = "/tmp/guide";
    crate::training::train::<MyAutodiffBackend>(
        artifact_dir,
        TrainingConfig::new(ModelConfig::new(10, 512), AdamConfig::new()),
        device.clone(),
    );
}

fn main() {


    let mut dataset = Vec::new();

    let mut pitcher = Pitcher::new(Handedness::Right);
    let mut batters = Vec::new();

    for i in 0..9 {
        batters.push(
            Batter {
                avg: rand::random_range(0.000..1.000),
                total_ABS: rand::random_range(0..100),
                at_bat: 0,
                running_speed: rand::random_range(21.0..31.0),
                handedness: Handedness::Right,
                prev_at_bat: None,
            }
        )
    }
    let mut previous_pitch: Option<Pitch> = None;
    let mut gamestate = GameState::init();

    for i in 0..1 {
        let mut batter = batters[i % 9].clone();
        batter.at_bat += 1;
        let inputs = Inputs {
            pitcher: pitcher.clone(),
            batter,
            gamestate,
            previous_pitch: previous_pitch.clone(),
        };

        let pitchType = match batter.running_speed {
                v if v >= 25.0 => CF,
                v if v < 25.0 => CB,
                _ => PitchType::FS
        };

        let mut pitch = Pitch {
            pitchType,
            velocity: batter.running_speed * 3.5,
            call: None,
            result: None,
            zone: rand::random_range(1..=17)
        };

        while pitch.result.is_none() {

            batter.recieve(&mut gamestate, &mut pitch);

            match pitch.call.unwrap() {
                Fair => {},
                Strike => {
                    pitch.result = gamestate.strike();
                    pitcher.total_strikes_thrown += 1;
                },
                Foul => {},
                Ball => {pitch.result = gamestate.ball()}
            }


            previous_pitch = Some(pitch);

            //println!("{}: Pitch: {}, Result: {}", i, pitch.velocity, pitch.result.is_some());

            dataset.push((inputs.to_array(), pitch.to_array()));
            break;
        }
    }

    let (inputs, pitches): (Vec<_>, Vec<_>) = dataset.into_iter().unzip();
    let inputs =inputs.iter().map(|i| Tensor::from_array(i.clone()).unwrap()).collect::<Vec<_>>();
    let pitches =pitches.iter().map(|p| Tensor::from_array(p.clone()).unwrap()).collect::<Vec<_>>();
    

    //network.forward(&t1).unwrap();
    println!("Network Trained");
}
