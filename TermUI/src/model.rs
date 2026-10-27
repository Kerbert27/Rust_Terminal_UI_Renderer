use std::path::Path;

use image::{Rgb, Pixel};
use imageproc::integral_image::ArrayData;

use super::*;

#[derive(Clone, Copy, Debug)]
pub struct VertexData {
    pub position: [f32; 4],
    pub color: Rgb<f32>,
    pub normal: Option<[f32; 3]>,
    pub texCoord: Option<[f32; 3]>
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

    pub fn new_pcnt(position: [f32; 3], color: Rgb<f32>, normal: [f32; 3], texCoord: [f32; 3]) -> Self {
        VertexData { position: [position[0], position[1], position[2], 1.0], color, normal: Some(normal), texCoord: Some(texCoord) }
    }

    pub fn new_pct(position: [f32; 3], color: Rgb<f32>, texCoord: [f32; 3]) -> Self {
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

    fn parse_face(chunk: &str) -> (usize, Option<usize>, Option<usize>) {
        let indices: Vec<&str> = chunk.split('/').collect();
        match indices.len() {
            1 => return (indices[0].parse().unwrap(), None, None),
            2 => return (indices[0].parse().unwrap(), if indices[1].is_empty() {None} else {Some(indices[1].parse().unwrap())}, None),
            3 => return (indices[0].parse().unwrap(), if indices[1].is_empty() {None} else {Some(indices[1].parse().unwrap())}, Some(indices[2].parse().unwrap())),
            _ => return (0, None, None)
        }
    }

    ///Create model from .obj file path.
    pub fn load_from_obj(file_path: &str, transform: Transform, view: Matrix<4>, projection: Matrix<4>, raster_mode: RasterMode, materials: &mut Vec<String>) -> Result<Self, Error> {
        let contents: Result<File, Error> = File::open(file_path);
        if contents.is_err() {return Err(contents.unwrap_err());}
        let reader: BufReader<File> = BufReader::new(contents.unwrap());
        let mut verts: Vec<VertexData> = Vec::new();
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut textures: Vec<(f32, f32)> = Vec::new();
        let mut indices: Vec<usize> = Vec::new();
        let mut norms: Vec<[f32; 3]> = Vec::new();
        let mut faceData: Vec<((usize, Option<usize>, Option<usize>), (usize, Option<usize>, Option<usize>), (usize, Option<usize>, Option<usize>), String)> = Vec::new();

        let mut currentMat: String = String::new();
        for line in reader.lines() {
            let line: String = line.unwrap();
            if line.is_empty(){continue;}
            let chunks: Vec<&str> = line.split_whitespace().collect();
            match chunks[0] {
                "v" => {
                    let x: f32 = chunks[1].parse().unwrap();
                    let y: f32 = chunks[2].parse().unwrap();
                    let z: f32 = chunks[3].parse().unwrap();
                    positions.push([x, y, z]);
                },
                "f" => {
                    let mut face: Vec<(usize, Option<usize>, Option<usize>)> = Vec::new();
                    for chunk in &chunks[1..chunks.len()] {
                        face.push(Model::parse_face(*chunk));
                    }
                    faceData.push((face[0], face[1], face[2], currentMat.clone()));

                    if face.len() == 4 {
                        faceData.push((face[0], face[2], face[3], currentMat.clone()));
                    }
                },
                "vn" => {
                    let x: f32 = chunks[1].parse().unwrap();
                    let y: f32 = chunks[2].parse().unwrap();
                    let z: f32 = chunks[3].parse().unwrap();
                    norms.push([x, y, z]);
                },
                "vt" => {
                    let x: f32 = chunks[1].parse().unwrap();
                    let y: f32 = chunks[2].parse().unwrap();
                    textures.push((x, y));
                },
                "usemtl" => {
                    currentMat = chunks[1].to_string();
                    materials.push(currentMat.to_string());
                },
                "#" => continue,
                _ => continue
            }
        }

        let mut i = 0;
        //FUCKING MINT PERFECT DONT FUCK UP!!!!!!
        for face in faceData.iter_mut() {
            let (i1, vt1, vn1) = face.0;
            let (i2, vt2, vn2) = face.1;
            let (i3, vt3, vn3) = face.2;
            let tex: String = face.3.clone();

            indices.push(i);
            indices.push(i + 1);
            indices.push(i + 2);
            i += 3;

            let mut vA: VertexData = VertexData::new_p(positions[i1 - 1]);
            let mut vB: VertexData = VertexData::new_p(positions[i2 - 1]);
            let mut vC: VertexData = VertexData::new_p(positions[i3 - 1]);

            if !norms.is_empty() {
                if vn1.is_some() && vn2.is_some() && vn3.is_some() {
                    vA.normal = Some(norms[vn1.unwrap() - 1]);
                    vB.normal = Some(norms[vn2.unwrap() - 1]);
                    vC.normal = Some(norms[vn3.unwrap() - 1]);
                } else {
                    vA.normal = Some(norms[i1 - 1]);
                    vB.normal = Some(norms[i2 - 1]);
                    vC.normal = Some(norms[i3 - 1]);
                }
            }
            if !textures.is_empty() {
                if vt1.is_some() && vt2.is_some() && vt3.is_some() {
                    let index  = materials.iter().position(|m| {
                        *m == tex
                    }).unwrap_or(0);
                    
                    vA.texCoord = Some([textures[vt1.unwrap() - 1].0, textures[vt1.unwrap() - 1].1, index as f32]);
                    vB.texCoord = Some([textures[vt2.unwrap() - 1].0, textures[vt2.unwrap() - 1].1, index as f32]);
                    vC.texCoord = Some([textures[vt3.unwrap() - 1].0, textures[vt3.unwrap() - 1].1, index as f32]);
                }
            }
            verts.push(vA);
            verts.push(vB);
            verts.push(vC);

        }

        Ok(Model::new(verts, indices, transform, view, projection, raster_mode))
    }


    pub fn save_to_obj(&self, file_path: &str) -> Result<bool, Error>{
        let mut file = File::create(Path::new(file_path)).unwrap();

        file.write("\n# Verticies\n".as_bytes()).unwrap();
        file.write_all(self.vertices.iter().map(|v| format!("v {} {} {}", v.position[0], v.position[1], v.position[2]).to_string()).collect::<Vec<String>>().join("\n").as_bytes()).unwrap();
        file.write("\n# Faces\n".as_bytes()).unwrap();
        file.write_all(self.indices.chunks(3).map(|f| format!("f {} {} {}", f[0] + 1, f[1] + 1, f[2] + 1).to_string()).collect::<Vec<String>>().join("\n").as_bytes()).unwrap();
        if self.vertices[0].normal.is_some() {
            file.write("\n# Normals\n".as_bytes()).unwrap();
            file.write_all(self.vertices.iter().map(|v| format!("vn {} {} {}", v.normal.unwrap()[0], v.normal.unwrap()[1], v.normal.unwrap()[2]).to_string()).collect::<Vec<String>>().join("\n").as_bytes()).unwrap();
        }

        Ok(true)
    }

    pub fn point(position: [f32; 3], view: Matrix<4>, projection: Matrix<4>) -> Self {
        let transform = Transform {position, scale: [1.0, 1.0, 1.0], rotation: [0.0, 0.0, 0.0]};


        Self::new(vec![VertexData::new_p([0.0, 0.0, 0.0])], vec![1], transform, view, projection, RasterMode::AsPoints)
    }

    pub fn plane(transform: Transform, view: Matrix<4>, projection: Matrix<4>, cullMode: CullMode) -> Self {
        let f = *transform.forward()[0..3].as_array::<3>().unwrap();
        let r = *transform.right()[0..3].as_array::<3>().unwrap();
        let b = neg(&f);
        let l = neg(&r);
        let v1: VertexData = VertexData::new_p(add(&transform.position, &add::<3>(&b, &r)));
        let v2: VertexData = VertexData::new_p(add(&transform.position, &add::<3>(&f, &l)));
        let v3: VertexData = VertexData::new_p(add(&transform.position, &add::<3>(&f, &r)));
        let v4: VertexData = VertexData::new_p(add(&transform.position, &add::<3>(&b, &l)));

        let indices: Vec<usize> = match cullMode {
            CullMode::CullFront => vec![3, 0, 2, 1, 2, 0],   //counter-clockwise rounding
            CullMode::CullBack => vec![0, 3, 2, 1, 2, 3],   //clock-wise rounding
            CullMode::CullNone => vec![3, 0, 2, 1, 2, 0, 0, 3, 2, 1, 2, 3],
        };

        let vertices = vec!(v1, v2, v3, v4);
        Self::new(vertices, indices, transform, view, projection, RasterMode::AsTriangles)
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
        let mut target = add(&camera.target, &camera.transform.position);
        target[2] = camera.target[2];
        let eye = Matrix::<4>::create_rotation(camera.view_angles.1, camera.view_angles.0, 0.0).mul_v(&camera.transform.position).unwrap();
        let up = *camera.transform.up()[0..3].as_array::<3>().unwrap();

        camera.view = Matrix::<4>::create_look_at(eye, target, up);


        self.modelViewProjection = self.projection.mul(&camera.view.mul(&self.transform.get_world()).unwrap()).unwrap();

        // dbg!(&self.projection, self.modelViewProjection.mul_v::<3>(&[0.0, 0.0, 0.0]).unwrap());
        // panic!();
    }

    ///Sets each vertex into pixel buffer as a single point.
    pub fn rasterize_as_points(&mut self, pixel_buffer: &mut Vec<Option<Fragment>>, clipBox: &Rectangle) {
        for vertex in self.vertices.iter() {
            let clipPos = self.modelViewProjection.mul_v(&vertex.position).unwrap();

            let to_screen = |c: &[f32; 4]| -> [f32; 4] {
                let inv_w = 1.0 / c[3];
                let sx = (c[0] * inv_w * 0.5 + 0.5) * 0.5 * clipBox.width as f32;
                let sy = (1.0 - (c[1] * inv_w * 0.5 + 0.5)) * 0.5 * clipBox.height as f32 + 10.0; // y down
                [sx, sy, c[2] * inv_w, inv_w]
            };

            let projPos = to_screen(&clipPos);

            if projPos[0] < clipBox.x as f32 || projPos[0] > clipBox.width as f32 || projPos[1] < clipBox.y as f32 || projPos[1] > clipBox.height as f32 {continue;}
            let bufferWidth: usize = (clipBox.width - clipBox.x).into();
            let index: usize = projPos[1] as usize * bufferWidth + projPos[0] as usize;
            pixel_buffer[index] = if pixel_buffer[index].is_some() && pixel_buffer[index].as_mut().unwrap().position[2] > projPos[2] {
                pixel_buffer[index].clone()
            } else {
                let [r, g, b] = vertex.color.data();
                Some(Fragment {color: vertex.color, position: [projPos[0] as f32, projPos[1] as f32, projPos[2]], ascii: 'X', normal: normalize(&[r, g, b])})
            }
        }
    }

    ///Rasterize each face into pixel buffer as triangle polygon. 
    pub fn rasterize_as_triangles(&mut self, pixel_buffer: &mut Vec<Option<Fragment>>, clipBox: &Rectangle, textures: &Vec<Texture>){
        let bufferWidth: usize = (clipBox.width - clipBox.x).into();

        for verts in self.indices.as_chunks::<3>().0 {
            //local positions of each vertex of triangle
            let vA: VertexData = self.vertices[verts[0]];
            let vB: VertexData = self.vertices[verts[1]];
            let vC: VertexData = self.vertices[verts[2]];

            //VertexShader

            //convert to screen-space
            let clipA: [f32; 4] = self.modelViewProjection.mul_v::<4>(&vA.position).unwrap();
            let clipB: [f32; 4] = self.modelViewProjection.mul_v::<4>(&vB.position).unwrap();
            let clipC: [f32; 4] = self.modelViewProjection.mul_v::<4>(&vC.position).unwrap();


            let mut norm: [f32; 3] = [0.0, 0.0, 0.0];
            if vA.normal.is_none() || vB.normal.is_none() || vC.normal.is_none() {
                let u = sub::<3>(&vA.position, &vB.position);
                let v = sub::<3>(&vC.position, &vB.position);
                norm = normalize(&cross(&u, &v));
            }

            if clipA[3] < 0.1 || clipB[3] < 0.1 || clipC[3] < 0.1 { continue; }

            let to_screen = |c: &[f32; 4]| -> [f32; 4] {
                let inv_w: f32 = 1.0 / c[3];
                let sx: f32 = (c[0] * inv_w * 0.5 + 0.5) * 0.5 * clipBox.width as f32;
                let sy: f32 = (1.0 - (c[1] * inv_w * 0.5 + 0.5)) * 0.5 * clipBox.height as f32; // y down
                [sx, sy, c[2] * inv_w, inv_w]
            };

            let projA: [f32; 4] = to_screen(&clipA);
            let projB: [f32; 4] = to_screen(&clipB);
            let projC: [f32; 4] = to_screen(&clipC);

            //calculate normals from position if not given in VertexData
            
            let minX: isize = projA[0].min(projB[0].min(projC[0])) as isize;
            let minY: isize = projA[1].min(projB[1].min(projC[1]))as isize;
            let maxX: isize = projA[0].max(projB[0].max(projC[0])) as isize;
            let maxY: isize = projA[1].max(projB[1].max(projC[1])) as isize;

            if minX >= clipBox.width as isize || minY >= clipBox.height as isize || maxX <= clipBox.x as isize || maxY <=clipBox.y as isize {continue;}

            let den: f32 = (projB[1] - projC[1]) * (projA[0] - projC[0]) + (projC[0] - projB[0]) * (projA[1] - projC[1]);
            if den.abs() <= 0.001 {continue;}

            for x in minX..=maxX {
                for y in minY..=maxY {
                    if x < clipBox.x as isize || x >= clipBox.width as isize || y < clipBox.y as isize || y >= clipBox.height as isize {continue;}
                    let w0: f32 = ((projB[1] - projC[1]) * (x as f32 - projC[0]) + (projC[0] - projB[0]) * (y as f32 - projC[1])) / den;
                    let w1: f32 = ((projC[1] - projA[1]) * (x as f32 - projC[0]) + (projA[0] - projC[0]) * (y as f32 - projC[1])) / den;
                    let w2: f32 = 1.0 - w0 - w1;
                    if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {continue;}
                    
                    let z: f32 = w0 * projA[2] + w1 * projB[2] + w2 * projC[2];
                    if z < 0.0 || z > 1.0 { continue; }
                    
                    if let Some(normA) = vA.normal && let Some(normB) = vB.normal && let Some(normC) = vC.normal{
                        let normA: [f32; 3] = scale(&normA, w0);
                        let normB: [f32; 3] = scale(&normB, w1);
                        let normC: [f32; 3] = scale(&normC, w2);
                        norm = add(&normA, &add::<3>(&normB, &normC));
                    }
                    
                    let col: Rgb<f32> = if vA.texCoord.is_some() && vB.texCoord.is_some() && vC.texCoord.is_some() {
                        let index: usize = vC.texCoord.unwrap()[2] as usize;
                        let uv_A: [f32; 3] = scale(&vA.texCoord.unwrap(), w0);
                        let uv_B: [f32; 3] = scale(&vB.texCoord.unwrap(), w1);
                        let uv_C: [f32; 3] = scale(&vC.texCoord.unwrap(), w2);
                        
                        let uv: [f32; 3] = add(&uv_A, &add::<3>(&uv_B, &uv_C));
                        textures.get(index).unwrap_or(&Texture::default()).sample_at(uv[0], 1.0 - uv[1], TextureMode::Tile)
                    } else {
                        vA.color.map2(&vB.color, |cA, cB| cA* w0 + cB * w1).map2(&vC.color, |cAB, cC| cAB+ cC * w2)
                    };
                    
                    
                    let index: usize = y as usize * bufferWidth + x as usize;
                    let closer = match &pixel_buffer[index] {
                        Some(f) => z < f.position[2],
                        None => true,
                    };
                    if closer {
                        pixel_buffer[index] = Some(Fragment {color: col, position: [x as f32, y as f32, z], ascii: 'X', normal: norm})
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
                    
                    if pixel_buffer[index].is_some() && pixel_buffer[index].as_mut().unwrap().position[2] > lz {
                        continue;
                    } else {
                        pixel_buffer[index] = Some(Fragment {color: Rgb(c), position: [lx, ly, lz], ascii: 'X', normal: [0.0; 3]})
                    }
                }
                zs.push(vs);
            }
    }
}

}