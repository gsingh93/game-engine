use nalgebra::{normalize, cross, zero, Vec3, BaseFloat};

pub struct Camera {
    fov: f32,
    near: f32,
    far: f32,
    aspect_ratio: f32,
    pos: Vec3<f32>,
    look_at: Vec3<f32>,
}

impl Camera {
    pub fn new(pos: Vec3<f32>) -> Self {
        Camera { fov: BaseFloat::frac_pi_2(), near: 0.1, far: 1024., aspect_ratio: 4./3.,
                 pos: pos, look_at: zero() }
    }

    pub fn set_pos(&mut self, pos: Vec3<f32>) {
        self.pos = pos;
    }

    pub fn get_projection_matrix(&self) -> [[f32; 4]; 4] {
        let n = self.near;
        let f = self.far;

        let y = 1. / (self.fov / 2.).tan();
        let x = y * self.aspect_ratio;
        let a = (f + n) / (n - f);
        let b = (2. * f * n) / (n - f);

        [[x,  0., 0., 0.],
         [0., y,  0., 0.],
         [0., 0., a, -1.],
         [0., 0., b,  0.]]
    }

    pub fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        // Forward vector
        let w = normalize(&(self.look_at - self.pos));
        let up = Vec3::new(0., 1., 0.);

        // The up vector cannot be parallel (in this case, the same) as the forward vector
        debug_assert!(w != up);

        // Right vector
        let u = normalize(&cross(&w, &up));

        // Up vector (we shouldn't need to normalize as the cross product of two orthogonal unit
        // vectors is a unit vector)
        let v = cross(&u, &w);

        [[u.x, v.x, -w.x, 0.],
         [u.y, v.y, -w.y, 0.],
         [u.z, v.z, -w.z, 0.],
         [-self.pos.x, -self.pos.y, -self.pos.z, 1.]]
    }
}
