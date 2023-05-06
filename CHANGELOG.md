# Changelog
This file keeps track of the changes done for every raytracer version.


## 0.2.0 - Diffuse material [06-05-2023]
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
- `rand` to instead `fastrand`, for much better performance (since we don't need cryptographically secure random anyway).


## 0.1.0 - Initial release [05-2023]
### Added
- A raytracer that can trace rays (but not scatter them).
