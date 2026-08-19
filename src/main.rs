//  MAIN.rs
//    by Lut99
//
//  Created:
//    23 Apr 2023, 11:30:03
//  Last edited:
//    19 May 2023, 12:53:51
//  Auto updated?
//    Yes
//
//  Description:
//!   Entrypoint to the main `raytracer` application.
//

use std::num::NonZeroU64;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use error_trace::{ErrorTrace as _, toplevel};
use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info};
use raytracer::common::input::Dimensions;
use raytracer::generate;
use raytracer::hittree::HitTree;
use raytracer::math::{AABB, Camera, Colour, Vec3};
use raytracer::render::backends::multi::{MultiThreadRenderer, MultiThreadRendererConfig};
use raytracer::render::backends::single::SingleThreadRenderer;
use raytracer::render::image::Image;
use raytracer::render::{RayRenderer as _, RenderBackend};
use raytracer::specifications::Loadable as _;
use raytracer::specifications::animations::{Animation, Vertical};
use raytracer::specifications::materials::{Dielectric, DiffuseLight, Isotropic, Lambertian, LambertianTexture, Material, Metal};
use raytracer::specifications::objects::plane::Qd;
use raytracer::specifications::objects::{AnimatedSphere, Box, ConstantDensity, Object, Quad, RotateY, Sphere, Translate};
use raytracer::specifications::scene::{Background, Environment, SceneFile};
use raytracer::specifications::textures::image::Image as TexImage;
use raytracer::specifications::textures::{SpatialChecker, Texture};


/***** ARGUMENTS *****/
/// Defines the arguments for the `raytracer` application.
#[derive(Debug, Parser)]
struct Arguments {
    /// Whether to set [`DebugMode::Debug`] instead of [`DebugMode::HumanFriendly`].
    #[clap(
        long,
        global = true,
        help = "If given, will enable additional debug prints (at the `info` and `debug` log level). Also makes the `warning` and `error` prints \
                more extensive."
    )]
    debug: bool,
    /// Whether to set [`DebugMode::Full`] instead of [`DebugMode::HumanFriendly`].
    #[clap(long, global = true, help = "If given, will enable most verbose debug prints. Implies `--debug`.")]
    trace: bool,

    /// The particular subcommand to select.
    #[clap(subcommand)]
    subcommand: RaytracerSubcommand,
}



/// Defines subcommands for the `raytracer` application.
#[derive(Debug, Subcommand)]
enum RaytracerSubcommand {
    /// Renders a new scene.
    #[clap(name = "render", about = "Renders a particular scene.")]
    Render(RenderArguments),
    /// Generates something.
    #[clap(name = "generate", about = "Generates files for testing or for rendering.")]
    Generate(GenerateArguments),
}

/// Defines the arguments for the `render` subcommand.
#[derive(Debug, Parser)]
struct RenderArguments {
    /// The output size of the image.
    #[clap(short, long, help = "The size of the output image for this render. If omitted, defaults to the value in the scene file.")]
    dims:     Option<Dimensions>,
    /// Whether to fix missing directories when generating the output image or not.
    #[clap(short, long, help = "If given, will generate missing directories for the output image.")]
    fix_dirs: bool,

    /// The backend to use for rendering.
    #[clap(short, long, default_value = "single", help = "The backend to use for rendering.")]
    backend: RenderBackend,
    /// Any additional config parameters to set for the backend file.
    #[clap(long, help = "If given, defines a file that defines backend-specific properties.")]
    backend_config: Option<PathBuf>,

    /// Whether to enable gamma correction (or rather, to disable it).
    #[clap(long, help = "If given, disables gamma correction")]
    disable_gamma_correction: bool,
    /// Whether to enable anti-aliasing (or rather, to disable it).
    #[clap(long, help = "If given, disables anti-aliasing (shorthand for '--n-samples 1')")]
    disable_anti_aliasing: bool,
    /// Determines the number of rays to cast per pixel.
    #[clap(
        long,
        help = "The number of rays to cast per pixel. Setting to '1' implies disabling anti-aliasing. If omitted, uses the value from the scene \
                file."
    )]
    n_samples: Option<NonZeroU64>,
    /// Determines the number of times a ray may bounce at most.
    #[clap(
        long,
        help = "The number of times a ray may bounce at most. Setting to '1' implies not bouncing anything ever (i.e., direct illumination), and \
                setting to '0' not even fires the ray. If omitted, uses the value from the scene file."
    )]
    ray_max_depth: Option<usize>,

    /// A once-more nested subcommand that defines what type of media to render.
    #[clap(subcommand)]
    media: RenderSubcommand,
}
/// Defines the subcommands for the `render` subcommand.
#[derive(Debug, Subcommand)]
enum RenderSubcommand {
    /// Renders a single frame/image.
    #[clap(name = "image", alias = "frame", about = "Renders a single frame of the given scene.")]
    Image(RenderImageArguments),
    /// Renders the cover of the book.
    #[clap(name = "cover", alias = "book", about = "Renders the cover of the Raytracing In One Weekend book.")]
    Cover(RenderCoverArguments),
}
/// Defines the arguments for the `render image` subcommand.
#[derive(Debug, Parser)]
struct RenderImageArguments {
    /// The path to the scene file to render.
    #[clap(name = "SCENE_PATH", help = "The path to the scene file which we want to render.")]
    scene_path:  PathBuf,
    /// The path to the image file to output.
    #[clap(name = "OUTPUT_PATH", default_value = "./image.png", help = "The path to write the rendered image to.")]
    output_path: PathBuf,
}
/// Defines the arguments for the `render image` subcommand.
#[derive(Debug, Parser)]
struct RenderCoverArguments {
    /// Which book cover to render
    #[clap(name = "BOOK", help = "The book of who we render the cover.")]
    book: Book,
    /// Any shutter time (in microseconds) to set. Since the cover is secretly animated, setting this will reveal motion blur.
    #[clap(short, long, default_value = "1000")]
    shutter_time: u64,
    /// The path to the image file to output.
    #[clap(name = "OUTPUT_PATH", default_value = "./image.png", help = "The path to write the rendered image to.")]
    output_path: PathBuf,
}
/// Defines possible book covers.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "snake_case")]
enum Book {
    #[clap(alias = "book1")]
    OneWeekend,
    #[clap(alias = "book2")]
    NextWeek,
}

/// Defines the arguments for the `generate` subcommand.
#[derive(Debug, Parser)]
struct GenerateArguments {
    /// Whether to create missing directories or not.
    #[clap(short, long, global = true, help = "If given, generates missing directories instead of erroring.")]
    fix_dirs: bool,

    /// The thing to generate.
    #[clap(subcommand)]
    subcommand: GenerateSubcommand,
}
/// Defines the things we can generate.
#[derive(Debug, Subcommand)]
enum GenerateSubcommand {
    #[clap(name = "gradient", about = "Generates the test gradient image discussed in the tutorial.")]
    Gradient {
        /// The output path where to generate the file to.
        #[clap(name = "PATH", default_value = "./image.png", help = "The output path to generate the file to.")]
        path: PathBuf,
        /// The dimensions of the image, given as `WIDTHxHEIGHT`.
        #[clap(
            name = "DIMENSIONS",
            default_value = "256x256",
            help = "The dimensions of the output image. Should be given as a `<WIDTH>x<HEIGHT>` pair, where `<WIDTH>` is the image's width, and \
                    `<HEIGHT>` is the image's height."
        )]
        dims: Dimensions,
    },
}





/***** ENTRYPOINT *****/
fn main() -> ExitCode {
    // Read the command-line arguments
    let args: Arguments = Arguments::parse();

    // Setup the logger
    if let Err(err) = HumanLogger::terminal(DebugMode::from_flags(args.trace, args.debug)).init() {
        eprintln!("WARNING: Failed to setup logger: {err} (no logging enabled for this session)");
    }
    info!("raytracer-rs v{}", env!("CARGO_PKG_VERSION"));

    // Match on the subcommand
    match args.subcommand {
        RaytracerSubcommand::Render(render) => {
            // Match further on the media type
            match render.media {
                RenderSubcommand::Image(image) => {
                    // Load the given scene file
                    debug!("Loading scene file '{}'...", image.scene_path.display());
                    let mut scene: SceneFile = match SceneFile::from_path(&image.scene_path) {
                        Ok(scene) => scene,
                        Err(err) => {
                            error!("{}", err.trace());
                            return ExitCode::FAILURE;
                        },
                    };
                    if let Some(dims) = render.dims {
                        scene.camera.dims = (dims.0, dims.1);
                    }
                    if let Some(n_samples) = render.n_samples {
                        scene.camera.n_samples = n_samples;
                    }
                    if render.disable_anti_aliasing {
                        // SAFETY: It's 1
                        scene.camera.n_samples = unsafe { NonZeroU64::new_unchecked(1) };
                    }

                    // Convert that to a static HitList and load it
                    for (i, obj) in scene.objects.iter_mut().enumerate() {
                        if let Err(err) = obj.load(image.scene_path.parent().unwrap_or(&image.scene_path)) {
                            error!("{}", toplevel!(("Failed to load external references in object {i}"), err));
                            return ExitCode::FAILURE;
                        }
                    }
                    let list: HitTree = HitTree::with_objs(scene.objects, (0..=scene.camera.shutter_time.into()).into());

                    // Now render based on the backend
                    let output: Image = match render.backend {
                        RenderBackend::SingleThreaded => {
                            debug!("Rendering with single-threaded backend");
                            let renderer: SingleThreadRenderer =
                                SingleThreadRenderer::new(true, render.ray_max_depth.unwrap_or(50), !render.disable_gamma_correction);
                            renderer.render_frame(&list, &Camera::from(scene.camera), &scene.environment).unwrap()
                        },

                        RenderBackend::MultiThreaded => {
                            debug!("Rendering with multi-threaded backend");

                            // Read the given file, if any
                            let config: MultiThreadRendererConfig = match render.backend_config {
                                Some(path) => {
                                    debug!("Loading multi-threaded backend file '{}'...", path.display());
                                    match MultiThreadRendererConfig::from_path(path) {
                                        Ok(config) => config,
                                        Err(err) => {
                                            error!("{}", err.trace());
                                            return ExitCode::FAILURE;
                                        },
                                    }
                                },
                                None => Default::default(),
                            };

                            // Create the backend
                            let renderer: MultiThreadRenderer =
                                match MultiThreadRenderer::new(true, render.ray_max_depth.unwrap_or(50), !render.disable_gamma_correction, config) {
                                    Ok(renderer) => renderer,
                                    Err(err) => {
                                        error!("{}", err.trace());
                                        return ExitCode::FAILURE;
                                    },
                                };

                            // Now render with this backend
                            renderer.render_frame(&list, &Camera::from(scene.camera), &scene.environment).unwrap()
                        },
                    };

                    // Now write the image to disk
                    if let Err(err) = output.to_path(&image.output_path, render.fix_dirs) {
                        error!("Failed to save rendered image to '{}': {}", image.output_path.display(), err);
                        return ExitCode::FAILURE;
                    }
                    ExitCode::SUCCESS
                },

                RenderSubcommand::Cover(cover) => {
                    // Generate the list of objects for the correct book
                    let mut objects: Vec<Object> = match cover.book {
                        Book::OneWeekend => {
                            let mut objects: Vec<Object> = Vec::with_capacity(1 + 21 * 21 + 3);
                            objects.push(Object::Sphere(Sphere {
                                center:   Vec3::new(0.0, -1000.0, 0.0),
                                radius:   1000.0,
                                material: Material::LambertianTexture(LambertianTexture {
                                    texture: Texture::SpatialChecker(SpatialChecker {
                                        scale: 0.32,
                                        black: Colour::new(0.2, 0.3, 0.1, 1.0),
                                        white: Colour::new(0.9, 0.9, 0.9, 1.0),
                                    }),
                                }),
                            }));
                            for a in -11..11 {
                                for b in -11..11 {
                                    let mat = fastrand::f64();
                                    let center = Vec3::new(a as f64 + 0.9 * fastrand::f64(), 0.2, b as f64 + 0.9 * fastrand::f64());
                                    if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                                        if mat < 0.8 {
                                            // It'll be a tiny diffuse sphere
                                            let colour = Colour::new(fastrand::f64(), fastrand::f64(), fastrand::f64(), 1.0);
                                            let sphere = Sphere { center, radius: 0.2, material: Material::Lambertian(Lambertian { colour }) };
                                            objects.push(if fastrand::f64() < 0.1 {
                                                Object::AnimatedSphere(AnimatedSphere {
                                                    sphere,
                                                    animation: Animation::Vertical(Vertical { len: 0.5 * fastrand::f64(), at: 0, duration: 1000 }),
                                                })
                                            } else {
                                                Object::Sphere(sphere)
                                            });
                                        } else if mat < 0.95 {
                                            // Metal, with random fuzziness
                                            let colour = Colour::new(
                                                fastrand::f64() / 2.0 + 0.5,
                                                fastrand::f64() / 2.0 + 0.5,
                                                fastrand::f64() / 2.0 + 0.5,
                                                1.0,
                                            );
                                            let fuzz = fastrand::f64() / 2.0;
                                            objects.push(Object::Sphere(Sphere {
                                                center,
                                                radius: 0.2,
                                                material: Material::Metal(Metal { colour, fuzz }),
                                            }));
                                        } else {
                                            // Glass
                                            objects.push(Object::Sphere(Sphere {
                                                center,
                                                radius: 0.2,
                                                material: Material::Dielectric(Dielectric {
                                                    refraction_index: 1.5,
                                                    colour: Colour::new(1.0, 1.0, 1.0, 1.0),
                                                }),
                                            }));
                                        }
                                    }
                                }
                            }
                            objects.push(Object::Sphere(Sphere {
                                center:   Vec3::new(0.0, 1.0, 0.0),
                                radius:   1.0,
                                material: Material::Dielectric(Dielectric { refraction_index: 1.5, colour: Colour::new(1.0, 1.0, 1.0, 1.0) }),
                            }));
                            objects.push(Object::Sphere(Sphere {
                                center:   Vec3::new(-4.0, 1.0, 0.0),
                                radius:   1.0,
                                material: Material::Lambertian(Lambertian { colour: Colour::new(0.4, 0.2, 0.1, 1.0) }),
                            }));
                            objects.push(Object::Sphere(Sphere {
                                center:   Vec3::new(4.0, 1.0, 0.0),
                                radius:   1.0,
                                material: Material::Metal(Metal { colour: Colour::new(0.7, 0.6, 0.5, 1.0), fuzz: 0.0 }),
                            }));
                            objects
                        },

                        Book::NextWeek => {
                            // Define materials
                            let ground = Material::Lambertian(Lambertian { colour: Colour::new(0.48, 0.83, 0.53, 1.0) });
                            let light = Material::DiffuseLight(DiffuseLight { colour: Colour::new(7.0, 7.0, 7.0, 1.0) });
                            let brown = Material::Lambertian(Lambertian { colour: Colour::new(0.7, 0.3, 0.1, 1.0) });
                            let glass = Material::Dielectric(Dielectric { colour: Colour::new(1.0, 1.0, 1.0, 1.0), refraction_index: 1.5 });
                            let grey_metal = Material::Metal(Metal { colour: Colour::new(0.8, 0.8, 0.9, 1.0), fuzz: 1.0 });
                            let earth = Material::LambertianTexture(LambertianTexture {
                                texture: Texture::Image(TexImage::ToLoad {
                                    path:   PathBuf::from("tests/scenes/earthmap.jpg"),
                                    format: Some(image::ImageFormat::Jpeg),
                                }),
                            });
                            let perlin_wink = Material::Lambertian(Lambertian { colour: Colour::new(0.5, 0.5, 0.5, 1.0) });
                            let white = Material::Lambertian(Lambertian { colour: Colour::new(0.73, 0.73, 0.73, 1.0) });

                            // Define the ground
                            let mut objects: Vec<Object> = Vec::with_capacity(1000);
                            const BOXES_PER_SIDE: u32 = 20;
                            for i in 0..BOXES_PER_SIDE {
                                for j in 0..BOXES_PER_SIDE {
                                    // Compute the dimensions of each box
                                    let w = 100.0;
                                    let x0 = -1000.0 + i as f64 * w;
                                    let z0 = -1000.0 + j as f64 * w;
                                    let y0 = 0.0;
                                    let x1 = x0 + w;
                                    let y1 = fastrand::f64() * 100.0 + 1.0;
                                    let z1 = z0 + w;
                                    objects.push(Object::Box(Box {
                                        aabb:     AABB::from_points(Vec3::new(x0, y0, z0), Vec3::new(x1, y1, z1)),
                                        material: ground.clone(),
                                    }));
                                }
                            }

                            // Define the ceiling light
                            objects.push(Object::Quad(Quad {
                                qd: Qd { pos: Vec3::new(123.0, 554.0, 147.0), u: Vec3::new(300.0, 0.0, 0.0), v: Vec3::new(0.0, 0.0, 265.0) },
                                material: light,
                            }));

                            // Define the blurry sphere
                            objects.push(Object::AnimatedSphere(AnimatedSphere {
                                sphere:    Sphere { center: Vec3::new(400.0, 400.0, 200.0), radius: 50.0, material: brown },
                                animation: Animation::Vertical(Vertical { len: 30.0, at: 0, duration: cover.shutter_time }),
                            }));

                            // Define the loose glass & metal spheres
                            objects.push(Object::Sphere(Sphere { center: Vec3::new(260.0, 150.0, 45.0), radius: 50.0, material: glass.clone() }));
                            objects.push(Object::Sphere(Sphere { center: Vec3::new(0.0, 150.0, 145.0), radius: 50.0, material: grey_metal }));

                            // Define glossy sphere (a dense fog in a glass sphere)
                            let boundary =
                                Object::Sphere(Sphere { center: Vec3::new(360.0, 150.0, 145.0), radius: 70.0, material: glass.clone() });
                            objects.push(boundary.clone());
                            objects.push(Object::ConstantDensity(ConstantDensity {
                                boundary: std::boxed::Box::new(boundary),
                                density: 0.2,
                                phase_function: Isotropic { colour: Colour::new(0.2, 0.4, 0.9, 1.0) },
                            }));

                            // Define the overall haze over the scene
                            objects.push(Object::ConstantDensity(ConstantDensity {
                                boundary: std::boxed::Box::new(Object::Sphere(Sphere {
                                    center:   Vec3::new(0.0, 0.0, 0.0),
                                    radius:   5000.0,
                                    material: glass,
                                })),
                                density: 0.0001,
                                phase_function: Isotropic { colour: Colour::new(1.0, 1.0, 1.0, 1.0) },
                            }));

                            // Define the earthy sphere and perlin noise sphere (although we just use a blank lambertian sphere)
                            objects.push(Object::Sphere(Sphere { center: Vec3::new(400.0, 200.0, 400.0), radius: 100.0, material: earth }));
                            objects.push(Object::Sphere(Sphere { center: Vec3::new(220.0, 280.0, 300.0), radius: 80.0, material: perlin_wink }));

                            // Define the box made out of spheres
                            const NUMBER_OF_SPHERES: usize = 1000;
                            let mut orbs = Vec::with_capacity(NUMBER_OF_SPHERES);
                            for _ in 0..NUMBER_OF_SPHERES {
                                orbs.push(Object::Sphere(Sphere {
                                    center:   Vec3::new(fastrand::f64() * 165.0, fastrand::f64() * 165.0, fastrand::f64() * 165.0),
                                    radius:   10.0,
                                    material: white.clone(),
                                }));
                            }
                            objects.push(Object::Translate(Translate {
                                pos: Vec3::new(-100.0, 270.0, 395.0),
                                obj: std::boxed::Box::new(Object::RotateY(RotateY {
                                    angle: 15.0,
                                    obj:   std::boxed::Box::new(Object::Group(std::boxed::Box::new(HitTree::with_objs(
                                        orbs,
                                        (0..=cover.shutter_time).into(),
                                    )))),
                                })),
                            }));

                            // Done
                            objects
                        },
                    };

                    // Ensure to load all
                    for (i, obj) in objects.iter_mut().enumerate() {
                        if let Err(err) = obj.load(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))) {
                            error!("{}", toplevel!(("Failed to load external references in object {i}"), err));
                            return ExitCode::FAILURE;
                        }
                    }

                    // Convert that to a static HitList
                    let list: HitTree = HitTree::with_objs(objects, (0..=cover.shutter_time).into());
                    let dims: (u32, u32) = if let Some(dims) = render.dims { (dims.0.into(), dims.1.into()) } else { (800, 600) };
                    let cam = match cover.book {
                        Book::OneWeekend => Camera::new(
                            dims,
                            100,
                            20.0,
                            0.6,
                            10.0,
                            cover.shutter_time,
                            Vec3::new(13.0, 2.0, 3.0),
                            Vec3::new(0.0, 0.0, 0.0),
                            Vec3::new(0.0, 1.0, 0.0),
                        ),
                        Book::NextWeek => Camera::new(
                            dims,
                            5000,
                            40.0,
                            0.0,
                            0.0,
                            cover.shutter_time,
                            Vec3::new(478.0, 278.0, -600.0),
                            Vec3::new(278.0, 278.0, 0.0),
                            Vec3::new(0.0, 1.0, 0.0),
                        ),
                    };
                    let env = match cover.book {
                        Book::OneWeekend => Environment::default(),
                        Book::NextWeek => Environment { background: Background::None, ..Default::default() },
                    };

                    // Now render based on the backend
                    let output: Image = match render.backend {
                        RenderBackend::SingleThreaded => {
                            debug!("Rendering with single-threaded backend");
                            let renderer: SingleThreadRenderer =
                                SingleThreadRenderer::new(true, render.ray_max_depth.unwrap_or(50), !render.disable_gamma_correction);
                            renderer.render_frame(&list, &cam, &env).unwrap()
                        },

                        RenderBackend::MultiThreaded => {
                            debug!("Rendering with multi-threaded backend");

                            // Read the given file, if any
                            let config: MultiThreadRendererConfig = match render.backend_config {
                                Some(path) => {
                                    debug!("Loading multi-threaded backend file '{}'...", path.display());
                                    match MultiThreadRendererConfig::from_path(path) {
                                        Ok(config) => config,
                                        Err(err) => {
                                            error!("{}", err.trace());
                                            return ExitCode::FAILURE;
                                        },
                                    }
                                },
                                None => Default::default(),
                            };

                            // Create the backend
                            let renderer: MultiThreadRenderer =
                                match MultiThreadRenderer::new(true, render.ray_max_depth.unwrap_or(50), !render.disable_gamma_correction, config) {
                                    Ok(renderer) => renderer,
                                    Err(err) => {
                                        error!("{}", err.trace());
                                        return ExitCode::FAILURE;
                                    },
                                };

                            // Now render with this backend
                            renderer.render_frame(&list, &cam, &env).unwrap()
                        },
                    };

                    // Now write the image to disk
                    if let Err(err) = output.to_path(&cover.output_path, render.fix_dirs) {
                        error!("Failed to save rendered image to '{}': {}", cover.output_path.display(), err);
                        return ExitCode::FAILURE;
                    }
                    ExitCode::SUCCESS
                },
            }
        },

        RaytracerSubcommand::Generate(generate) => {
            // Further match
            match generate.subcommand {
                GenerateSubcommand::Gradient { path, dims } => {
                    // Run the command
                    if let Err(err) = generate::gradient(path, (dims.0.into(), dims.1.into()), generate.fix_dirs) {
                        error!("{}", err.trace());
                        return ExitCode::FAILURE;
                    }
                    ExitCode::SUCCESS
                },
            }
        },
    }
}
