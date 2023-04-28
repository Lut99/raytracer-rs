# raytracer-rs
A new attempt at writing a simple raytracer. This time, no fumbling about with real-time stuff, but instead creating a good-old, offline renderer. Based on <https://raytracing.github.io/books/RayTracingInOneWeekend.html>.


## Installation
To get started with this project, first make sure that you install [Rust](https://rust.org). The easiest is to download it using rustup, [here](https://rustup.rs/) (don't forget to process the new environment file).

Once downloaded, you can then clone this repository:
```bash
git clone https://github.com/Lut99/raytracer-rs ./raytracer
cd raytracer
```

The project can then be compiled using [Cargo](https://crates.io/):
```bash
cargo build --release
```
and executed using:
```bash
# Unix
./target/release/raytracer
```
```bash
# Windows
./target/release/raytracer.exe
```


## Usage

### Command-Line Interface
The `raytracer` executable has various features. Currently, these are them:
- `raytracer render <file>` renders a scene defined in a _scene file_ (see [below](#scene-files)) to an image. There are some additional options available, use `raytracer render --help` to see them.
- `raytracer generate` is the subcommand that groups the generation of various files. The following sub-subcommands are supported:
  - `raytracer generate gradient`: Generates the [example gradient image](https://raytracing.github.io/books/RayTracingInOneWeekend.html#outputanimage/creatinganimagefile) from the tutorial we are using.

### Scene files
To describe a scene to render, we use our own scene file format. It is written in [YAML](https://yaml.org), and knows of the following fields:
- `objects`: Describes a list of objects to render, as a vector. The following objects can be chosen:
  - `sphere`: Renders a perfect sphere. It has a `center` option, which takes a list of three coordinates (X, Y, Z), and a `radius` option, which determines its radius. For example:
    ```yaml
    objects:
    - !sphere
      center: [ 0, 0, -1 ]
      radius: 0.5
    ```

For examples of scene files, check the [`tests/scenes`](./tests/scenes/) directory.


## Contribution
Note that this is mostly a hobby project for myself, not meant for distribution or serious use. That said, if you like to contribute to this project or use it for something, feel free to let me know by dropping an [issue](https://gihub.com/Lut99/raytracer-rs/issues) or creating a [pull request](https://github.com/Lut99/raytracer-rs/pulls).


## License
This project is licensed under GPLv3. See [`LICENSE`](./LICENSE) for more information.
