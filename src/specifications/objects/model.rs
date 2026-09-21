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

#[cfg(feature = "obj")]
use super::super::materials::Lambertian;
use super::super::materials::{LambertianTexture, Material};
use super::super::textures::{SpatialChecker, Texture};
use super::super::transforms::Transform;
use super::plane::Triangle;
use super::{DynObject, Group, JsonObject, Object};
use crate::math::{Colour, Vec3};


/***** CONSTANTS *****/
/// Default, gray material.
pub const DEFAULT_MAT: Material = Material::Lambertian(Lambertian { colour: Colour { r: 0.5, g: 0.5, b: 0.5, a: 1.0 } });
/// Checkered material for when the material was unknown
pub const UNKNOWN_MAT: Material = Material::LambertianTexture(LambertianTexture {
    texture: Texture::SpatialChecker(SpatialChecker {
        scale: 0.35,
        black: Colour { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
        white: Colour { r: 0.0, g: 1.0, b: 1.0, a: 1.0 },
    }),
});





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





/***** LIBRARY *****/


/// Defines an object that loads a model from disk.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Model {
    /// A reference to a to-be-loaded model.
    pub path: PathBuf,
    /// The format to the file if the user bothered to give it.
    pub format: Option<ModelFormat>,
    /// Any transforms to apply to the model.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,
}

// Loading
impl Model {
    /// Loads this Model into a [`Group`] of models.
    ///
    /// # Arguments
    /// - `dir`: The parent directory of the file this model is loaded from.
    ///
    /// # Returns
    /// A new [`Group`] with the loaded triangles.
    ///
    /// # Errors
    /// This function can error if we fail to load the model somehow.
    pub fn load(&self, dir: &Path) -> Result<Group, Error> {
        // Determine a format
        let fmt: ModelFormat = self
            .format
            .ok_or_else(|| {
                // Inspect the file extension to see what's what
                let spath = self.path.to_string_lossy();
                #[cfg(feature = "obj")]
                if spath.ends_with(".obj") {
                    return Ok(ModelFormat::Obj);
                }
                return Err(Error::UnknownModelExtension {
                    name: self.path.file_name().map(OsStr::to_string_lossy).map(Cow::into_owned).unwrap_or_else(String::new),
                });
            })
            .or_else(std::convert::identity)?;

        // Load as that format
        match fmt {
            #[cfg(feature = "obj")]
            ModelFormat::Obj => {
                // Open the file

                use std::collections::HashMap;
                let path: Cow<Path> = if self.path.is_relative() { Cow::Owned(dir.join(&self.path)) } else { Cow::Borrowed(&self.path) };
                debug!("Loading model {path:?} as .obj file...");
                let handle = match File::open(&path) {
                    Ok(handle) => handle,
                    Err(err) => return Err(Error::FileOpen { path: path.into_owned(), err }),
                };

                // Use our libraries to load everything
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

                // Generate materials from the loaded ones
                let mtls: HashMap<String, Material> = mtls
                    .into_iter()
                    .map(|(name, mtl)| {
                        // Note: very narrow, extend as we go
                        if let Some(kd) = mtl.color_diffuse {
                            (name, Material::Lambertian(Lambertian { colour: Colour::new(kd.r, kd.g, kd.b, 1.0) }))
                        } else {
                            panic!("Unsupported coloring on material {name:?}");
                        }
                    })
                    .collect();

                // Generate a list of Raytracer vertices from this
                let mut i: usize = 0;
                let mut groups: Vec<JsonObject> = Vec::with_capacity(obj.objs.values().map(|o| o.faces.len()).sum::<usize>());
                for (oname, obj) in obj.objs {
                    for group in obj.faces {
                        if group.faces.is_empty() {
                            continue;
                        }
                        let mut triags = Vec::with_capacity(group.faces.len());
                        for face in group.faces {
                            // Get the three vertices for this face and turn it into a triangle
                            let mat = group.material.as_ref().map(|m| mtls.get(m).unwrap_or(&UNKNOWN_MAT)).unwrap_or(&DEFAULT_MAT).clone();
                            match face.elems.as_slice() {
                                [v1, v2, v3] => {
                                    let [v1, v2, v3] = [
                                        vertex_get(&obj.vertices, v1.vertex)?,
                                        vertex_get(&obj.vertices, v2.vertex)?,
                                        vertex_get(&obj.vertices, v3.vertex)?,
                                    ];
                                    let [v1, v2, v3] = [Vec3::new(v1.x, v1.y, v1.z), Vec3::new(v2.x, v2.y, v2.z), Vec3::new(v3.x, v3.y, v3.z)];
                                    triags.push(JsonObject::Object(Object {
                                        obj: DynObject::Triangle(Triangle { pos: v1, u: v2 - v1, v: v3 - v1 }),
                                        mat,
                                        volumized: None,
                                        transforms: Vec::new(),
                                    }));
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
                                    triags.push(JsonObject::Object(Object {
                                        obj: DynObject::Triangle(Triangle {
                                            pos: sides[0][0],
                                            u:   sides[0][1] - sides[0][0],
                                            v:   sides[0][2] - sides[0][0],
                                        }),
                                        mat: mat.clone(),
                                        volumized: None,
                                        transforms: Vec::new(),
                                    }));
                                    triags.push(JsonObject::Object(Object {
                                        obj: DynObject::Triangle(Triangle {
                                            pos: sides[1][0],
                                            u:   sides[1][1] - sides[1][0],
                                            v:   sides[1][2] - sides[1][0],
                                        }),
                                        mat,
                                        volumized: None,
                                        transforms: Vec::new(),
                                    }));
                                },
                                _ => return Err(Error::NonTriangleFace { path: path.into(), oname, gname: None, i, got: face.elems.len() }),
                            }
                            i += 1;
                        }
                        groups.push(JsonObject::Group(Group { objs: triags, transforms: Vec::new() }));
                    }
                }

                // When loaded, replace us with the loaded model
                debug!(
                    "Succesfully loaded model {path:?} with {i} faces ({} triangles)",
                    groups.iter().map(|g| if let JsonObject::Group(g) = g { g.objs.len() } else { 0 }).sum::<usize>()
                );
                // for t in &triangles {
                //     println!("{{ {}, {} x {} }}", t.pos, t.u, t.v);
                // }
                Ok(Group { objs: groups, transforms: Vec::new() })
            },
        }
    }
}
