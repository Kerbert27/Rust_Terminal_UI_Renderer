use std::{fs::{self, File}, io::{BufRead, BufReader}};

use image::{Pixel, Rgb};
use imageproc::integral_image::ArrayData;

use super::*;


#[derive(Clone)]
pub struct Model {
    pub vertices: Vec<([f32;3], Rgb<f32>)>,
    pub indices: Vec<usize>,
    pub transform: Transform,
    pub raster_mode: RasterMode,
    view: Matrix<4>,
    projection: Matrix<4>,
    modelViewProjection: Matrix<4>
}


impl Model {
    pub fn new(vertices: Vec<([f32;3], Rgb<f32>)>, indices: Vec<usize>, transform: Transform, view: Matrix<4>, projection: Matrix<4>, raster_mode: RasterMode) -> Model {

        let modelViewProjection: Matrix<4> = projection.mul(&view.mul(&transform.get_world()).unwrap()).unwrap();

        Model {vertices, indices, transform, raster_mode, view, projection, modelViewProjection}
    }

    pub fn load_from_obj(file_path: &str, transform: Transform, view: Matrix<4>, projection: Matrix<4>, raster_mode: RasterMode) -> Self {
        let contents = File::open(file_path).unwrap();
        let reader = BufReader::new(contents);
        let mut verts = Vec::new();
        let mut indices = Vec::new();
        for line in reader.lines() {
            let line = line.unwrap();
            let chunks: Vec<&str> = line.split(' ').collect();
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
                "#" => continue,
                _ => continue
            }
        }

        Model::new(verts, indices, transform, view, projection, raster_mode)
    }

    pub fn point(position: [f32; 3], color: Rgb<f32>, view: Matrix<4>, projection: Matrix<4>) -> Self {
        let transform = Transform {position, scale: [0.0, 0.0, 0.0], rotation: [0.0, 0.0, 0.0]};

        //let world= transform.get_world();

        Self::new(vec![(position, color)], vec![0], transform, view, projection, RasterMode::AsPoints)
    }

    ///Appends other's vertices and indices into self, leaving other empty. Does not change transform or raster mode of self.
    pub fn combine(&mut self, other: &mut Model) {
        let _ = other.indices.iter_mut().map(|i| *i += self.vertices.len());
        self.vertices.append(&mut other.vertices);
        self.indices.append(&mut other.indices);
    }

    

    pub fn update(&mut self, offset: (f32, f32), camera: &mut Camera) {
        // let axis: [f32; 3] = [-self.transform.scale[0] / 2.0, -self.transform.scale[1] / 2.0, -self.transform.scale[2] / 2.0];
        // let world: Matrix<4> = Matrix::<4>::create_world(
        //     Matrix::<4>::create_translation(self.transform.position[0], self.transform.position[1], self.transform.position[2]),
        //     Matrix::<4>::rotate_around(self.transform.rotation[0], self.transform.rotation[1], self.transform.rotation[2], axis),
        //     Matrix::<4>::create_scale(self.transform.scale[0], self.transform.scale[1], self.transform.scale[2]),
        // );

        self.view = camera.look_at([0.0, 0.0, 0.0], Matrix::<4>::create_rotation(camera.view_angles.1, camera.view_angles.0, 0.0).mul_v(&[0.0, 1.0, 0.0]).unwrap());



        self.modelViewProjection = self.projection.mul(&self.view.mul(&self.transform.get_world()).unwrap()).unwrap();
    }

    pub fn rasterize_as_points(&mut self, pixel_buffer: &mut Vec<Option<Fragment>>, clipBox: &Rectangle) {
        let mut ws = Vec::new();
        for vertex in self.vertices.iter() {
            let [mut lx, mut ly, mut lz, w] = self.modelViewProjection.mul_v(&vertex.0).unwrap();
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
                let [r, g, b] = vertex.1.data();
                Some(Fragment {color: vertex.1, position: [lx as f32, ly as f32, lz], ascii: String::from("XX"), normal: normalize(&[r, g, b])})
            }
        }
    }

    pub fn rasterize_as_triangles(&mut self, mut pixel_buffer: &mut Vec<Option<Fragment>>, clipBox: Rectangle){
        for verts in self.indices.chunks_exact(3) {
            //local positions of each vertex of triangle
            let vA: ([f32; 3], Rgb<f32>) = self.vertices[verts[0]];
            let vB: ([f32; 3], Rgb<f32>) = self.vertices[verts[1]];
            let vC: ([f32; 3], Rgb<f32>) = self.vertices[verts[2]];

            //color of each vertex
            let colA: Rgb<f32> = vA.1;
            let colB: Rgb<f32> = vB.1;
            let colC: Rgb<f32> = vC.1;
            
            //convert to screen-space
            let clipA: Vec<f32> = self.modelViewProjection.mul_v::<4>(&vA.0).unwrap().to_vec();
            let clipB: Vec<f32> = self.modelViewProjection.mul_v::<4>(&vB.0).unwrap().to_vec();
            let clipC: Vec<f32> = self.modelViewProjection.mul_v::<4>(&vC.0).unwrap().to_vec();
            
            //convert to clip-space and transform to center of screen
            let mut projA: Vec<f32> = clipA.iter().map(|c| c / clipA[3].max(0.01)).collect();
            projA[0] += clipBox.width as f32 / 4.0;
            projA[1] += clipBox.height as f32 / 4.0;
            let mut projB: Vec<f32> = clipB.iter().map(|c| c / clipB[3].max(0.01)).collect();
            projB[0] += clipBox.width as f32 / 4.0;
            projB[1] += clipBox.height as f32 / 4.0;
            let mut projC: Vec<f32> = clipC.iter().map(|c| c / clipC[3].max(0.01)).collect();
            projC[0] += clipBox.width as f32 / 4.0;
            projC[1] += clipBox.height as f32 / 4.0;

            //calculate normals
            let u = sub( &vA.0, &vB.0);
            let v = sub(&vC.0, &vB.0);
            let norm = normalize(&cross(&u, &v));

            let minX: usize = projA[0].min(projB[0].min(projC[0])) as usize;
            let minY: usize = projA[1].min(projB[1].min(projC[1])) as usize;
            let maxX: usize = projA[0].max(projB[0].max(projC[0])) as usize;
            let maxY: usize = projA[1].max(projB[1].max(projC[1])) as usize;
            let mut den: f32 = (projB[1] - projC[1]) * (projA[0] - projC[0]) + (projC[0] - projB[0]) * (projA[1] - projC[1]);
            if den == 0.0 {den = 0.01}
            for x in minX..=maxX {
                for y in minY..=maxY {
                    if x < clipBox.x as usize || x >= clipBox.width as usize|| y < clipBox.y as usize || y >= clipBox.height as usize {continue;}
                    let w0: f32 = ((projB[1] - projC[1]) * (x as f32 - projC[0]) + (projC[0] - projB[0]) * (y as f32 - projC[1])) / den;
                    let w1: f32 = ((projC[1] - projA[1]) * (x as f32 - projC[0]) + (projA[0] - projC[0]) * (y as f32 - projC[1])) / den;
                    let w2: f32 = 1.0 - w0 - w1;

                    if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {continue;}

                    let z: f32 = w0 * projA[2] + w1 * projB[2] + w2 * projC[2];

                    
                    let col = colA.map2(&colB, |cA, cB| cA * w0 + cB * w1).map2(&colC, |cAB, cC| cAB + cC * w2);

                    let bufferWidth: usize = (clipBox.width - clipBox.x).into();
                    let index: usize = y as usize * bufferWidth + x as usize;
                    pixel_buffer[index] = if pixel_buffer[index].is_some() && pixel_buffer[index].as_mut().unwrap().position[2] > z {
                        pixel_buffer[index].clone()
                    } else {
                        Some(Fragment {color: col, position: [x as f32, y as f32, z], ascii: String::from("XX"), normal: norm})
                    }
                }
            }
        }
    }

    pub fn rasterize_as_lines(&mut self, pixel_buffer: &mut Vec<Option<Fragment>>, clipBox: &Rectangle) {
        let mut zs = Vec::new();
        for mut v in self.indices.chunks(3) {
            let v: &[usize; 3] = v.as_array().unwrap();
            let verts= self.vertices.get_disjoint_mut(*v).unwrap();
            let verts = [(&verts[0], &verts[1]), (&verts[1], &verts[2]), (&verts[0], &verts[2])];
            for ((pos, color), (next, nextColor)) in verts.iter(){
                let pixelPos:[f32; 4]  = self.modelViewProjection.mul_v(pos).unwrap();
                let nextPixel:[f32; 4]  = self.modelViewProjection.mul_v(next).unwrap();
                let dist: f32 = dist::<2>(&pixelPos[0..2], &nextPixel[0..2]);

                let depth: f32 = (pos[2] - next[2]).abs();

                let mut vs: Vec<(f32, f32, f32, f32)> = Vec::new();
                for t in 0..=dist as usize {
                    let [mut lx, mut ly, mut lz, w] = Matrix::lerp(pixelPos, nextPixel, t as f32 / dist);
                    let c = Matrix::lerp(color.0, nextColor.0, t as f32 / dist);
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