//  SCENE.rs
//    by Lut99
//
//  Created:
//    23 Apr 2023, 11:40:52
//  Last edited:
//    07 May 2023, 12:43:21
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the scene file.
//

use std::num::{NonZeroU32, NonZeroU64};

use serde::{Deserialize, Serialize};

use super::objects::Object;
use crate::common::file::{impl_toml_from_path, impl_toml_from_string, impl_toml_to_path, impl_toml_to_string};
use crate::math::{Camera, Vec3};


/***** HELPER FUNCTIONS *****/
/// Returns the default dimensions for a [`CameraInfo`].
#[inline]
pub const fn default_camera_info_dims() -> (NonZeroU32, NonZeroU32) {
    // SAFETY: This works because the values are not 0.
    (unsafe { NonZeroU32::new_unchecked(800) }, unsafe { NonZeroU32::new_unchecked(600) })
}

/// Returns the default sample-per-pixel number for a [`CameraInfo`].
#[inline]
pub const fn default_camera_info_n_samples() -> NonZeroU64 {
    // SAFETY: This works because the value is not 0.
    unsafe { NonZeroU64::new_unchecked(100) }
}

/// Returns the default vertical field-of-view (FOV) for a [`CameraInfo`].
#[inline]
pub const fn default_camera_info_vfov() -> f64 { 90.0 }

/// Returns the default defocus angle for a [`CameraInfo`].
#[inline]
pub const fn default_camera_info_defocus_angle() -> f64 { 0.0 }

/// Returns the default focal point distance for a [`CameraInfo`].
#[inline]
pub const fn default_camera_info_focus_dist() -> f64 { 0.0 }

/// Returns the default shutter time for a [`CameraInfo`].
#[inline]
pub const fn default_camera_info_shutter_time() -> NonZeroU64 {
    // SAFETY: This works because the value is not 0.
    unsafe { NonZeroU64::new_unchecked(1) }
}



/// Function checking if something equals its default.
#[inline]
fn is_default<T: Default + PartialEq>(obj: &T) -> bool { obj == &T::default() }





/***** LIBRARY *****/
/// Defines properties of the environment passed to objects and materials.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Environment {
    /// The refraction index of the outer world.
    pub air_refraction_index: f64,
}
impl Default for Environment {
    #[inline]
    fn default() -> Self { Self { air_refraction_index: 1.0 } }
}



/// Defines properties of a [`Camera`]'s position and orientation.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct CameraPos {
    /// The point the camera is looking _from_.
    pub lookfrom: Vec3,
    /// The point the camera is looking _at_.
    pub lookat:   Vec3,
    /// The vector pointing the camera up.
    pub lookup:   Vec3,
}
impl Default for CameraPos {
    #[inline]
    fn default() -> Self { Self { lookfrom: Vec3::new(0.0, 0.0, 0.0), lookat: Vec3::new(0.0, 0.0, -1.0), lookup: Vec3::new(0.0, 1.0, 0.0) } }
}

/// Defines properties of the camera in a scene.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct CameraInfo {
    // Image properties
    /// The dimensions of the camera.
    #[serde(default = "default_camera_info_dims")]
    pub dims: (NonZeroU32, NonZeroU32),

    // Features
    /// The number of rays fired per pixel.
    #[serde(default = "default_camera_info_n_samples")]
    pub n_samples: NonZeroU64,
    /// The vertical field-of-view of the camera.
    #[serde(default = "default_camera_info_vfov")]
    pub vfov: f64,
    /// The vertical field-of-view of the camera.
    #[serde(default = "default_camera_info_defocus_angle")]
    pub defocus_angle: f64,
    /// The vertical field-of-view of the camera.
    #[serde(default = "default_camera_info_focus_dist")]
    pub focus_dist: f64,
    /// The shutter time, in microseconds, of the camera.
    ///
    /// Use `1` to disable it (instant shutter).
    #[serde(default = "default_camera_info_shutter_time")]
    pub shutter_time: NonZeroU64,

    // Position
    /// Defining the position & orientation of the camera.
    #[serde(default)]
    pub pos: CameraPos,
}
impl Default for CameraInfo {
    #[inline]
    fn default() -> Self {
        CameraInfo {
            dims: default_camera_info_dims(),
            n_samples: default_camera_info_n_samples(),
            vfov: default_camera_info_vfov(),
            defocus_angle: default_camera_info_defocus_angle(),
            focus_dist: default_camera_info_focus_dist(),
            shutter_time: default_camera_info_shutter_time(),
            pos: CameraPos::default(),
        }
    }
}
impl From<CameraInfo> for Camera {
    #[inline]
    fn from(value: CameraInfo) -> Self {
        Camera::new(
            (value.dims.0.into(), value.dims.1.into()),
            value.n_samples.into(),
            value.vfov,
            value.defocus_angle,
            value.focus_dist,
            value.shutter_time.into(),
            value.pos.lookfrom,
            value.pos.lookat,
            value.pos.lookup,
        )
    }
}



/// The SceneFile defines the scene's file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SceneFile {
    /// The environment properties.
    #[serde(default, skip_serializing_if = "is_default")]
    pub environment: Environment,
    /// The environment properties.
    #[serde(default, skip_serializing_if = "is_default")]
    pub camera:      CameraInfo,
    /// The objects found in this scene.
    pub objects:     Vec<Object>,
}
impl SceneFile {
    impl_toml_from_string!();
    impl_toml_to_string!();
    impl_toml_from_path!();
    impl_toml_to_path!();
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Colour;
    use crate::specifications::materials::{Diffuse, Material, NormalMap};
    use crate::specifications::objects::Sphere;

    #[test]
    fn test_scene_file_serialize() {
        assert_eq!(
            SceneFile { camera: CameraInfo::default(), environment: Environment::default(), objects: Vec::new() }.to_string().unwrap(),
            r#"{
  "objects": []
}"#
        );
        assert_eq!(
            SceneFile {
                camera:      CameraInfo::default(),
                environment: Environment::default(),
                objects:     vec![Object::Sphere(Sphere {
                    center:   [0.0, 0.0, 0.0].into(),
                    radius:   1.0,
                    material: Material::NormalMap(NormalMap),
                })],
            }
            .to_string()
            .unwrap(),
            r#"{
  "objects": [
    {
      "Sphere": {
        "center": [
          0.0,
          0.0,
          0.0
        ],
        "radius": 1.0,
        "material": {
          "NormalMap": null
        }
      }
    }
  ]
}"#
        );
        assert_eq!(
            SceneFile {
                camera:      CameraInfo::default(),
                environment: Environment::default(),
                objects:     vec![Object::Sphere(Sphere {
                    center:   [0.0, 0.0, 0.0].into(),
                    radius:   1.0,
                    material: Material::Diffuse(Diffuse { colour: Colour::new(1.0, 1.0, 1.0, 1.0) }),
                })],
            }
            .to_string()
            .unwrap(),
            r#"{
  "objects": [
    {
      "Sphere": {
        "center": [
          0.0,
          0.0,
          0.0
        ],
        "radius": 1.0,
        "material": {
          "Diffuse": {
            "colour": [
              1.0,
              1.0,
              1.0,
              1.0
            ]
          }
        }
      }
    }
  ]
}"#
        );
    }
}
