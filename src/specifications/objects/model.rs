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
use std::path::{Path, PathBuf};

use log::debug;
use obj::Vertex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::super::Loadable;
#[cfg(feature = "obj")]
use super::super::materials::Lambertian;
use super::super::materials::Material;
use super::super::scene::Environment;
use super::plane::Triag;
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
    #[error("Index {got} overflows for list of length {len}")]
    IndexOverflow { got: isize, len: usize },
    #[cfg(feature = "obj")]
    #[error("Failed to load file {path:?} as .mtl file")]
    Mtllib {
        path: PathBuf,
        #[source]
        err:  mtllib::Error,
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
    #[cfg(feature = "obj")]
    #[error("Encountered zero index")]
    ZeroIndex,
}





/***** HELPER FUNCTIONS *****/
/// Resolves a [`isize`] index to a vertex.
#[cfg(feature = "obj")]
#[inline]
const fn vertex_get(vertices: &[Vertex], i: isize) -> Result<&Vertex, Error> {
    if i < 0 {
        let ri: isize = vertices.len() as isize - i;
        if ri >= 0 { Ok(&vertices[ri as usize]) } else { Err(Error::IndexOverflow { got: i, len: vertices.len() }) }
    } else if i > 0 {
        if i as usize <= vertices.len() { Ok(&vertices[i as usize - 1]) } else { Err(Error::IndexOverflow { got: i, len: vertices.len() }) }
    } else {
        Err(Error::ZeroIndex)
    }
}

/// Split a face of four points into two triangles.
///
/// # Arguments
/// - `vs`: The list of vertices that are the points to split along.
///
/// # Returns
/// Two sets of vertices that make two triangles.
fn split_four_into_triangles(vs: [Vec3; 4]) -> [[Vec3; 3]; 2] {
    #[inline]
    const fn midpoint_of(p1: Vec3, p2: Vec3) -> Vec3 { Vec3::new(0.5 * (p1.x + p2.x), 0.5 * (p1.y + p2.y), 0.5 * (p1.z + p2.z)) }

    // The first triangle is the first three vertices
    let t1 = [vs[0], vs[1], vs[2]];

    // The second triangle is the line + the fourth point s.t. the fourth point is closest to it.
    // See: <https://stackoverflow.com/a/73431349/5270125>
    let axis = [(t1[0], t1[1]), (t1[1], t1[2]), (t1[0], t1[2])];
    let mut smallest_l: Option<(usize, f64)> = None;
    for (i, (p1, p2)) in axis.into_iter().enumerate() {
        let m = midpoint_of(p1, p2);
        let dist = (vs[3] - m).length2();
        if let Some((si, sd)) = &mut smallest_l {
            if dist < *sd {
                *si = i;
                *sd = dist;
            }
        } else {
            smallest_l = Some((i, dist));
        }
    }
    let smallest_i: usize = smallest_l.unwrap().0;
    [t1, [axis[smallest_i].0, axis[smallest_i].1, vs[3]]]
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
    ToLoad { path: PathBuf, format: Option<ModelFormat> },
}

// Interface
impl Loadable for Model {
    type Error = Error;

    fn load(&mut self) -> Result<(), Self::Error> {
        let Self::ToLoad { path, format } = &*self else { return Ok(()) };

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

                use std::collections::HashMap;
                let path: Cow<Path> = if path.is_relative() { Cow::Owned(dir.join(path)) } else { Cow::Borrowed(path) };
                debug!("Loading model {path:?} as .obj file...");
                let handle = match File::open(&path) {
                    Ok(handle) => handle,
                    Err(err) => return Err(Error::FileOpen { path: path.into_owned(), err }),
                };

                // Use our libraries for this
                let obj = match obj::Obj::from_reader(handle) {
                    Ok(handle) => handle,
                    Err(err) => return Err(Error::Obj { path: path.into_owned(), err }),
                };
                let mut mtls = HashMap::<String, mtllib::Material>::new();
                for mtl in &obj.mtllibs {
                    // Resolve the path
                    let mtl: Cow<Path> = if mtl.is_relative() { Cow::Owned(dir.join(mtl)) } else { Cow::Borrowed(mtl) };

                    // Attempt to load the file
                    debug!("Loading model {mtl:?} as .mtllib file...");
                    let handle = match File::open(&mtl) {
                        Ok(handle) => handle,
                        Err(err) => return Err(Error::FileOpen { path: mtl.into(), err }),
                    };
                    let mtl = match mtllib::Mtl::from_reader(handle) {
                        Ok(mtl) => mtl,
                        Err(err) => return Err(Error::Mtllib { path: mtl.into(), err }),
                    };
                    mtls.extend(mtl.mtls);
                }

                // Generate a list of Raytracer vertices from this
                let mut i: usize = 0;
                let mut triangles = Vec::with_capacity(obj.objs.values().map(|o| o.faces.values().map(|g| g.faces.len()).sum::<usize>()).sum());
                for (oname, obj) in obj.objs {
                    for (gname, group) in obj.faces {
                        for face in group.faces {
                            // Get the three vertices for this face and turn it into a triangle
                            match face.elems.as_slice() {
                                [v1, v2, v3] => {
                                    let [v1, v2, v3] = [
                                        vertex_get(&obj.vertices, v1.vertex)?,
                                        vertex_get(&obj.vertices, v2.vertex)?,
                                        vertex_get(&obj.vertices, v3.vertex)?,
                                    ];
                                    let [v1, v2, v3] = [Vec3::new(v1.x, v1.y, v1.z), Vec3::new(v2.x, v2.y, v2.z), Vec3::new(v3.x, v3.y, v3.z)];
                                    triangles.push(Triag { pos: v1, u: v2 - v1, v: v3 - v1 });
                                },
                                [v1, v2, v3, v4] => {
                                    // Get the vertex equivalent
                                    let [v1, v2, v3, v4] = [
                                        vertex_get(&obj.vertices, v1.vertex)?,
                                        vertex_get(&obj.vertices, v2.vertex)?,
                                        vertex_get(&obj.vertices, v3.vertex)?,
                                        vertex_get(&obj.vertices, v4.vertex)?,
                                    ];
                                    let [v1, v2, v3, v4] = [
                                        Vec3::new(v1.x, v1.y, v1.z),
                                        Vec3::new(v2.x, v2.y, v2.z),
                                        Vec3::new(v3.x, v3.y, v3.z),
                                        Vec3::new(v4.x, v4.y, v4.z),
                                    ];

                                    // Split it into two triangles and add them
                                    let sides = split_four_into_triangles([v1, v2, v3, v4]);
                                    triangles.push(Triag { pos: sides[0][0], u: sides[0][1] - sides[0][0], v: sides[0][2] - sides[0][0] });
                                    triangles.push(Triag { pos: sides[1][0], u: sides[1][1] - sides[1][0], v: sides[1][2] - sides[1][0] });
                                },
                                _ => return Err(Error::NonTriangleFace { path: path.into(), oname, gname, i, got: face.elems.len() }),
                            }
                            i += 1;
                        }
                    }
                }

                // When loaded, replace us with the loaded model
                debug!("Succesfully loaded model {path:?} with {i} faces ({} triangles)", triangles.len());
                // for t in &triangles {
                //     println!("{{ {}, {} x {} }}", t.pos, t.u, t.v);
                // }
                *self = Self::Loaded(LoadedModel {
                    aabb:   triangles.iter().map(|t| t.aabb(0)).collect(),
                    groups: vec![LoadedGroup {
                        triags: HitTree::with_objs(triangles, (0..=1).into()),
                        mat:    Material::Lambertian(Lambertian { colour: Colour::new(fastrand::f64(), fastrand::f64(), fastrand::f64(), 1.0) }),
                    }],
                });
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
            Self::ToLoad { path, format: _ } => panic!("Cannot get AABB of unloaded model {path:?}"),
        }
    }
}
impl Hittable<Material> for Model {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&'_ Material>> {
        match self {
            Self::Loaded(m) => m.hit(ray, t_min, t_max, env),
            Self::ToLoad { path, format: _ } => panic!("Cannot check hit of unloaded model {path:?}"),
        }
    }
}





/***** LIBRARY *****/
/// Represents a group of triangles, already loaded.
#[derive(Clone, Debug)]
struct LoadedGroup {
    /// A list of triangles that we can render.
    triags: HitTree<Triag>,
    /// The material that we render with.
    mat:    Material,
}

// Interface
impl BoundingBoxable for LoadedGroup {
    #[inline]
    fn aabb(&self, t_us: u64) -> AABB { self.triags.aabb(t_us) }
}
impl Hittable<Material> for LoadedGroup {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&Material>> {
        self.triags.hit(ray, t_min, t_max, env).map(|rec| HitRecord { mat: &self.mat, data: rec.data })
    }
}



/// A loaded counterpart of [`Model`].
#[derive(Clone, Debug)]
pub struct LoadedModel {
    /// Overarching set of AABBs.
    aabb:   AABB,
    /// A set of groups, each with their own material.
    groups: Vec<LoadedGroup>,
}

// Raytracer
impl BoundingBoxable for LoadedModel {
    #[inline]
    fn aabb(&self, _t_us: u64) -> AABB { self.aabb }
}
impl Hittable<Material> for LoadedModel {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&'_ Material>> {
        // Attempt to hit all groups
        let mut hit = None;
        let mut t = t_max;
        for g in &self.groups {
            if let Some(ghit) = g.hit(ray, t_min, t, env) {
                hit = Some(ghit);
                t = ghit.data.t;
            }
        }
        hit
    }
}
