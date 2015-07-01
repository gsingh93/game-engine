use nalgebra::{dot, BaseFloat, Col, Mat4, Vec3, Vec4};

pub struct Camera {
    fov: f32,
    near: f32,
    far: f32,
    aspect_ratio: f32,
    transform: Mat4<f32>,
}

impl Camera {
    pub fn new(pos: Vec3<f32>, aspect_ratio: f32) -> Self {
        let transform = Mat4::new(1., 0., 0., pos.x,
                                  0., 1., 0., pos.y,
                                  0., 0., 1., pos.z,
                                  0., 0., 0., 1.);
        Camera {
            fov: BaseFloat::frac_pi_2(),
            near: 0.1,
            far: 1024.,
            aspect_ratio: aspect_ratio,
            transform: transform,
        }
    }

    pub fn fov(&self) -> f32 {
        self.fov
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
        debug!("Camera fov set to {:?}", fov);
    }

    pub fn set_pos(&mut self, pos: &Vec3<f32>) {
        self.transform.set_col(3, Vec4::new(pos.x, pos.y, pos.z, 1.));
        debug!("Camera position set to {:?}", pos);
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn projection_matrix(&self) -> Mat4<f32> {
        let n = self.near;
        let f = self.far;

        let y = 1. / (self.fov / 2.).tan();
        let x = y / self.aspect_ratio;
        let a = (f + n) / (n - f);
        let b = (2. * f * n) / (n - f);

        Mat4::new(x,  0., 0.,  0.,
                  0., y,  0.,  0.,
                  0., 0., a,   b,
                  0., 0., -1., 0.)
    }

    pub fn view_matrix(&self) -> Mat4<f32> {
        let t = &self.transform;

        // This is the camera position applied after the rotation
        let p = (-dot(&t.col(3), &t.col(0)),
                 -dot(&t.col(3), &t.col(1)),
                 -dot(&t.col(3), &t.col(2)));

        // We take the inverse of the rotational part of the transform matrix
        // Because this is an orthogonal matrix, we can just take the transpose
        Mat4::new(t[(0, 0)], t[(1, 0)], t[(2, 0)], p.0,
                  t[(0, 1)], t[(1, 1)], t[(2, 1)], p.1,
                  t[(0, 2)], t[(1, 2)], t[(2, 2)], p.2,
                  0.,        0.,        0.,        1.)
    }

    pub fn set_abs_rotation(&mut self, pitch: f32, yaw: f32) {
        let pitch_mat = Mat4::new(1., 0.,           0.,          0.,
                                  0., pitch.cos(), -pitch.sin(), 0.,
                                  0., pitch.sin(),  pitch.cos(), 0.,
                                  0., 0.,           0.,          1.);
        let yaw_mat = Mat4::new(yaw.cos(),  0., yaw.sin(), 0.,
                                0.,         1., 0.,        0.,
                                -yaw.sin(), 0., yaw.cos(), 0.,
                                0.,         0., 0.,        1.);

        let trans_row = self.transform.col(3);
        self.transform = pitch_mat * yaw_mat;
        self.transform.set_col(3, trans_row);

        debug!("Transform set to {:?}", self.transform);
    }

    pub fn rotate(&mut self, pitch: f32, yaw: f32) {
        let pitch_mat = Mat4::new(1., 0.,           0.,          0.,
                                  0., pitch.cos(), -pitch.sin(), 0.,
                                  0., pitch.sin(),  pitch.cos(), 0.,
                                  0., 0.,           0.,          1.);
        let yaw_mat = Mat4::new(yaw.cos(),  0., yaw.sin(), 0.,
                                0.,         1., 0.,        0.,
                                -yaw.sin(), 0., yaw.cos(), 0.,
                                0.,         0., 0.,        1.);

        self.transform = self.transform * pitch_mat * yaw_mat;

        debug!("Transform set to {:?}", self.transform);
    }
}
