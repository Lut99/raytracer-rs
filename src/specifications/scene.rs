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

use serde::{Deserialize, Serialize};

use crate::common::file::{impl_toml_from_path, impl_toml_from_string, impl_toml_to_path, impl_toml_to_string};
use crate::math::Vec3;
use crate::specifications::animations::vertical::Vertical;
use crate::specifications::materials::{Dielectric, Diffuse, Lambertian, Metal, NormalMap, PartialDielectric, StaticColour};
use crate::specifications::objects::Sphere;
use crate::specifications::objects::sphere::AnimatedSphere;


/***** HELPER FUNCTIONS *****/
/// Checks for the default environment props.
#[inline]
pub fn is_default_environment(env: &Environment) -> bool { env == &Environment::default() }

/// Checks for the default camera props.
#[inline]
pub fn is_default_camera(cam: &Camera) -> bool { cam == &Camera::default() }





/***** AUXILLARY *****/
/// Helper trait that we use to get some specialization in on retrieving the internal object in the [`Object`] and [`Material`] enums.
pub trait IntoInner<T> {
    /// Returns ourselves as the given type if we are.
    ///
    /// # Returns
    /// The internal object, or [`None`].
    fn into_inner(self) -> Option<T>;
}



/// Defines an abstraction over objects that makes it more intuitive for the user to pass them.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Object {
    // Normal objects
    /// A perfect sphere.
    Sphere(Sphere<Material>),
    /// A perfect sphere with animation.
    AnimatedSphere(AnimatedSphere<Material, Animation>),
}

impl<M> IntoInner<Sphere<M>> for Object
where
    Material: IntoInner<M>,
{
    #[inline]
    fn into_inner(self) -> Option<Sphere<M>> {
        if let Self::Sphere(s) = self { Some(Sphere { center: s.center, radius: s.radius, material: s.material.into_inner()? }) } else { None }
    }
}
impl<M, A> IntoInner<AnimatedSphere<M, A>> for Object
where
    Material: IntoInner<M>,
    Animation: IntoInner<A>,
{
    #[inline]
    fn into_inner(self) -> Option<AnimatedSphere<M, A>> {
        if let Self::AnimatedSphere(s) = self {
            Some(AnimatedSphere {
                sphere:    Sphere { center: s.sphere.center, radius: s.sphere.radius, material: s.sphere.material.into_inner()? },
                animation: s.animation.into_inner()?,
            })
        } else {
            None
        }
    }
}



/// Defines an abstraction over materials that we can use to parse objects independently from sphere.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Material {
    // Basic materials
    /// A non-lighted static colour.
    StaticColour(StaticColour),
    /// A non-lighted normal map.
    NormalMap(NormalMap),

    // Diffuse materials
    /// The basic diffuse material.
    Diffuse(Diffuse),
    /// A better diffuse material.
    Lambertian(Lambertian),
    /// Metallic material.
    Metal(Metal),
    /// (Partially correct) Dielectric material.
    PartialDielectric(PartialDielectric),
    /// Dielectric material.
    Dielectric(Dielectric),
}

impl IntoInner<StaticColour> for Material {
    #[inline]
    fn into_inner(self) -> Option<StaticColour> { if let Self::StaticColour(c) = self { Some(c) } else { None } }
}
impl IntoInner<NormalMap> for Material {
    #[inline]
    fn into_inner(self) -> Option<NormalMap> { if let Self::NormalMap(nm) = self { Some(nm) } else { None } }
}

impl IntoInner<Diffuse> for Material {
    #[inline]
    fn into_inner(self) -> Option<Diffuse> { if let Self::Diffuse(d) = self { Some(d) } else { None } }
}
impl IntoInner<Lambertian> for Material {
    #[inline]
    fn into_inner(self) -> Option<Lambertian> { if let Self::Lambertian(d) = self { Some(d) } else { None } }
}
impl IntoInner<Metal> for Material {
    #[inline]
    fn into_inner(self) -> Option<Metal> { if let Self::Metal(m) = self { Some(m) } else { None } }
}
impl IntoInner<PartialDielectric> for Material {
    #[inline]
    fn into_inner(self) -> Option<PartialDielectric> { if let Self::PartialDielectric(m) = self { Some(m) } else { None } }
}
impl IntoInner<Dielectric> for Material {
    #[inline]
    fn into_inner(self) -> Option<Dielectric> { if let Self::Dielectric(m) = self { Some(m) } else { None } }
}



/// Collects all animations.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Animation {
    Vertical(Vertical),
}

impl IntoInner<Vertical> for Animation {
    #[inline]
    fn into_inner(self) -> Option<Vertical> {
        let Self::Vertical(v) = self;
        Some(v)
    }
}





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



/// Defines properties of the camera in a scene.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Camera {
    /// The dimensions of the camera.
    pub dims: (u32, u32),
    /// The vertical field-of-view of the camera.
    pub vfov: f64,
    /// The vertical field-of-view of the camera.
    pub defocus_angle: f64,
    /// The vertical field-of-view of the camera.
    pub focus_dist: f64,
    /// The shutter time, in microseconds, of the camera.
    ///
    /// Use `1` to disable it (instant shutter).
    pub shutter_time: u64,
    /// The point the camera is looking _from_.
    pub lookfrom: Vec3,
    /// The point the camera is looking _at_.
    pub lookat: Vec3,
    /// The vector pointing the camera up.
    pub lookup: Vec3,
}
impl Default for Camera {
    #[inline]
    fn default() -> Self {
        Camera {
            dims: (800, 600),
            vfov: 90.0,
            defocus_angle: 0.0,
            focus_dist: 0.0,
            shutter_time: 1,
            lookfrom: Vec3::new(0.0, 0.0, 0.0),
            lookat: Vec3::new(0.0, 0.0, -1.0),
            lookup: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}
impl From<Camera> for crate::math::Camera {
    #[inline]
    fn from(value: Camera) -> Self {
        crate::math::Camera::new(
            value.dims,
            value.vfov,
            value.defocus_angle,
            value.focus_dist,
            value.shutter_time,
            value.lookfrom,
            value.lookat,
            value.lookup,
        )
    }
}



/// The SceneFile defines the scene's file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SceneFile {
    /// The environment properties.
    #[serde(default, skip_serializing_if = "is_default_environment")]
    pub environment: Environment,
    /// The environment properties.
    #[serde(default, skip_serializing_if = "is_default_camera")]
    pub camera:      Camera,
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

    #[test]
    fn test_scene_file_serialize() {
        assert_eq!(
            SceneFile { camera: Camera::default(), environment: Environment::default(), objects: Vec::new() }.to_string().unwrap(),
            r#"{
  "objects": []
}"#
        );
        assert_eq!(
            SceneFile {
                camera:      Camera::default(),
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
                camera:      Camera::default(),
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
