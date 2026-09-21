//  UPDATE SCENE FILES.rs
//    by Lut99
//
//  Description:
//!   Small utility file for updating old scene files to new ones.
//

use std::env;
use std::fs::{self, File};
use std::io::{Seek as _, SeekFrom};
use std::path::PathBuf;
use std::process::ExitCode;

use error_trace::toplevel;
use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info};
use raytracer_new::math as math_new;
use raytracer_new::specifications::{
    materials as materials_new, objects as objects_new, scene as scene_new, textures as textures_new, transforms as transforms_new,
    volumes as volumes_new,
};
use raytracer_old::math as math_old;
use raytracer_old::specifications::{
    animations as animations_old, materials as materials_old, objects as objects_old, scene as scene_old, textures as textures_old,
};


/***** HELPER FUNCTIONS *****/
fn inject_vol_in_json_object(obj: &mut objects_new::JsonObject, vol: volumes_new::Volume) {
    match obj {
        objects_new::JsonObject::Object(o) => {
            o.mat = materials_new::Material::Empty;
            o.volumized = Some(vol);
        },
        objects_new::JsonObject::Group(g) => {
            for obj in &mut g.objs {
                inject_vol_in_json_object(obj, vol.clone());
            }
        },
        objects_new::JsonObject::Model(m) => m.volumized = Some(vol),
    }
}

fn inject_trans_in_json_object(obj: &mut objects_new::JsonObject, trans: transforms_new::Transform) {
    match obj {
        objects_new::JsonObject::Object(o) => {
            o.transforms.push(trans);
        },
        objects_new::JsonObject::Group(g) => {
            for obj in &mut g.objs {
                inject_trans_in_json_object(obj, trans.clone());
            }
        },
        objects_new::JsonObject::Model(m) => m.transforms.push(trans),
    }
}





/***** CONVERSION FUNCTIONS *****/
#[inline]
const fn interval(a: math_old::aabb::Interval) -> math_new::aabb::Interval {
    // SAFETY: OK because the old one is always ordered correctly
    unsafe { math_new::aabb::Interval::new_ordered(a.min(), a.max()) }
}
#[inline]
const fn aabb(a: math_old::AABB) -> math_new::AABB { math_new::AABB { x: interval(a.x), y: interval(a.y), z: interval(a.z) } }

#[inline]
fn colour(c: math_old::Colour) -> math_new::Colour { math_new::Colour::new(c.r, c.g, c.b, c.a) }

#[inline]
const fn vec3(vec: math_old::Vec3) -> math_new::Vec3 { math_new::Vec3::new(vec.x, vec.y, vec.z) }

#[inline]
const fn model_format(fmt: objects_old::model::ModelFormat) -> objects_new::model::ModelFormat {
    match fmt {
        objects_old::model::ModelFormat::Obj => objects_new::model::ModelFormat::Obj,
    }
}

fn object(obj: objects_old::Object) -> objects_new::JsonObject {
    match obj {
        objects_old::Object::AnimatedSphere(s) => objects_new::JsonObject::Object(objects_new::Object {
            obj: objects_new::DynObject::Sphere(objects_new::Sphere { center: vec3(s.sphere.center), radius: s.sphere.radius }),
            mat: material(s.sphere.material),
            volumized: None,
            transforms: vec![animation(s.animation)],
        }),
        objects_old::Object::Box(b) => objects_new::JsonObject::Object(objects_new::Object {
            obj: objects_new::DynObject::Box(objects_new::Box { aabb: aabb(b.aabb) }),
            mat: material(b.material),
            volumized: None,
            transforms: Vec::new(),
        }),
        objects_old::Object::ConstantDensity(cd) => {
            let mut obj = object(*cd.boundary);
            let vol = volumes_new::Volume::ConstantDensity(volumes_new::ConstantDensity {
                density: cd.density,
                phase_function: materials_new::Isotropic { colour: colour(cd.phase_function.colour) },
            });
            inject_vol_in_json_object(&mut obj, vol);
            obj
        },
        objects_old::Object::Group(g) => {
            objects_new::JsonObject::Group(objects_new::Group { objs: g.into_iter().map(object).collect(), transforms: Vec::new() })
        },
        objects_old::Object::Model(objects_old::Model::ToLoad { path, format }) => {
            objects_new::JsonObject::Model(objects_new::Model { path, format: format.map(model_format), volumized: None, transforms: Vec::new() })
        },
        objects_old::Object::Model(_) => unreachable!(),
        objects_old::Object::Quad(q) => objects_new::JsonObject::Object(objects_new::Object {
            obj: objects_new::DynObject::Quad(objects_new::Quad { pos: vec3(q.qd.pos), u: vec3(q.qd.u), v: vec3(q.qd.v) }),
            mat: material(q.material),
            volumized: None,
            transforms: Vec::new(),
        }),
        objects_old::Object::RotateX(rx) => {
            let mut obj = object(*rx.obj);
            let trans = transforms_new::Transform::RotateX(transforms_new::RotateX { angle: rx.angle });
            inject_trans_in_json_object(&mut obj, trans);
            obj
        },
        objects_old::Object::RotateY(ry) => {
            let mut obj = object(*ry.obj);
            let trans = transforms_new::Transform::RotateY(transforms_new::RotateY { angle: ry.angle });
            inject_trans_in_json_object(&mut obj, trans);
            obj
        },
        objects_old::Object::RotateZ(rz) => {
            let mut obj = object(*rz.obj);
            let trans = transforms_new::Transform::RotateZ(transforms_new::RotateZ { angle: rz.angle });
            inject_trans_in_json_object(&mut obj, trans);
            obj
        },
        objects_old::Object::Sphere(s) => objects_new::JsonObject::Object(objects_new::Object {
            obj: objects_new::DynObject::Sphere(objects_new::Sphere { center: vec3(s.center), radius: s.radius }),
            mat: material(s.material),
            volumized: None,
            transforms: Vec::new(),
        }),
        objects_old::Object::Translate(t) => {
            let mut obj = object(*t.obj);
            let trans = transforms_new::Transform::Translate(transforms_new::Translate { pos: vec3(t.pos) });
            inject_trans_in_json_object(&mut obj, trans);
            obj
        },
        objects_old::Object::Triangle(t) => objects_new::JsonObject::Object(objects_new::Object {
            obj: objects_new::DynObject::Triangle(objects_new::Triangle { pos: vec3(t.triag.pos), u: vec3(t.triag.u), v: vec3(t.triag.v) }),
            mat: material(t.material),
            volumized: None,
            transforms: Vec::new(),
        }),
    }
}

fn material(mat: materials_old::Material) -> materials_new::Material {
    match mat {
        materials_old::Material::Dielectric(d) => {
            materials_new::Material::Dielectric(materials_new::Dielectric { refraction_index: d.refraction_index, colour: colour(d.colour) })
        },
        materials_old::Material::Diffuse(d) => materials_new::Material::Diffuse(materials_new::Diffuse { colour: colour(d.colour) }),
        materials_old::Material::DiffuseLight(d) => materials_new::Material::DiffuseLight(materials_new::DiffuseLight { colour: colour(d.colour) }),
        materials_old::Material::Isotropic(i) => materials_new::Material::Isotropic(materials_new::Isotropic { colour: colour(i.colour) }),
        materials_old::Material::Lambertian(l) => materials_new::Material::Lambertian(materials_new::Lambertian { colour: colour(l.colour) }),
        materials_old::Material::LambertianTexture(l) => {
            materials_new::Material::LambertianTexture(materials_new::LambertianTexture { texture: texture(l.texture) })
        },
        materials_old::Material::Metal(m) => materials_new::Material::Metal(materials_new::Metal { colour: colour(m.colour), fuzz: m.fuzz }),
        materials_old::Material::NormalMap(_) => materials_new::Material::NormalMap(materials_new::NormalMap),
        materials_old::Material::PartialDielectric(d) => materials_new::Material::PartialDielectric(materials_new::PartialDielectric {
            refraction_index: d.refraction_index,
            colour: colour(d.colour),
        }),
        materials_old::Material::StaticColour(s) => materials_new::Material::StaticColour(materials_new::StaticColour { colour: colour(s.colour) }),
    }
}

fn texture(tex: textures_old::Texture) -> textures_new::Texture {
    match tex {
        textures_old::Texture::Checker(c) => {
            textures_new::Texture::Checker(textures_new::Checker { scale: c.scale, black: colour(c.black), white: colour(c.white) })
        },
        textures_old::Texture::Gradient(g) => {
            textures_new::Texture::Gradient(textures_new::Gradient { colour1: colour(g.colour1), colour2: colour(g.colour2) })
        },
        textures_old::Texture::Image(textures_old::Image::ToLoad { path, format }) => {
            textures_new::Texture::Image(textures_new::Image::ToLoad { path, format })
        },
        textures_old::Texture::Image(_) => unreachable!(),
        textures_old::Texture::SpatialChecker(c) => {
            textures_new::Texture::SpatialChecker(textures_new::SpatialChecker { scale: c.scale, black: colour(c.black), white: colour(c.white) })
        },
    }
}

fn animation(ani: animations_old::Animation) -> transforms_new::Transform {
    match ani {
        animations_old::Animation::Vertical(v) => transforms_new::Transform::AnimatedTranslate(transforms_new::AnimatedTranslate {
            trajectory: transforms_new::translate::Trajectory::Vertical { len: v.len },
            at: v.at,
            duration: v.duration,
        }),
    }
}





/***** ENTRYPOINT *****/
fn main() -> ExitCode {
    // Setup logger
    let debug: bool = env::var("DEBUG").map(|d| d == "1").unwrap_or(false);
    if let Err(err) = HumanLogger::terminal(if debug { DebugMode::Debug } else { DebugMode::HumanFriendly }).init() {
        eprintln!("WARNING: Could not setup logger: {err} (no logging for this session)");
    }
    info!("{} - v{}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));

    // Load all json files as old
    let scene_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().join("tests").join("scenes");
    let entries = match fs::read_dir(&scene_dir) {
        Ok(entries) => entries,
        Err(err) => {
            error!("{}", toplevel!(("Failed to read directory {scene_dir:?}"), err));
            return ExitCode::FAILURE;
        },
    };
    for (i, entry) in entries.enumerate() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                error!("{}", toplevel!(("Failed to read entry {i} in directory {scene_dir:?}"), err));
                return ExitCode::FAILURE;
            },
        };
        let entry_path = entry.path();
        debug!("Found entry {entry_path:?}");

        // Check if it's JSON
        let entry_name = entry.file_name();
        let entry_name = entry_name.to_string_lossy();
        if !entry_name.ends_with(".json") {
            info!("Skipping entry {entry_path:?}, it is not a JSON file");
            continue;
        }

        // Check if it's parsable as a NEW scene file
        let mut handle = match File::open(&entry_path) {
            Ok(handle) => handle,
            Err(err) => {
                error!("{}", toplevel!(("Failed to open entry {entry_path:?}"), err));
                return ExitCode::FAILURE;
            },
        };
        if serde_json::from_reader::<&mut File, scene_new::SceneFile>(&mut handle).is_ok() {
            info!("Skipping entry {entry_path:?}, it is already a new file");
            continue;
        }

        // Else, check if it's parsable as an OLD scene file
        if let Err(err) = handle.seek(SeekFrom::Start(0)) {
            error!("{}", toplevel!(("Failed to seek in entry {entry_path:?}"), err));
            return ExitCode::FAILURE;
        }
        let old: scene_old::SceneFile = match serde_json::from_reader(handle) {
            Ok(scene) => scene,
            Err(_) => {
                info!("Skipping entry {entry_path:?}, it is not parsable as an old file");
                continue;
            },
        };

        // Now convert it
        let upd = scene_new::SceneFile {
            environment: scene_new::Environment {
                air_refraction_index: old.environment.air_refraction_index,
                background: match old.environment.background {
                    scene_old::Background::Colour(c) => scene_new::Background::Colour(colour(c)),
                    scene_old::Background::IlluminatedSky => scene_new::Background::IlluminatedSky,
                    scene_old::Background::None => scene_new::Background::None,
                },
            },
            camera:      scene_new::CameraInfo {
                dims: old.camera.dims,
                n_samples: old.camera.n_samples,
                vfov: old.camera.vfov,
                defocus_angle: old.camera.defocus_angle,
                focus_dist: old.camera.focus_dist,
                shutter_time: old.camera.shutter_time,
                pos: scene_new::CameraPos {
                    lookfrom: vec3(old.camera.pos.lookfrom),
                    lookat:   vec3(old.camera.pos.lookat),
                    lookup:   vec3(old.camera.pos.lookup),
                },
            },
            objects:     old.objects.into_iter().map(object).collect(),
        };
        let handle = match File::create(&entry_path) {
            Ok(handle) => handle,
            Err(err) => {
                error!("{}", toplevel!(("Failed to re-create entry {entry_path:?}"), err));
                return ExitCode::FAILURE;
            },
        };
        debug!("Writing updated result");
        if let Err(err) = serde_json::to_writer_pretty(handle, &upd) {
            error!("{}", toplevel!(("Failed to write updated entry {entry_path:?}"), err));
            return ExitCode::FAILURE;
        }
        info!("Successfully updated entry");
    }

    ExitCode::SUCCESS
}
