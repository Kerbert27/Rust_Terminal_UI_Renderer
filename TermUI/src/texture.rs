use super::*;

pub enum TextureMode {
    ///UV's outside the `[0.0, 1.0]` range will be repeated as if tiled.
    Tile,
    ///UV's outside the `[0.0, 1.0]` range will be mirrored.
    Mirror,
    ///UV's outside the `[0.0, 1.0]` range will be clamped to 0.0 or 1.0
    Clamp,
    ///UV's outside the `[0.0, 1.0]` range will be treated as undefined and return as Magenta.
    Exact,
    ///UV's will be remapped to the specified range.
    Stretch(f32, f32)
    
}


#[derive(Clone)]
pub struct Texture {
    buf: Vec<Vec<Vec<u8>>>,
    pub width: usize,
    pub height: usize
}

impl Default for Texture {
    fn default() -> Self {
        Texture {buf: vec![], width: 0, height: 0}
    }
}

impl Texture {
    ///Create a new texture from a 2D array of [u8] Rgba values
    pub fn new(buf: Vec<Vec<Vec<u8>>>) -> Self {
        let (width, height) = if buf.is_empty() {
            (0, 0)
        } else {
            (buf[0].len(), buf.len())
        };
        Texture {buf: buf.clone(), width, height}
    }

    ///Sample the texture at a given UV and with `TextureMode`
    pub fn sample_at(&self, u: f32, v: f32, texMode: TextureMode) -> Rgb<f32> {
        if self.width == 0 || self.height == 0 {
            return Rgb([255.0, 0.0, 255.0]);
        }

        match texMode {
            TextureMode::Clamp => {
                let x = Matrix::lerp([0.0], [self.width as f32], u.min(1.0).max(0.0))[0] as usize;
                let y = Matrix::lerp([0.0], [self.height as f32], v.min(1.0).max(0.0))[0] as usize;
                return Rgb(self.buf[y][x][0..3].as_array::<3>().unwrap().map(|p| p as f32))
            },
            TextureMode::Tile => {
                let x = Matrix::lerp([0.0], [self.width as f32], u % 1.0)[0] as usize;
                let y = Matrix::lerp([0.0], [self.height as f32], v % 1.0)[0] as usize;
                let col = &self.buf[y][x][0..3];
                return Rgb([col[0] as f32, col[1] as f32, col[2] as f32])
            },
            TextureMode::Mirror => {
                let x = Matrix::lerp([0.0], [self.width as f32], u.abs() % 1.0)[0] as usize;
                let y = Matrix::lerp([0.0], [self.height as f32], v.abs() % 1.0)[0] as usize;
                return Rgb(self.buf[y][x][0..3].as_array::<3>().unwrap().map(|p| p as f32))
            },
            TextureMode::Exact => {
                let x = Matrix::lerp([0.0], [self.width as f32], u)[0] as usize;
                let y = Matrix::lerp([0.0], [self.height as f32], v)[0] as usize;
                if x < self.width && y < self.height {
                    return Rgb(self.buf[y][x][0..3].as_array::<3>().unwrap().map(|p| p as f32))
                } else {
                    return Rgb([255.0, 0.0, 255.0]);
                }
            },
            TextureMode::Stretch(w, h) => {
                let u = Matrix::lerp([0.0], [w], u)[0];
                let v = Matrix::lerp([0.0], [h], v)[0];
                let x = Matrix::lerp([0.0], [self.width as f32], u)[0] as usize;
                let y = Matrix::lerp([0.0], [self.height as f32], v)[0] as usize;
                if x < self.width && y < self.height {
                    return Rgb(self.buf[y][x][0..3].as_array::<3>().unwrap().map(|p| p as f32))
                } else {
                    return Rgb([255.0, 0.0, 255.0]);
                }
            }
        }
    }
}

///Attempts to load textures from target directory from array of file names.
pub fn load_materials(target_dir: &str, textures: &mut Vec<String>) -> Vec<Texture> {
    let mut loaded_textures: Vec<Texture> = Vec::new();
    if textures.is_empty() {
        return vec![Texture::default()]
    }
    for tex in textures {
        let path = format!("{}/{}.png", target_dir, tex);
        let file = File::open(path.clone());
        if file.is_err() {
            loaded_textures.push(Texture::new(Vec::new()));
            println!("{} does not exist. Failed to load Texture", path.clone());
            continue;
        }
        let decoder = Decoder::new(BufReader::new(file.unwrap()));
        let mut reader = decoder.read_info().unwrap();
        // Allocate the output buffer.
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();
        // Grab the bytes of the image.
        //dbg!(&buf[0..4]);
        let bytes = match info.color_type {
            ColorType::Rgba => { buf[..info.buffer_size()].chunks(4).map(|c| [c[0], c[1], c[2], c[3]].to_vec()).collect::<Vec<_>>()},
            ColorType::Rgb => {buf[..info.buffer_size()].chunks(3).map(|c| [c[0], c[1], c[2], 255].to_vec()).collect::<Vec<_>>()}
            ColorType::GrayscaleAlpha => {buf[..info.buffer_size()].chunks(2).map(|c| [c[0], c[0], c[0], c[1]].to_vec()).collect::<Vec<_>>()}
            ColorType::Grayscale => {buf[..info.buffer_size()].chunks(2).map(|c| [c[0], c[0], c[0], 255].to_vec()).collect::<Vec<_>>()}
            ColorType::Indexed => {buf[..info.buffer_size()].chunks(1).map(|c| [c[0]].to_vec()).collect::<Vec<_>>()}
        };
        let texture: Texture = Texture::new(bytes.chunks(info.width as usize).map(|c| c.to_vec()).collect::<Vec<_>>());
        //dbg!(bytes.len(), texture.buf.len(), info.width, info.height, &texture.buf[info.height as usize - 1][info.width as usize - 1]);
        println!("{} loaded successfully.", path.clone());
        loaded_textures.push(texture);
    }

    loaded_textures
    

    //loaded_textures;
}