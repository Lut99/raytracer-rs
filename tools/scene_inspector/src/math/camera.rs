//  CAMERA.rs
//    by Lut99
//
//  Description:
//!   A total copy of [`raytracer::math::Camera`] but now for rasterization
//!   instead or rays ¯\_(ツ)_/¯
//

use raytracer::math::camera::degrees_to_radians;
use wgpu::util::DeviceExt;

use super::{Interval, Mat4, Vec3};


/***** HELPER FUNCTIONS *****/
/// Computes the cotanges of a value.
pub fn cotan(x: f64) -> f64 {
    let x2 = 2.0 * x;
    -x2.sin() / (x2.cos() - 1.0)
}





/***** AUXILLARY *****/
/// A bytemuck-safe version of the [`Camera`]'s
/// [view projection matrix](Camera::build_view_proj_mat4()).
#[derive(Clone, Copy, Debug, bytemuck::NoUninit)]
#[repr(C)]
pub struct CameraViewProjMat4 {
    view_proj: [[f32; 4]; 4],
}





/***** LIBRARY *****/
/// Represents a [`raytracer::math::Camera`] but for rasterization.
pub struct Camera {
    // Properties
    /// Vertical Field-of-View
    vfov:   f64,
    /// Aspect ratio.
    aspect: f64,
    /// The interval on which the Z-axis is rendered.
    z:      Interval,

    // Location
    /// The point where the camera is.
    lookfrom: Vec3,
    /// The point where the camera looks at.
    lookat: Vec3,
    /// A vector pointing to the up of the camera.
    up: Vec3,

    // GPU
    /// A buffer to load the raw CPU data in.
    gpu: Option<(wgpu::BindGroupLayout, wgpu::BindGroup)>,
}

// Constructors
impl Camera {
    /// Creates a new Camera based on the given position vectors.
    ///
    /// # Arguments
    /// - `vfov`: The vertical Field-of-View for the camera.
    /// - `aspect`: The aspect ratio of the camera.
    /// - `z`: Determines what on the Z-axis is rendered, and what is trimmed.
    /// - `lookfrom`: The point where the camera is.
    /// - `lookat`: The point where the camera looks at.
    /// - `up`: A vector pointing to the up of the camera.
    ///
    /// # Returns
    /// A new Camera.
    #[inline]
    pub const fn new(vfov: f64, aspect: f64, z: Interval, lookfrom: Vec3, lookat: Vec3, up: Vec3) -> Self {
        Self { vfov, aspect, z, lookfrom, lookat, up, gpu: None }
    }
}

// Camera
impl Camera {
    /// Builds a view-projection matrix that implemens the full camera.
    ///
    /// Essentially combinese [`Camera::build_view_mat4()`] and [`Camera::build_proj_mat4()`].
    ///
    /// # Returns
    /// A 4D matrix that represents the current cammera.
    pub fn build_view_proj_mat4(&self) -> Mat4 { Mat4::OPENGL_TO_WGPU * self.build_proj_mat4() * self.build_view_mat4() }

    /// Builds a view matrix that moves & rotates the entire scene in accordance with the camera's
    /// position.
    ///
    /// # Returns
    /// A 4D matrix that represents the current camera's position & orientation.
    pub fn build_view_mat4(&self) -> Mat4 {
        let direct = self.lookat - self.lookfrom;
        // From: https://docs.rs/cgmath/latest/src/cgmath/matrix.rs.html#366-378
        let f = direct.unit();
        let s = f.cross(self.up).unit();
        let u = s.cross(f);
        #[cfg_attr(rustfmt, rustfmt_skip)]
        Mat4([
            [ s.x,  s.y,  s.z, -self.lookfrom.dot(s)],
            [ u.x,  u.y,  u.z, -self.lookfrom.dot(u)],
            [-f.x, -f.y, -f.z,  self.lookfrom.dot(f)],
            [ 0.0,  0.0,  0.0,  1.0                 ],
        ])
    }

    /// Builds a projection matrix that flattens the 3D space to 2D.
    ///
    /// # Returns
    /// A 4D matrix that represents the current camera's depth properties.
    pub fn build_proj_mat4(&self) -> Mat4 {
        // From: https://docs.rs/cgmath/latest/src/cgmath/projection.rs.html#109-171
        let angle = degrees_to_radians(self.vfov);
        let f = cotan(angle / 2.0);
        #[cfg_attr(rustfmt, rustfmt_skip)]
        Mat4([
            [f / self.aspect, 0.0,  0.0,                                                           0.0                                                                ],
            [0.0,             f,    0.0,                                                           0.0                                                                ],
            [0.0,             0.0,  (self.z.max() + self.z.min()) / (self.z.min() - self.z.max()), (2.0 * self.z.max() * self.z.min()) / (self.z.min() - self.z.max())],
            [0.0,             0.0, -1.0,                                                           0.0                                                                ],
        ])
    }
}

// GPU
impl Camera {
    /// Loads GPU resources for this camera's matrix.
    ///
    /// The resources are loaded as a uniform buffer, with bind group layouts.
    ///
    /// # Arguments
    /// - `device`: The [`wgpu::Device`] to load on.
    pub fn load_gpu(&mut self, device: &wgpu::Device) {
        // Build the view projection matrix
        // NOTE: We do this transposed because our matrix representation is transposed from what
        // WGPU expects.
        let view_proj = self.build_view_proj_mat4().0;
        let view_proj: [[f32; 4]; 4] = [
            [view_proj[0][0] as f32, view_proj[1][0] as f32, view_proj[2][0] as f32, view_proj[3][0] as f32],
            [view_proj[0][1] as f32, view_proj[1][1] as f32, view_proj[2][1] as f32, view_proj[3][1] as f32],
            [view_proj[0][2] as f32, view_proj[1][2] as f32, view_proj[2][2] as f32, view_proj[3][2] as f32],
            [view_proj[0][3] as f32, view_proj[1][3] as f32, view_proj[2][3] as f32, view_proj[3][3] as f32],
        ];

        // Create the buffer
        let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("camera buffer"),
            contents: bytemuck::cast_slice(std::slice::from_ref(&view_proj)),
            usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
            label:   Some("camera bind group layout"),
        });
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout:  &bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.as_entire_binding() }],
            label:   Some("camera bind group"),
        });

        // Store self
        self.gpu = Some((bgl, bg));
    }



    /// Returns the inner bind group layout.
    ///
    /// # Panics
    /// This function panics if you haven't yet called [`Camera::load_gpu()`].
    #[inline]
    #[track_caller]
    pub const fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        let Some((bgl, _)) = &self.gpu else { panic!("Cannot call Camera::bind_group_layout() before calling Camera::load_gpu()") };
        bgl
    }

    /// Returns the inner bind group.
    ///
    /// # Panics
    /// This function panics if you haven't yet called [`Camera::load_gpu()`].
    #[inline]
    #[track_caller]
    pub const fn bind_group(&self) -> &wgpu::BindGroup {
        let Some((_, bg)) = &self.gpu else { panic!("Cannot call Camera::bind_group() before calling Camera::load_gpu()") };
        bg
    }
}





#[cfg(test)]
mod tests {
    use cgmath::{Matrix4, Point3, Vector3, Vector4};

    use super::*;


    /***** HELPER FUNCTIONS *****/
    #[inline]
    const fn mat4_to_matrix4(mat: Mat4) -> Matrix4<f64> {
        Matrix4 {
            x: Vector4 { x: mat.0[0][0], y: mat.0[1][0], z: mat.0[2][0], w: mat.0[3][0] },
            y: Vector4 { x: mat.0[0][1], y: mat.0[1][1], z: mat.0[2][1], w: mat.0[3][1] },
            z: Vector4 { x: mat.0[0][2], y: mat.0[1][2], z: mat.0[2][2], w: mat.0[3][2] },
            w: Vector4 { x: mat.0[0][3], y: mat.0[1][3], z: mat.0[2][3], w: mat.0[3][3] },
        }
    }

    #[inline]
    const fn vec3_to_point3(vec: Vec3) -> Point3<f64> { Point3 { x: vec.x, y: vec.y, z: vec.z } }

    #[inline]
    const fn vec3_to_vector3(vec: Vec3) -> Vector3<f64> { Vector3 { x: vec.x, y: vec.y, z: vec.z } }



    #[inline]
    fn round_matrix4<const DECIMALS: i32>(mat: &mut Matrix4<f64>) {
        let decimals: f64 = 10.0f64.powi(DECIMALS);
        for y in 0..4 {
            for x in 0..4 {
                mat[y][x] = (decimals * mat[y][x]).round() / decimals;
            }
        }
    }





    /***** TESTS *****/
    #[test]
    fn test_camera_build_view_proj_mat4() {
        let vfov = 45.0;
        let aspect = 637.0 / 984.0;
        let z = Interval::new(0.1, 100.0);
        let lookfrom = Vec3::new(0.0, 1.0, 2.0);
        let lookat = Vec3::new(0.0, 0.0, 0.0);
        let lookup = Vec3::new(0.0, 1.0, 0.0);

        let cam = Camera::new(vfov, aspect, z, lookfrom, lookat, lookup);
        let mut cgm = cgmath::Matrix4::from_cols(
            cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
            cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
            cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
            cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
        ) * cgmath::perspective(cgmath::Deg(vfov), aspect, z.min(), z.max())
            * Matrix4::look_at_rh(vec3_to_point3(lookfrom), vec3_to_point3(lookat), vec3_to_vector3(lookup));

        let mut view_proj = mat4_to_matrix4(cam.build_view_proj_mat4());
        round_matrix4::<10>(&mut view_proj);
        round_matrix4::<10>(&mut cgm);

        assert_eq!(view_proj, cgm);
    }

    #[test]
    fn test_camera_build_view_mat4() {
        let vfov = 45.0;
        let aspect = 16.0 / 9.0;
        let z = Interval::new(0.1, 100.0);
        let lookfrom = Vec3::new(0.0, 1.0, 2.0);
        let lookat = Vec3::new(0.0, 0.0, 0.0);
        let lookup = Vec3::new(0.0, 1.0, 0.0);

        let cam = Camera::new(vfov, aspect, z, lookfrom, lookat, lookup);
        let cgm = Matrix4::look_at_rh(vec3_to_point3(lookfrom), vec3_to_point3(lookat), vec3_to_vector3(lookup));

        assert_eq!(mat4_to_matrix4(cam.build_view_mat4()), cgm);
    }

    #[test]
    fn test_camera_build_proj_mat4() {
        let vfov = 45.0;
        let aspect = 16.0 / 9.0;
        let z = Interval::new(0.1, 100.0);
        let lookfrom = Vec3::new(0.0, 1.0, 2.0);
        let lookat = Vec3::new(0.0, 0.0, 0.0);
        let lookup = Vec3::new(0.0, 1.0, 0.0);

        let cam = Camera::new(vfov, aspect, z, lookfrom, lookat, lookup);
        let mut cgm = cgmath::perspective(cgmath::Deg(vfov), aspect, z.min(), z.max());

        // Funny ~ the Camera seems to compute in a slightly higher significance than cgmath(?)
        // Anyway, the matrices seem correct except for this. Let's test by rounding down the
        // numbers a bit.
        let mut proj = mat4_to_matrix4(cam.build_proj_mat4());
        round_matrix4::<10>(&mut proj);
        round_matrix4::<10>(&mut cgm);

        assert_eq!(proj, cgm);
    }
}
