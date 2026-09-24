# Changelog
This file keeps track of the changes done for every raytracer version.

[Semantic versioning](https://semver.org) is used for the version numbers. As such, breaking
changes are indicated with **(BREAKING)**.


## 1.0.0 - TODO
Finished the book series - introducing a relatively mature raytracer.

### Added
- A multi-threaded backend.
- Support for more objects (Box, Quad, Triangle).
- Support for more materials (Dieletric, DiffuseLight, Lambertian, LambertianTexture, Metal,
  PartialDielectric, StaticColour).
- Support for transformations and animations (AnimatedTranslate, RotateX, RotateY, RotateZ,
  Translate).
- Support for volumetric objects (ConstantDensity).
- Support for motion blur.
- Support for lights.
- Support for external models.
- Support for (external) textures.
- Support for different kinds of backgrounds (Colour, IlluminatedSky, None).

### Changed
- All scene files now use JSON instead of YAML. **(BREAKING)**

### Removed
- The features file. **(BREAKING)**


## 0.2.0 - 2023-05-06
### Added
- Anti-aliasing support, or rather, shooting multiple rays per pixel.
- Support for scattering rays over different material types.
  - The `!NormalMap` material type, which represents the pre-0.2.0 default material
  - The `!Diffuse` material type, which implements a hacky-but-diffuse-y lambartian.
- Support for enabling/disabling features at will based on CLI-arguments and a `features.yml` file.
- A `RayGenerator` class that takes care of generating rays.
- A `HitList` class that we use to support multiple (types of) objects.
- Gamma correction.
- A progress bar to indicate status.

### Changed
- `rand` to instead `fastrand`, for much better performance (since we don't need cryptographically
  secure random anyway).


## 0.1.0 - 2023-05
### Added
- A raytracer that can trace rays (but not scatter them).
