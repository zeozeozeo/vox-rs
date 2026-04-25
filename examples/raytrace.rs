use std::env;
use std::fmt::Write as _;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use vox_rs::{Rgba, Scene, Transform};

const DEFAULT_WIDTH: usize = 48;
const DEFAULT_HEIGHT: usize = 24;
const CAMERA_FOV_Y_DEG: f32 = 45.0;
const CAMERA_PADDING: f32 = 1.25;
const EPSILON: f32 = 1e-3;

#[derive(Clone, Copy, Debug, Default)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    fn normalize(self) -> Self {
        let length = self.length();
        if length == 0.0 {
            Self::zero()
        } else {
            self.mul(1.0 / length)
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Bounds {
    min: Vec3,
    max: Vec3,
}

impl Bounds {
    fn empty() -> Self {
        Self {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    fn include(&mut self, point: Vec3) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }

    fn center(&self) -> Vec3 {
        self.min.add(self.max).mul(0.5)
    }

    fn size(&self) -> Vec3 {
        self.max.sub(self.min)
    }

    fn radius(&self) -> f32 {
        self.size().length() * 0.5
    }
}

#[derive(Clone, Copy, Debug)]
struct RenderInstance {
    transform: Transform,
    model_index: usize,
    bounds: Bounds,
}

#[derive(Clone, Copy, Debug)]
struct ViewSpec {
    label: &'static str,
    direction: Vec3,
    up: Vec3,
}

#[derive(Clone, Copy, Debug)]
struct CameraFrame {
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    aspect: f32,
    tan_half_fov: f32,
}

const VIEWS: [ViewSpec; 6] = [
    ViewSpec {
        label: "front",
        direction: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        up: Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    },
    ViewSpec {
        label: "right",
        direction: Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        up: Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    },
    ViewSpec {
        label: "back",
        direction: Vec3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        },
        up: Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    },
    ViewSpec {
        label: "left",
        direction: Vec3 {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        },
        up: Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    },
    ViewSpec {
        label: "top",
        direction: Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        up: Vec3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        },
    },
    ViewSpec {
        label: "bottom",
        direction: Vec3 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        },
        up: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    },
];

fn print_help(program: &str) {
    println!("raytrace");
    println!("usage:");
    println!("  {program} <input.vox> [width] [height]");
    println!();
    println!("defaults:");
    println!("  input  = testdata/chr_knight.vox");
    println!("  width  = {DEFAULT_WIDTH}");
    println!("  height = {DEFAULT_HEIGHT}");
}

fn parse_usize_arg(value: Option<std::ffi::OsString>, default: usize) -> Result<usize, String> {
    let Some(value) = value else {
        return Ok(default);
    };
    let value = value
        .into_string()
        .map_err(|_| "arguments must be valid UTF-8".to_owned())?;
    value
        .parse::<usize>()
        .map_err(|_| format!("invalid numeric argument: {value}"))
}

fn load_scene(path: &Path) -> Result<Scene, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    Ok(Scene::read(&mut reader)?)
}

fn transform_point(transform: Transform, point: Vec3) -> Vec3 {
    Vec3::new(
        point.x * transform.m00 + point.y * transform.m10 + point.z * transform.m20 + transform.m30,
        point.x * transform.m01 + point.y * transform.m11 + point.z * transform.m21 + transform.m31,
        point.x * transform.m02 + point.y * transform.m12 + point.z * transform.m22 + transform.m32,
    )
}

fn inverse_transform_point(transform: Transform, point: Vec3) -> Vec3 {
    let offset = point.sub(Vec3::new(transform.m30, transform.m31, transform.m32));
    Vec3::new(
        offset.x * transform.m00 + offset.y * transform.m01 + offset.z * transform.m02,
        offset.x * transform.m10 + offset.y * transform.m11 + offset.z * transform.m12,
        offset.x * transform.m20 + offset.y * transform.m21 + offset.z * transform.m22,
    )
}

fn inverse_transform_vector(transform: Transform, vector: Vec3) -> Vec3 {
    Vec3::new(
        vector.x * transform.m00 + vector.y * transform.m01 + vector.z * transform.m02,
        vector.x * transform.m10 + vector.y * transform.m11 + vector.z * transform.m12,
        vector.x * transform.m20 + vector.y * transform.m21 + vector.z * transform.m22,
    )
}

fn shade_color(color: Rgba, normal: Vec3) -> Rgba {
    let light_dir = Vec3::new(0.35, 0.9, 0.25).normalize();
    let diffuse = normal.dot(light_dir).max(0.0);
    let factor = 0.72 + diffuse * 0.28;
    Rgba {
        r: ((color.r as f32 * factor).round().clamp(0.0, 255.0)) as u8,
        g: ((color.g as f32 * factor).round().clamp(0.0, 255.0)) as u8,
        b: ((color.b as f32 * factor).round().clamp(0.0, 255.0)) as u8,
        a: color.a,
    }
}

fn append_color_escape(out: &mut String, prefix: &str, color: Option<Rgba>) {
    let color = color.unwrap_or(Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    });
    let _ = write!(out, "\x1b[{prefix};2;{};{};{}m", color.r, color.g, color.b);
}

fn append_cell(out: &mut String, top: Option<Rgba>, bottom: Option<Rgba>) {
    if top.is_none() && bottom.is_none() {
        out.push_str("\x1b[0m ");
        return;
    }

    append_color_escape(out, "38", top);
    append_color_escape(out, "48", bottom);
    out.push('▀');
}

fn apply_view_basis(direction: Vec3, up: Vec3, center: Vec3, distance: f32) -> (Vec3, Vec3, Vec3) {
    let eye = center.add(direction.mul(distance));
    let forward = center.sub(eye).normalize();
    let right = up.cross(forward).normalize();
    let camera_up = forward.cross(right).normalize();
    (eye, right, camera_up)
}

fn fit_camera_distance(bounds: &Bounds, width: usize, height: usize) -> f32 {
    let radius = bounds.radius().max(1.0);
    let pixel_aspect = width as f32 / (height as f32 * 2.0);
    let vertical_scale = 1.0 / (CAMERA_FOV_Y_DEG.to_radians() * 0.5).sin();
    let horizontal_scale = pixel_aspect.max(1.0);
    radius * CAMERA_PADDING * vertical_scale * horizontal_scale
}

fn scene_render_instances(
    scene: &Scene,
) -> Result<(Vec<RenderInstance>, Bounds), Box<dyn std::error::Error>> {
    let mut instances = Vec::new();
    let mut bounds = Bounds::empty();

    for (index, instance) in scene.instances.iter().enumerate() {
        let transform = scene.sample_instance_transform_global(index, 0)?;
        let model_index = instance.sample_model_index(0);
        let model = &scene.models[model_index];
        if model.solid_voxel_count() == 0 {
            continue;
        }

        let model_bounds = model_world_bounds(transform, model.size_x, model.size_y, model.size_z);
        bounds.include(model_bounds.min);
        bounds.include(model_bounds.max);
        instances.push(RenderInstance {
            transform,
            model_index,
            bounds: model_bounds,
        });
    }

    Ok((instances, bounds))
}

fn model_world_bounds(transform: Transform, size_x: u32, size_y: u32, size_z: u32) -> Bounds {
    let mut bounds = Bounds::empty();
    let corners = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(size_x as f32, 0.0, 0.0),
        Vec3::new(0.0, size_y as f32, 0.0),
        Vec3::new(0.0, 0.0, size_z as f32),
        Vec3::new(size_x as f32, size_y as f32, 0.0),
        Vec3::new(size_x as f32, 0.0, size_z as f32),
        Vec3::new(0.0, size_y as f32, size_z as f32),
        Vec3::new(size_x as f32, size_y as f32, size_z as f32),
    ];
    for corner in corners {
        bounds.include(transform_point(transform, corner));
    }
    bounds
}

fn intersect_aabb(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<(f32, f32)> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    for axis in 0..3 {
        let (origin_component, dir_component, min_component, max_component) = match axis {
            0 => (origin.x, dir.x, min.x, max.x),
            1 => (origin.y, dir.y, min.y, max.y),
            _ => (origin.z, dir.z, min.z, max.z),
        };

        if dir_component == 0.0 {
            if origin_component < min_component || origin_component > max_component {
                return None;
            }
            continue;
        }

        let inv_dir = 1.0 / dir_component;
        let mut t0 = (min_component - origin_component) * inv_dir;
        let mut t1 = (max_component - origin_component) * inv_dir;
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }
        t_min = t_min.max(t0);
        t_max = t_max.min(t1);
        if t_max < t_min {
            return None;
        }
    }

    Some((t_min, t_max))
}

fn trace_model(
    origin_world: Vec3,
    dir_world: Vec3,
    instance: &RenderInstance,
    model: &vox_rs::Model,
) -> Option<(f32, u8, Vec3)> {
    let origin = inverse_transform_point(instance.transform, origin_world);
    let dir = inverse_transform_vector(instance.transform, dir_world);
    let local_bounds_min = Vec3::zero();
    let local_bounds_max = Vec3::new(
        model.size_x as f32,
        model.size_y as f32,
        model.size_z as f32,
    );
    let (entry, exit) = intersect_aabb(origin, dir, local_bounds_min, local_bounds_max)?;
    let start_t = entry.max(0.0);
    if start_t > exit {
        return None;
    }

    let position = origin.add(dir.mul(start_t + EPSILON));
    let mut x = position.x.floor() as i32;
    let mut y = position.y.floor() as i32;
    let mut z = position.z.floor() as i32;

    let step_x = if dir.x > 0.0 {
        1
    } else if dir.x < 0.0 {
        -1
    } else {
        0
    };
    let step_y = if dir.y > 0.0 {
        1
    } else if dir.y < 0.0 {
        -1
    } else {
        0
    };
    let step_z = if dir.z > 0.0 {
        1
    } else if dir.z < 0.0 {
        -1
    } else {
        0
    };

    let mut t_max_x = if step_x == 0 {
        f32::INFINITY
    } else if step_x > 0 {
        (x as f32 + 1.0 - position.x) / dir.x
    } else {
        (x as f32 - position.x) / dir.x
    };
    let mut t_max_y = if step_y == 0 {
        f32::INFINITY
    } else if step_y > 0 {
        (y as f32 + 1.0 - position.y) / dir.y
    } else {
        (y as f32 - position.y) / dir.y
    };
    let mut t_max_z = if step_z == 0 {
        f32::INFINITY
    } else if step_z > 0 {
        (z as f32 + 1.0 - position.z) / dir.z
    } else {
        (z as f32 - position.z) / dir.z
    };

    let t_delta_x = if step_x == 0 {
        f32::INFINITY
    } else {
        1.0 / dir.x.abs()
    };
    let t_delta_y = if step_y == 0 {
        f32::INFINITY
    } else {
        1.0 / dir.y.abs()
    };
    let t_delta_z = if step_z == 0 {
        f32::INFINITY
    } else {
        1.0 / dir.z.abs()
    };

    let size_x = model.size_x as i32;
    let size_y = model.size_y as i32;
    let size_z = model.size_z as i32;
    let mut current_t = start_t;
    let mut normal = Vec3::zero();

    loop {
        if x < 0 || y < 0 || z < 0 || x >= size_x || y >= size_y || z >= size_z {
            return None;
        }

        let index = x as usize
            + y as usize * model.size_x as usize
            + z as usize * model.size_x as usize * model.size_y as usize;
        let color_index = model.voxels[index];
        if color_index != 0 {
            return Some((current_t, color_index, normal));
        }

        if t_max_x < t_max_y && t_max_x < t_max_z {
            x += step_x;
            current_t = t_max_x;
            t_max_x += t_delta_x;
            normal = if step_x > 0 {
                Vec3::new(-1.0, 0.0, 0.0)
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            };
        } else if t_max_y < t_max_z {
            y += step_y;
            current_t = t_max_y;
            t_max_y += t_delta_y;
            normal = if step_y > 0 {
                Vec3::new(0.0, -1.0, 0.0)
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            };
        } else {
            z += step_z;
            current_t = t_max_z;
            t_max_z += t_delta_z;
            normal = if step_z > 0 {
                Vec3::new(0.0, 0.0, -1.0)
            } else {
                Vec3::new(0.0, 0.0, 1.0)
            };
        }
    }
}

fn trace_scene(
    scene: &Scene,
    instances: &[RenderInstance],
    origin: Vec3,
    dir: Vec3,
) -> Option<Rgba> {
    let mut best_t = f32::INFINITY;
    let mut best_color = None;

    for instance in instances {
        let model = &scene.models[instance.model_index];
        if let Some((entry, _exit)) =
            intersect_aabb(origin, dir, instance.bounds.min, instance.bounds.max)
        {
            if entry >= best_t {
                continue;
            }
        } else {
            continue;
        }

        if let Some((t, color_index, normal)) = trace_model(origin, dir, instance, model) {
            let palette_color = scene.palette.colors[color_index as usize];
            if palette_color.a == 0 {
                continue;
            }
            if t < best_t {
                best_t = t;
                best_color = Some(shade_color(palette_color, normal));
            }
        }
    }

    best_color
}

fn render_view(
    scene: &Scene,
    instances: &[RenderInstance],
    bounds: &Bounds,
    width: usize,
    height: usize,
    view: ViewSpec,
) -> String {
    let center = bounds.center();
    let distance = fit_camera_distance(bounds, width, height);
    let (eye, right, camera_up) = apply_view_basis(view.direction, view.up, center, distance);
    let forward = center.sub(eye).normalize();
    let pixel_height = height * 2;
    let aspect = width as f32 / pixel_height as f32;
    let camera = CameraFrame {
        eye,
        forward,
        right,
        up: camera_up,
        aspect,
        tan_half_fov: (CAMERA_FOV_Y_DEG.to_radians() * 0.5).tan(),
    };
    let mut out = String::with_capacity(width * height * 48 + height * 16);

    for row in 0..height {
        let top_row = row * 2;
        let bottom_row = top_row + 1;
        for column in 0..width {
            let top = sample_pixel(
                scene,
                instances,
                &camera,
                column,
                top_row,
                width,
                pixel_height,
            );
            let bottom = sample_pixel(
                scene,
                instances,
                &camera,
                column,
                bottom_row,
                width,
                pixel_height,
            );
            append_cell(&mut out, top, bottom);
        }
        out.push_str("\x1b[0m\n");
    }

    out
}

fn sample_pixel(
    scene: &Scene,
    instances: &[RenderInstance],
    camera: &CameraFrame,
    column: usize,
    row: usize,
    width: usize,
    pixel_height: usize,
) -> Option<Rgba> {
    let u =
        ((column as f32 + 0.5) / width as f32 * 2.0 - 1.0) * camera.aspect * camera.tan_half_fov;
    let v = (1.0 - (row as f32 + 0.5) / pixel_height as f32 * 2.0) * camera.tan_half_fov;
    let dir = camera
        .forward
        .add(camera.right.mul(u))
        .add(camera.up.mul(v))
        .normalize();
    trace_scene(scene, instances, camera.eye, dir)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os();
    let program = args
        .next()
        .and_then(|value| Path::new(&value).file_name().map(|value| value.to_owned()))
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| "raytrace".to_owned());

    let input = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new("testdata").join("chr_knight.vox"));
    let width = parse_usize_arg(args.next(), DEFAULT_WIDTH)
        .map_err(|error| format!("{program}: {error}"))?;
    let height = parse_usize_arg(args.next(), DEFAULT_HEIGHT)
        .map_err(|error| format!("{program}: {error}"))?;

    if args.next().is_some() {
        print_help(&program);
        return Err(format!("{program}: too many arguments").into());
    }
    if width == 0 || height == 0 {
        return Err(format!("{program}: width and height must be greater than zero").into());
    }

    let scene = load_scene(&input)?;
    let (instances, bounds) = scene_render_instances(&scene)?;
    if instances.is_empty() {
        return Err(format!("{program}: scene contains no visible voxels to render").into());
    }

    println!(
        "rendering {} instance(s) from {} using {}x{} terminal cells",
        instances.len(),
        input.display(),
        width,
        height
    );

    for (index, view) in VIEWS.iter().enumerate() {
        println!();
        println!("=== {} ({}/{}) ===", view.label, index + 1, VIEWS.len());
        let image = render_view(&scene, &instances, &bounds, width, height, *view);
        print!("{image}");
    }

    print!("\x1b[0m");
    Ok(())
}
