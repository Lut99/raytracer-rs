//  MODEL.rs
//    by Lut99
//
//  Description:
//!   Implements an object that loads a model.
//

use std::borrow::Cow;
use std::ffi::OsStr;
#[cfg(feature = "obj")]
use std::fs::File;
use std::path::PathBuf;

use log::debug;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::super::Loadable;
#[cfg(feature = "obj")]
use super::super::materials::Lambertian;
use super::super::materials::Material;
use super::super::scene::Environment;
use super::plane::Triangle;
use super::{BoundingBoxable, HitRecord, Hittable};
use crate::hittree::HitTree;
use crate::math::{AABB, Colour, Ray, Vec3};


/***** ERRORS *****/
/// Defines problems with loading models.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to open file {path:?}")]
    FileOpen {
        path: PathBuf,
        #[source]
        err:  std::io::Error,
    },
    #[cfg(feature = "obj")]
    #[error("Face {i}{}{} in file {path:?} is not a face of 3/4 vertices (i.e., one or two triangle(s)), but rather {got}", if let Some(oname) = oname {format!(" in object {oname:?}")} else { String::new()}, if let Some(gname) = gname {format!(" in group {gname:?}")} else { String::new()})]
    NonTriangleFace { path: PathBuf, oname: Option<String>, gname: Option<String>, i: usize, got: usize },
    #[cfg(feature = "obj")]
    #[error("Failed to load file {path:?} as .obj file")]
    Obj {
        path: PathBuf,
        #[source]
        err:  obj::Error,
    },
    #[error("Cannot guess format from {name:?} (specify it manually instead)")]
    UnknownModelExtension { name: String },
}





/***** AUXILLARY *****/
/// Defines all the model formats we support.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFormat {
    /// `.obj` file formats.
    #[cfg(feature = "obj")]
    Obj,
}



/// Defines an object that loads a model from disk.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Model {
    /// A model that's already loaded.
    #[serde(skip)]
    Loaded(LoadedModel),
    /// A reference to a to-be-loaded model.
    ToLoad { path: PathBuf, format: Option<ModelFormat>, pos: Vec3, scale: f64 },
}

// Interface
impl Loadable for Model {
    type Error = Error;

    fn load(&mut self) -> Result<(), Self::Error> {
        let Self::ToLoad { path, format, scale, pos } = &*self else { return Ok(()) };

        // Determine a format
        let fmt: ModelFormat = format
            .ok_or_else(|| {
                // Inspect the file extension to see what's what
                let spath = path.to_string_lossy();
                #[cfg(feature = "obj")]
                if spath.ends_with(".obj") {
                    return Ok(ModelFormat::Obj);
                }
                return Err(Error::UnknownModelExtension {
                    name: path.file_name().map(OsStr::to_string_lossy).map(Cow::into_owned).unwrap_or_else(String::new),
                });
            })
            .or_else(std::convert::identity)?;

        // Load as that format
        match fmt {
            #[cfg(feature = "obj")]
            ModelFormat::Obj => {
                // Open the file
                debug!("Loading model {path:?} as .obj file...");
                let handle = File::open(path).map_err(|err| Error::FileOpen { path: path.clone(), err })?;

                // Use our library for this
                let obj = obj::Obj::from_reader(handle).map_err(|err| Error::Obj { path: path.clone(), err })?;

                // Generate a list of Raytracer vertices from this
                let mut i: usize = 0;
                let mut vertices = Vec::with_capacity(obj.objs.values().map(|o| o.faces.values().map(|g| g.faces.len()).sum::<usize>()).sum());
                for (oname, obj) in obj.objs {
                    for (gname, group) in obj.faces {
                        for face in group.faces {
                            // Get the three vertices for this face and turn it into a triangle
                            match face.elems.as_slice() {
                                [v1, v2, v3] => {
                                    let [v1, v2, v3] = [obj.vertices[v1.vertex - 1], obj.vertices[v2.vertex - 1], obj.vertices[v3.vertex - 1]];
                                    let [v1, v2, v3] = [
                                        *pos + *scale * Vec3::new(v1.x, v1.y, v1.z),
                                        *pos + *scale * Vec3::new(v2.x, v2.y, v2.z),
                                        *pos + *scale * Vec3::new(v3.x, v3.y, v3.z),
                                    ];
                                    vertices.push(Triangle {
                                        pos: v1,
                                        u: v2 - v1,
                                        v: v3 - v1,
                                        material: Material::Lambertian(Lambertian { colour: Colour::new(0.5, 0.5, 0.5, 1.0) }),
                                    });
                                },
                                [v1, v2, v3, v4] => {
                                    // Split it into TWO triangles
                                    let [v1, v2, v3, v4] = [
                                        obj.vertices[v1.vertex - 1],
                                        obj.vertices[v2.vertex - 1],
                                        obj.vertices[v3.vertex - 1],
                                        obj.vertices[v4.vertex - 1],
                                    ];
                                    let [v1, v2, v3, v4] = [
                                        *pos + *scale * Vec3::new(v1.x, v1.y, v1.z),
                                        *pos + *scale * Vec3::new(v2.x, v2.y, v2.z),
                                        *pos + *scale * Vec3::new(v3.x, v3.y, v3.z),
                                        *pos + *scale * Vec3::new(v4.x, v4.y, v4.z),
                                    ];
                                    vertices.push(Triangle {
                                        pos: v1,
                                        u: v2 - v1,
                                        v: v3 - v1,
                                        material: Material::Lambertian(Lambertian { colour: Colour::new(0.5, 0.5, 0.5, 1.0) }),
                                    });
                                    vertices.push(Triangle {
                                        pos: v2,
                                        u: v3 - v2,
                                        v: v4 - v2,
                                        material: Material::Lambertian(Lambertian { colour: Colour::new(0.5, 0.5, 0.5, 1.0) }),
                                    });
                                },
                                _ => return Err(Error::NonTriangleFace { path: path.clone(), oname, gname, i, got: face.elems.len() }),
                            }
                            i += 1;
                        }
                    }
                }

                // When loaded, replace us with the loaded model
                debug!("Succesfully loaded model {path:?} with {i} faces");
                *self = Self::Loaded(LoadedModel { vertices: HitTree::with_objs(vertices, (0..=1).into()) });
                Ok(())
            },
        }
    }
}
impl BoundingBoxable for Model {
    #[inline]
    fn aabb(&self, t_us: u64) -> AABB {
        match self {
            Self::Loaded(m) => m.aabb(t_us),
            Self::ToLoad { path, format: _, pos: _, scale: _ } => panic!("Cannot get AABB of unloaded model {path:?}"),
        }
    }
}
impl Hittable<Material> for Model {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&'_ Material>> {
        match self {
            Self::Loaded(m) => m.hit(ray, t_min, t_max, env),
            Self::ToLoad { path, format: _, pos: _, scale: _ } => panic!("Cannot check hit of unloaded model {path:?}"),
        }
    }
}





/***** LIBRARY *****/
/// A loaded counterpart of [`Model`].
#[derive(Clone, Debug)]
pub struct LoadedModel {
    /// The list of vertices in this model.
    vertices: HitTree<Triangle<Material>>,
}

// Raytracer
impl BoundingBoxable for LoadedModel {
    #[inline]
    fn aabb(&self, t_us: u64) -> AABB { self.vertices.aabb(t_us) }
}
impl Hittable<Material> for LoadedModel {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&'_ Material>> {
        self.vertices.hit(ray, t_min, t_max, env)
    }
}
