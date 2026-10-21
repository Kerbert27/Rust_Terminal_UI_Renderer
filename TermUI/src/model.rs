use std::{fmt::Error, fs::{self, File}, io::{BufRead, BufReader, Write}, path::Path};

use image::{Rgb, Pixel};
use imageproc::integral_image::ArrayData;

use super::*;

#[derive(Clone, Copy)]
pub struct VertexData {
    pub position: [f32; 4],
    pub color: Rgb<f32>,
    pub normal: Option<[f32; 3]>,
    pub texCoord: Option<[f32; 2]>
}

impl VertexData {
    pub fn new_p(position: [f32; 3]) -> Self {
        VertexData { position: [position[0], position[1], position[2], 1.0], color: Rgb([255.0, 255.0, 255.0]), normal: None, texCoord: None }
    }

    pub fn new_pc(position: [f32; 3], color: Rgb<f32>) -> Self {
        VertexData { position: [position[0], position[1], position[2], 1.0], color, normal: None, texCoord: None }
    }

    pub fn new_pcn(position: [f32; 3], color: Rgb<f32>, normal: [f32; 3]) -> Self {
        VertexData { position: [position[0], position[1], position[2], 1.0], color, normal: Some(normal), texCoord: None }
    }

    pub fn new_pcnt(position: [f32; 3], color: Rgb<f32>, normal: [f32; 3], texCoord: [f32; 2]) -> Self {
        VertexData { position: [position[0], position[1], position[2], 1.0], color, normal: Some(normal), texCoord: Some(texCoord) }
    }

    pub fn new_pct(position: [f32; 3], color: Rgb<f32>, texCoord: [f32; 2]) -> Self {
        VertexData { position: [position[0], position[1], position[2], 1.0], color, normal: None, texCoord: Some(texCoord) }
    }
}

#[derive(Clone)]
pub struct Model {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<usize>,
    pub transform: Transform,
    pub raster_mode: RasterMode,
    view: Matrix<4>,
    projection: Matrix<4>,
    modelViewProjection: Matrix<4>
}


impl Model {
    ///Creates new model from given vertices and indices for triangulation. 
    pub fn new(vertices: Vec<VertexData>, indices: Vec<usize>, transform: Transform, view: Matrix<4>, projection: Matrix<4>, raster_mode: RasterMode) -> Model {

        let modelViewProjection: Matrix<4> = projection.mul(&view.mul(&transform.get_world()).unwrap()).unwrap();

        Model {vertices, indices, transform, raster_mode, view, projection, modelViewProjection}
    }

    ///Create model from .obj file path.
    pub fn load_from_obj(file_path: &str, transform: Transform, view: Matrix<4>, projection: Matrix<4>, raster_mode: RasterMode) -> Self {
        let contents = File::open(file_path).unwrap();
        let reader = BufReader::new(contents);
        let mut verts = Vec::new();
        let mut indices = Vec::new();
        let mut norms = Vec::new();
        for line in reader.lines() {
            let line = line.unwrap();
            if line.is_empty(){continue;}
            let chunks: Vec<&str> = line.split_ascii_whitespace().collect();
            println!("{}", line);

            match chunks[0] {
                "v" => {
                    let x: f32 = chunks[1].parse().unwrap();
                    let y: f32 = chunks[2].parse().unwrap();
                    let z: f32 = chunks[3].parse().unwrap();
                    verts.push(([x, y, z], Rgb([255.0, 255.0, 255.0])));
                },
                "f" => {
                    let i1: usize = chunks[1].parse().unwrap();
                    let i2: usize = chunks[2].parse().unwrap();
                    let i3: usize = chunks[3].parse().unwrap();
                    indices.push(i1 - 1);
                    indices.push(i2 - 1);
                    indices.push(i3 - 1);

                },
                "vn" => {
                    let x: f32 = chunks[1].parse().unwrap();
                    let y: f32 = chunks[2].parse().unwrap();
                    let z: f32 = chunks[3].parse().unwrap();
                    norms.push([x, y, z]);
                },
                "#" => continue,
                _ => continue
            }
        }

        let verts = if !norms.is_empty() {
            verts.iter().zip(norms).map(|v| VertexData::new_pcn(v.0.0, v.0.1, v.1)).collect()
        } else {
            verts.iter().map(|v| VertexData::new_pc(v.0, v.1)).collect()
        };

        Model::new(verts, indices, transform, view, projection, raster_mode)
    }

    pub fn save_to_obj(&self, file_path: &str) -> Result<bool, Error>{
        let mut file = File::create(Path::new(file_path)).unwrap();

        file.write("\n# Verticies\n".as_bytes()).unwrap();
        file.write_all(self.vertices.iter().map(|v| format!("v {} {} {}", v.position[0], v.position[1], v.position[2]).to_string()).collect::<Vec<String>>().join("\n").as_bytes()).unwrap();
        file.write("\n# Faces\n".as_bytes()).unwrap();
        file.write_all(self.indices.chunks(3).map(|f| format!("f {} {} {}", f[0], f[1], f[2]).to_string()).collect::<Vec<String>>().join("\n").as_bytes()).unwrap();
        if self.vertices[0].normal.is_some() {
            file.write("\n# Normals\n".as_bytes()).unwrap();
            file.write_all(self.vertices.iter().map(|v| format!("vn {} {} {}", v.normal.unwrap()[0], v.normal.unwrap()[1], v.normal.unwrap()[2]).to_string()).collect::<Vec<String>>().join("\n").as_bytes()).unwrap();
        }

        Ok(true)
    }

    pub fn point(position: [f32; 3], color: Rgb<f32>, view: Matrix<4>, projection: Matrix<4>) -> Self {
        let transform = Transform {position, scale: [0.0, 0.0, 0.0], rotation: [0.0, 0.0, 0.0]};

        //let world= transform.get_world();

        Self::new(vec![VertexData::new_pc(position, color)], vec![0], transform, view, projection, RasterMode::AsPoints)
    }

    ///Appends other's vertices and indices into self, leaving other empty. Does not change transform or raster mode of self.
    pub fn combine(&mut self, other: &mut Model) {
        let _ = other.indices.iter_mut().map(|i| *i += self.vertices.len());
        self.vertices.append(&mut other.vertices);
        self.indices.append(&mut other.indices);
    }

    

    pub fn update(&mut self, camera: &mut Camera) {

        //FPS VIEW
        // camera.transform.rotation = [camera.view_angles.1, camera.view_angles.0, 0.0];
        // let target = add(&camera.transform.position, camera.transform.forward()[0..3].as_array::<3>().unwrap());
        // let eye = camera.transform.position;
        // let up = Matrix::<4>::create_rotation(camera.view_angles.1, camera.view_angles.0, 0.0).mul_v(&[0.0, 1.0, 0.0]).unwrap();
        
        //CORRECT ORBIT AROUND TARGET
        camera.transform.rotation = [camera.view_angles.1, camera.view_angles.0, 0.0];
        let target = [0.0, 0.0, 0.0];//camera.target;//*camera.transform.forward()[0..3].as_array::<3>().unwrap();
        let eye = Matrix::<4>::rotate_around(camera.view_angles.1, camera.view_angles.0, 0.0, target).mul_v(&camera.transform.position).unwrap();
        let up = *camera.transform.up()[0..3].as_array::<3>().unwrap();

        camera.view = Matrix::<4>::create_look_at(eye, target, up);

        //dbg!(camera.target, self.transform.position);

        self.modelViewProjection = self.projection.mul(&camera.view.mul(&self.transform.get_world()).unwrap()).unwrap();

    }

    ///Sets each vertex into pixel buffer as a single point.
    pub fn rasterize_as_points(&mut self, pixel_buffer: &mut Vec<Option<Fragment>>, clipBox: &Rectangle) {
        let mut ws = Vec::new();
        for vertex in self.vertices.iter() {
            let [mut lx, mut ly, mut lz, w] = self.modelViewProjection.mul_v(&vertex.position).unwrap();
            ws.push((lx, ly, lz, w));
            lx /= w;
            ly /= w;
            lz /= w;

            lx += clipBox.width as f32 / 4.0;
            ly += clipBox.height as f32 / 4.0;
            

            if lx < clipBox.x as f32 || lx > clipBox.width as f32 || ly < clipBox.y as f32 || ly > clipBox.height as f32 {continue;}
            let bufferWidth: usize = (clipBox.width - clipBox.x).into();
            let index: usize = ly as usize * bufferWidth + lx as usize;
            pixel_buffer[index] = if pixel_buffer[index].is_some() && pixel_buffer[index].as_mut().unwrap().position[2] > lz {
                pixel_buffer[index].clone()
            } else {
                let [r, g, b] = vertex.color.data();
                Some(Fragment {color: vertex.color, position: [lx as f32, ly as f32, lz], ascii: String::from("XX"), normal: normalize(&[r, g, b])})
            }
        }
    }

    ///Rasterize each face into pixel buffer as triangle polygon. 
    pub fn rasterize_as_triangles(&mut self, pixel_buffer: &mut Vec<Option<Fragment>>, clipBox: Rectangle){
        for verts in self.indices.chunks_exact(3) {
            //local positions of each vertex of triangle
            let vA = self.vertices[verts[0]];
            let vB = self.vertices[verts[1]];
            let vC = self.vertices[verts[2]];

            
            //color of each vertex
            let colA: Rgb<f32> = vA.color;
            let colB: Rgb<f32> = vB.color;
            let colC: Rgb<f32> = vC.color;
            
            //convert to screen-space
            let clipA = self.modelViewProjection.mul_v::<4>(&vA.position).unwrap();
            let clipB = self.modelViewProjection.mul_v::<4>(&vB.position).unwrap();
            let clipC = self.modelViewProjection.mul_v::<4>(&vC.position).unwrap();

            if clipA[3] < 0.1 || clipB[3] < 0.1 || clipC[3] < 0.1 { continue; }

            let to_screen = |c: &[f32; 4]| -> [f32; 4] {
                let inv_w = 1.0 / c[3];
                let sx = (c[0] * inv_w * 0.5 + 0.5) * 0.5 * clipBox.width as f32;
                let sy = (1.0 - (c[1] * inv_w * 0.5 + 0.5)) * 0.5 * clipBox.height as f32 + 10.0; // y down
                [sx, sy, c[2] * inv_w, inv_w]
            };

            let projA = to_screen(&clipA);
            let projB = to_screen(&clipB);
            let projC = to_screen(&clipC);


            //calculate normals
            let u = &sub( &vA.position, &vB.position)[0..3];
            let v = &sub(&vC.position, &vB.position)[0..3];
            let mut norm = normalize(&cross(&u.as_array::<3>().unwrap(), &v.as_array::<3>().unwrap()));

            let minX: usize = projA[0].min(projB[0].min(projC[0])).max(0.0) as usize;
            let minY: usize = projA[1].min(projB[1].min(projC[1])).max(0.0) as usize;
            let maxX: usize = projA[0].max(projB[0].max(projC[0])).min(clipBox.width as f32) as usize;
            let maxY: usize = projA[1].max(projB[1].max(projC[1])).min(clipBox.height as f32) as usize;
            let den: f32 = (projB[1] - projC[1]) * (projA[0] - projC[0]) + (projC[0] - projB[0]) * (projA[1] - projC[1]);
            if den.abs() <= 0.001 {continue;}
            for x in minX..=maxX {
                for y in minY..=maxY {
                    if x < clipBox.x as usize || x >= clipBox.width as usize|| y < clipBox.y as usize || y >= clipBox.height as usize {continue;}
                    let w0: f32 = ((projB[1] - projC[1]) * (x as f32 - projC[0]) + (projC[0] - projB[0]) * (y as f32 - projC[1])) / den;
                    let w1: f32 = ((projC[1] - projA[1]) * (x as f32 - projC[0]) + (projA[0] - projC[0]) * (y as f32 - projC[1])) / den;
                    let w2: f32 = 1.0 - w0 - w1;

                    if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {continue;}

                    let z: f32 = w0 * projA[2] + w1 * projB[2] + w2 * projC[2];

                    if z < 0.0 || z > 1.0 { continue; }
                    
                    let col = colA.map2(&colB, |cA, cB| cA * w0 + cB * w1).map2(&colC, |cAB, cC| cAB + cC * w2);
                    
                    if vA.normal.is_some() && vB.normal.is_some() && vC.normal.is_some() {
                        norm = Matrix::lerp(Matrix::lerp(vA.normal.unwrap(), vC.normal.unwrap(), w1),Matrix::lerp(vC.normal.unwrap(), vB.normal.unwrap(), w0),  w2)   
                    }
                    let bufferWidth: usize = (clipBox.width - clipBox.x).into();
                    let index: usize = y as usize * bufferWidth + x as usize;
                    let closer = match &pixel_buffer[index] {
                        Some(f) => z < f.position[2],
                        None => true,
                    };
                    if closer {
                        pixel_buffer[index] = Some(Fragment {color: col, position: [x as f32, y as f32, z], ascii: String::from("XX"), normal: norm})
                    }
                }
            }
        }
    }

    ///Rasterize each vertex into pixel buffer as wireframe mesh, with adjacent points in index buffer being defined as edges.
    pub fn rasterize_as_lines(&mut self, pixel_buffer: &mut Vec<Option<Fragment>>, clipBox: &Rectangle) {
        let mut zs = Vec::new();
        for mut v in self.indices.chunks(3) {
            let v: &[usize; 3] = v.as_array().unwrap();
            let verts= self.vertices.get_disjoint_mut(*v).unwrap();
            let verts = [(&verts[0], &verts[1]), (&verts[1], &verts[2]), (&verts[0], &verts[2])];
            for (v0, v1) in verts.iter(){
                let pixelPos:[f32; 4]  = self.modelViewProjection.mul_v(&v0.position).unwrap();
                let nextPixel:[f32; 4]  = self.modelViewProjection.mul_v(&v1.position).unwrap();
                let dist: f32 = dist::<2>(&pixelPos[0..2], &nextPixel[0..2]);

                let depth: f32 = (v0.position[2] - v1.position[2]).abs();

                let mut vs: Vec<(f32, f32, f32, f32)> = Vec::new();
                for t in 0..=dist as usize {
                    let [mut lx, mut ly, mut lz, w] = Matrix::lerp(pixelPos, nextPixel, t as f32 / dist);
                    let c = Matrix::lerp(v0.color.0, v1.color.0, t as f32 / dist);
                    // let mut lx: f32 = pixelPos[0] + (nextPixel[0] - pixelPos[0]) * (t as f32/dist);
                    // let mut ly: f32 = pixelPos[1] + (nextPixel[1] - pixelPos[1]) * (t as f32/dist);
                    // let mut lz: f32 = pixelPos[2] + (nextPixel[2] - pixelPos[2]) * (t as f32/dist);
                    //let w: f32 = pixelPos[3] + (nextPixel[3] - pixelPos[3]) * (t as f32/dist);
                    lx /= w;
                    ly /= w;
                    lz /= w;
                    
                    vs.push((lx, ly, -1.0/lz, w));
                    if lx < clipBox.x as f32 || lx >= clipBox.width as f32 || ly < clipBox.y as f32 || ly >= clipBox.height as f32 {continue;}

                    let bufferWidth: usize = (clipBox.width - clipBox.x).into();
                    let index: usize = ly as usize * bufferWidth + lx as usize;
                    
                    pixel_buffer[index] = if pixel_buffer[index].is_some() && pixel_buffer[index].as_mut().unwrap().position[2] > lz {
                        pixel_buffer[index].clone()
                    } else {
                        Some(Fragment {color: Rgb(c), position: [lx, ly, lz], ascii: String::from("XX"), normal: [0.0; 3]})
                    }
                }
                zs.push(vs);
            }
    }
}

}