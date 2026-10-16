use std::{fmt::Debug};


#[derive(Clone)]
pub struct Matrix<const N: usize>{
    dimensions: usize, 
    pub elements: Vec<f32>,
}

impl<const N: usize> Debug for Matrix<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //let mut it = Vec::new();
        for row in 0..self.dimensions {
            f.debug_set().entries(self.row(row));
        }
        f.debug_list().finish()
    }
}



impl<const N: usize> Matrix<N> {
    pub fn from(elements: Vec<Vec<f32>>) -> Matrix<N> {
        if elements[0].len() != elements.len() {
            panic!("Given matrix is not square")
        }
        Matrix {dimensions: N, elements: elements.into_iter().flatten().collect()}
    }


    pub fn get_dim(&self) -> usize {
        self.dimensions
    }

    pub fn column(&self, index: usize) -> Vec<f32> {
        if index >= self.dimensions {
            panic!("Column index out of bounds");
        }
        self.elements.iter()
            .skip(index)
            .step_by(self.dimensions)
            .copied().collect()
    }

    pub fn column_mut(&mut self, index: usize) -> Vec<&mut f32> {
        if index >= self.dimensions {
            panic!("Column index out of bounds");
        }
        self.elements.iter_mut()
            .skip(index)
            .step_by(self.dimensions)
            .collect()
    }

    pub fn row(&self, index: usize) -> &[f32] {
        if index >= self.dimensions {
            panic!("Row index out of bounds");
        }

        self.elements
            .get(self.dimensions as usize * index..(index + 1) * self.dimensions as usize)
            .unwrap()
    }

    pub fn get_cell(&self, col: usize, row: usize) -> f32{
        let index = row * self.dimensions + col;
        return self.elements[index];
    }

    pub fn set_cell(&mut self, col: usize, row: usize, val: f32) {
        let index = row * self.dimensions + col;
        self.elements[index] = val;
    }

    pub fn row_mut(&mut self, index: usize) -> &mut[f32] {
        if index >= self.dimensions {
            panic!("Row index out of bounds")
        }
        self.elements
            .get_mut(self.dimensions as usize * index..(index + 1) * self.dimensions as usize)
            .unwrap()
    }

    pub fn identity() -> Matrix<N> {
        let mut elements: Vec<Vec<f32>> = Vec::new();
        for i in 0..N {
            let mut row = Vec::new();
            for j in 0..N {
                row.push((i == j) as u8 as f32);
            }
            elements.push(row);
        }
        
        Matrix::from(elements)
    }

    
    pub fn inv_lerp(v1: [f32; N], v2: [f32; N], t: f32) -> Vec<f32>{
        let mut res = Vec::new();
        for i in 0..N {
            let l = (t - v1[i]) / (v2[i] - v1[i]);
            res.push(l);
        }
        res
    }


    pub fn lerp(v1: [f32; N], v2: [f32; N], t: f32) -> [f32; N]{
        let mut res = Vec::new();
        for i in 0..N {
            let l = v1[i] + (v2[i] - v1[i]) * t;
            res.push(l);
        }
        *res.as_array::<N>().unwrap()
    }

    pub fn create_projection_persp(aspect: f32, fov: f32, far: f32, near: f32) -> Matrix<N> {
        let a1 = 1.0 / (aspect * (fov/2.0).tan());
        let b2 = 1.0 / (fov/2.0).tan();
        let c3 = -far/(near - far);
        let c4 = -(far * near)/(near - far);

        Matrix::from(vec![
            vec![a1, 0.0, 0.0, 0.0],
            vec![0.0, b2, 0.0, 0.0],
            vec![0.0, 0.0, c3, c4],
            vec![0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn create_projection_ortho(left: f32, right: f32, top: f32, bottom: f32, far: f32, near: f32) -> Matrix<N>{
        let a1 = 2.0 / (right - left);
        let b2 = 2.0 / (top - bottom);
        let c3 = 1.0 / (near - far);
        let d1 = -(right + left)/(right - left);
        let d2 = -(top + bottom)/(top - bottom);
        let d3 = -(far + near)/(far - near);

        Matrix::from(vec![
            vec![a1, 0.0, 0.0, d1],
            vec![0.0, b2, 0.0, d2],
            vec![0.0, 0.0, c3, d3],
            vec![0.0, 0.0, 0.0 ,1.0],
        ])
    }

    pub fn mul(&self, other: &Matrix<N>) -> Result<Matrix<N>, &'static str> {
        if self.dimensions != other.dimensions {
            return Err("Mismatched Dimensions")
        }
        let mut elements: Vec<f32> = Vec::new();
        for n in 0..self.dimensions {
            let row_A = self.row(n);
            for m in 0..self.dimensions {
                let col_B: Vec<f32> = other.column(m);
                let sum: f32 = row_A.iter().zip(col_B.iter()).map(|(a, b)| a * b).sum();
                elements.push(sum);
            }
        }
        Ok(Matrix{dimensions: self.dimensions, elements})
    }

    pub fn mul_v<const M: usize>(&self, vec: &[f32]) -> Result<[f32; M], &'static str> {
        if self.dimensions < vec.len() {
            return Err("Mismatched Dimensions");
        }
        let mut vec = Vec::from(vec);
        if self.dimensions > vec.len() {
            vec.append([1.0].repeat(self.dimensions - vec.len()).as_mut());
        }
        let mut elements: Vec<f32> = Vec::new();
        for d in 0..self.dimensions {
            let row_A = self.row(d);
            let sum: f32 = row_A.iter().zip(vec.iter()).map(|(a, b)| a * b).sum();
            elements.push(sum);
        }
        Ok(*elements[0..M].as_array::<M>().unwrap())
    }

    pub fn rotate_x(theta: f32) -> Matrix<4> {
        Matrix::from(vec![
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, theta.cos(), -theta.sin(), 0.0],
            vec![0.0, theta.sin(), theta.cos(), 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotate_y(theta: f32) -> Matrix<4> {
        Matrix::from(vec![
            vec![theta.cos(),  0.0, theta.sin(), 0.0],
            vec![0.0,          1.0, 0.0,         0.0],
            vec![-theta.sin(), 0.0, theta.cos(), 0.0],
            vec![0.0,          0.0, 0.0,         1.0],
        ])
    }

    pub fn rotate_z(theta: f32) -> Matrix<4> {
        Matrix::from(vec![
            vec![theta.cos(), -theta.sin(), 0.0, 0.0],
            vec![theta.sin(), theta.cos(), 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ])
    }


    pub fn create_translation(x: f32, y: f32, z: f32) -> Matrix<4>{
        Matrix::from(vec![
            vec![1.0, 0.0, 0.0, x],
            vec![0.0, 1.0, 0.0, y],
            vec![0.0, 0.0, 1.0, z],
            vec![0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn create_rotation(x: f32, y: f32, z: f32) -> Matrix<4> {
        //Matrix::<4>::rotate_z(z).mul(&Matrix::<4>::rotate_y(y).mul(&Matrix::<4>::rotate_x(x)).unwrap()).unwrap()
        Matrix::<4>::rotate_z(z).mul(&Matrix::<4>::rotate_y(y).mul(&Matrix::<4>::rotate_x(x)).unwrap()).unwrap()
    }

    pub fn rotate_around(x: f32, y: f32, z: f32, axis: [f32;3]) -> Matrix<4> {
        let t: Matrix<4> = Matrix::<4>::create_translation(axis[0], axis[1], axis[2]);
        let r: Matrix<4> = Matrix::<4>::rotate_z(z).mul(&Matrix::<4>::rotate_y(y).mul(&Matrix::<4>::rotate_x(x)).unwrap()).unwrap();
        let invt: Matrix<4> = Matrix::<4>::create_translation(-axis[0], -axis[1], -axis[2]);
        invt.mul(&r.mul(&t).unwrap()).unwrap()
    }

    pub fn create_scale(x: f32, y: f32, z:f32) -> Matrix<4> {
        Matrix::from(vec![
            vec![x, 0.0, 0.0, 0.0],
            vec![0.0, y, 0.0, 0.0],
            vec![0.0, 0.0, z, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn create_world(translation: Matrix<4>, rotation: Matrix<4>, scale: Matrix<4>) -> Matrix<4> {
        translation.mul(&rotation.mul(&scale).unwrap()).unwrap()
        //scale.mul(&rotation.mul(&translation).unwrap()).unwrap()
    }

    // pub fn create_look_at(eye: [f32; N], target: [f32; N], up: [f32; N]) -> Matrix<N> {
        
    // }

    pub fn transpose(&self) -> Matrix<N> {
        let mut m = Vec::new();
        for n in 0..self.dimensions {
            let r = self.column(n);
            m.push(r);
        }
        Matrix::from(m)
    }

}

///Subtracts v2 from v1 element-wise
pub fn sub<const N: usize>(v1: &[f32;N], v2: &[f32;N]) -> [f32; N] {
    let mut s: [f32; N] = [0.0; N];
    for i in 0..N {
        s[i] = v1[i] - v2[i];
    }
    *s.as_array::<N>().unwrap()
}

///Negates each element of v
pub fn neg<const N: usize>(v: &[f32; N]) -> [f32; N] {
    v.map(|e| -e)
}

pub fn abs<const N: usize>(v: &[f32; N]) -> [f32; N] {
    v.map(|e| e.abs())
}

///Adds v1 to v2 element-wise
pub fn add<const N: usize>(v1: &[f32; N], v2: &[f32; N]) -> [f32; N] {
    let mut s: [f32; N] = [0.0; N];
    for i in 0..N {
        s[i] = v1[i] + v2[i];
    }
    *s.as_array::<N>().unwrap()
}

///Scale each component of a vector by a factor of s.
pub fn scale<const N: usize>(v: &[f32; N], s: f32) -> [f32; N] {
    let mut r: [f32; N] = [0.0; N];
    for i in 0..N {
        r[i] = v[i] * s;
    }
    *r.as_array::<N>().unwrap()
}

pub fn cross(v1: &[f32;3], v2: &[f32;3]) -> [f32; 3] {
    [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ]
}

///Normalizes the vector. Returns vector unchanged if magnitude is zero.
pub fn normalize<const N: usize>(v1: &[f32; N]) -> [f32; N] {
    let mag = mag::<N>(v1);
    if mag != 0.0 {
        v1.map(|v| v / mag)
    }
    else {
        *v1
    }
}

pub fn dot<const N: usize>(v1: &[f32; N], v2: &[f32; N]) -> f32 {
    let a = normalize(v1);
    let b = normalize(v2);
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

///Calculate square distance between two vectors
pub fn dist<const T: usize>(v1: &[f32], v2: &[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..T {
        sum += (v1[i] - v2[i]).powi(2);
    }
    sum.sqrt()
}

///Calculate magnitude of vector (distance from zero vector)
pub fn mag<const T: usize>(v: &[f32]) -> f32 {
    dist::<T>(v, &[0.0; T])
}

pub fn max<const T: usize>(v: &[f32; T]) -> f32 {
    let mut max = f32::MIN;
    let _ = v.map(|e| max = e.max(max));
    max
}

pub fn min<const T: usize>(v: &[f32; T]) -> f32 {
    let mut min = f32::MAX;
    let _ = v.map(|e| min = e.min(min));
    min
}
