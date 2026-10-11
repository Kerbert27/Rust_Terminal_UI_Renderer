use std::collections::HashMap;
use std::io::Write;
use std::{collections::VecDeque, io::Stdout};

use crossterm::{QueueableCommand, terminal};
use crossterm::cursor::MoveTo;
use crossterm::style::Color;
use crossterm::style::Stylize;

use super::matrix::*;
use super::*;

#[derive(Clone)]
pub struct Variable {
    pub name: &'static str,
    pub range: (f32, f32)
}

#[derive(Clone)]
pub enum GraphType {
    Dot,
    Line,
    Bar,
    Curve,
}

#[derive(Clone)]
pub enum RangeType {
    ///User-defined range
    Static(f32, f32),
    ///Absolute Minimum and Maximum values of all variable ranges. 
    MinMax,
    ///Average of all variable ranges. For single-data graphs, this is equivalent to MinMax
    Average,
    ///Ignores defined ranges of variables and computes MinMax's from data points explicitly.
    DynamicMinMax,
    ///Ignores defined ranges of variables and computes averages from data points explicilty.
    DynamicAverage,
}

#[derive(Clone)]
pub struct GraphData {
    pub x_var: Variable,
    pub y_var: Variable,
    pub xy_pairs: VecDeque<(f32, f32)>,
    pub graphType: GraphType,
    pub color: Color
}

impl GraphData {
    pub fn new(x_var: Variable , y_var: Variable, graphType: GraphType, color: Color) -> GraphData {
        let xy_pairs: VecDeque<(f32, f32)> = VecDeque::new();
        
        GraphData {x_var, y_var, xy_pairs, graphType, color}
    }

    pub fn push_back(&mut self, x: f32, y: f32) {
        self.xy_pairs.push_back((x, y));
    }

    ///Inserts (x, y) into xy_pairs. If x already exists within xy_pairs, updates with y. 
    pub fn insert(&mut self, x: f32, y: f32) {
        if let Some(xy) = self.xy_pairs.iter_mut().find(|(p1, _)| *p1 == x) {
            xy.1 = y;
        } else {
            self.xy_pairs.push_back((x, y));
        }
    }

    pub fn pop_front(&mut self) {
        self.xy_pairs.pop_front();
    }

    ///Sets the value at index of xy_pairs to y. Returns Error if index is out of bounds of xy_pairs
    pub fn set_at(&mut self, index: usize, y: f32)  -> Result<bool, &str>
    {
        if index >= self.xy_pairs.len() {return Err("Index out of bounds")}
        self.xy_pairs[index].1 = y;
        return Ok(true);
    }

    pub fn find_nearest_mut(&mut self, x: f32, thresh: f32) -> Option<&mut (f32, f32)>{
        self.xy_pairs.iter_mut().find(|(_x, _y)| (_x - x).abs() <= thresh)
    }

    pub fn draw(&self, stdout: &mut StdoutBuffer, rect: Rectangle, yRange: (f32, f32), xRange: (f32, f32)) {
        let mut prev = [0.0, 0.0];
        for (i, (x, y)) in self.xy_pairs.iter().enumerate(){
            let xspan = rect.span_x();
            let yspan = rect.span_y();
            //x position as screen position
            let norm_x = Matrix::lerp([xspan[0] as f32], [xspan[1] as f32], i as f32 / (xRange.1 - xRange.0))[0];
            //y position as screen position

            let t = Matrix::inv_lerp([yRange.0], [yRange.1], *y)[0];
            let norm_y = Matrix::lerp([yspan[1] as f32], [yspan[0] as f32], t)[0];

            if i == 0 {
                prev = [norm_x, norm_y]
            }
        
            match self.graphType {
                GraphType::Dot => {
                    //stdout.queue(MoveTo(norm_x as u16, norm_y as u16)).unwrap();
                    stdout.write(norm_x as usize, norm_y as usize, "█".to_string());
                }
                GraphType::Line => {
                    draw_line(stdout, prev, [norm_x, norm_y], "█".with(self.color).to_string());
                }
                GraphType::Bar => {
                    let bar_width = rect.width as f32 / self.x_var.range.1;
                    Rectangle::draw(&Rectangle {
                        x: norm_x as u16 + bar_width as u16,
                        y: norm_y as u16,
                        width: bar_width as u16,
                        height: yspan[1] - norm_y as u16,
                        fill: RectFill::Filled('X'.with(Color::Blue))
                    }, stdout);
                    // draw_line(stdout, prev, [norm_x, norm_y]);
                    // draw_line(stdout, [norm_x, norm_y], [norm_x, yspan[1] as f32]);
                    //draw_line(stdout, [norm_x + 1.0, norm_y], [norm_x + 1.0, yspan[1] as f32]);
                },
                GraphType::Curve => {
                    draw_curve(stdout, prev, [norm_x, norm_y], [norm_x.max(prev[0]), norm_y.max(prev[1])]);
                },
            }

            prev = [norm_x,  norm_y];
        }
    }

}

#[derive(Clone)]
pub struct Graph
{
    pub rect: Rectangle,
    plots: HashMap<&'static str, GraphData>,
    pub rangeTypeX: RangeType,
    pub rangeTypeY: RangeType,
}


impl Graph
{
    pub fn new(rect: Rectangle, rangeTypeX: RangeType, rangeTypeY: RangeType) -> Graph {
        //let xy_pairs: VecDeque<(f32, f32)> = VecDeque::new();
        let plots = HashMap::new();
        
        Graph {rect, plots, rangeTypeX, rangeTypeY}
    }

    pub fn add_plot(&mut self, key: &'static str, plot: GraphData) {
        self.plots.insert(key, plot);
    }



    pub fn update<B, F>(&mut self, f: F) -> B 
    where F: Fn(&mut Graph) -> B {
        f(self)
    }

    fn get_line(vals: [(f64, f64); 2]) -> String {
        let slope_prev = (vals[1].1 - vals[0].1) / (vals[1].0 - vals[0].0);
        //let slope_next =  (vals[2].1 - vals[1].1) / (vals[2].0 - vals[1].0);
        dbg!(slope_prev);
        let line = match (slope_prev) {
            m if m >= 1.0 => String::from("+"),
            m if m <= -1.0 => String::from("-"),
            _ => String::from("=")
        };
        return line;
    }

    pub fn draw(&mut self, stdout: &mut StdoutBuffer) {
        self.rect.draw(stdout);

        let (yMin, yMax) = match self.rangeTypeY {
            RangeType::Average | RangeType::DynamicAverage => {
                let (minSums, maxSums) = self.plots.values().map(|p| p.y_var.range).fold((0.0, 0.0), |mut acc, s| {
                    acc.0 += s.0;
                    acc.1 += s.1;
                    acc
                });
                (minSums / self.plots.len() as f32, maxSums / self.plots.len() as f32)
                },
            RangeType::MinMax  | RangeType::DynamicMinMax=> {
                let max = self.plots.values().max_by(|a, b| a.y_var.range.1.total_cmp(&b.y_var.range.1)).unwrap().y_var.range.1;
                let min = self.plots.values().min_by(|a, b| a.y_var.range.0.total_cmp(&b.y_var.range.0)).unwrap().y_var.range.0;
                (min, max)
            },
            RangeType::Static(min, max) => (min, max)
        };


        //stdout.queue(terminal::SetTitle(format!("YMin: {}, YMax: {}", yMin, yMax))).unwrap();

        let (xMin, xMax) = match self.rangeTypeX {
            RangeType::Average | RangeType::DynamicAverage => {
                let (minSums, maxSums) = self.plots.values().map(|p| p.x_var.range).fold((0.0, 0.0), |mut acc, s| {
                    acc.0 += s.0;
                    acc.1 += s.1;
                    acc
                });
                (minSums / self.plots.len() as f32, maxSums / self.plots.len() as f32)
                },
            RangeType::MinMax  | RangeType::DynamicMinMax=> {
                let max = self.plots.values().max_by(|a, b| a.x_var.range.1.total_cmp(&b.x_var.range.1)).unwrap().x_var.range.1;
                let min = self.plots.values().min_by(|a, b| a.x_var.range.0.total_cmp(&b.x_var.range.0)).unwrap().x_var.range.0;
                (min, max)
            },
            RangeType::Static(min, max) => (min, max)
        };

        
        for y in 0..=self.rect.height {
            if y % 2 == 0 {
                stdout.write_all(self.rect.x as usize + 1, (self.rect.y + y) as usize, "-".with(Color::DarkGrey).to_string().repeat(self.rect.width as usize - 2));
                
                let y_label = Matrix::lerp([yMax], [yMin], y as f32 / self.rect.height as f32)[0];
                draw_at(stdout, self.rect.x as usize - 5, (self.rect.y + y) as usize, format!("{:.3}", y_label));
            }
        }
        
        for plot in self.plots.iter() {
            plot.1.draw(stdout, self.rect, (yMin, yMax), (xMin, xMax));
        }
        

        //draw_at(stdout, self.rect.right(), self.rect.bottom() + 1, &self.xy_pairs.back().unwrap().0.to_string().into_bytes());
        
    }
}

pub struct Timeline {
    pub graph: Graph,
    pub capacity: usize,
}

impl Timeline {
    pub fn new(rect: Rectangle, capacity: usize, rangeTypeX: RangeType, rangeTypeY: RangeType) -> Timeline{
        let graph = Graph::new(
            rect,
            rangeTypeX,
            rangeTypeY
        );

        Timeline {graph, capacity}
    }

    pub fn add_time_plot(&mut self, name: &'static str, y_var: Variable, color: Color) {
        self.graph.add_plot(name, GraphData::new(Variable {name: "Time", range: (0.0, self.capacity as f32)}, y_var, GraphType::Dot, color));
    }

    // pub fn update(&mut self, x: f32, y: f32) {
    //     self.push_step(x, y);
    //     //self.graph.update(|g| )
    // }

    pub fn draw(&mut self, stdout: &mut StdoutBuffer) {
        self.graph.draw(stdout);
    }

    pub fn push_step(&mut self, key: &'static str, x: f32, y: f32) {
        self.graph.plots.get_mut(key).unwrap().push_back(x, y);
        if self.graph.plots.get_mut(key).unwrap().xy_pairs.len() > self.capacity {
            self.graph.plots.get_mut(key).unwrap().pop_front();
        }
    }
}

