use alloc::string::String;
use alloc::vec::Vec;
use core::array;
use core::error::Error;
use core::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m03: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,
    pub m20: f32,
    pub m21: f32,
    pub m22: f32,
    pub m23: f32,
    pub m30: f32,
    pub m31: f32,
    pub m32: f32,
    pub m33: f32,
}

impl Transform {
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            m00: 1.0,
            m01: 0.0,
            m02: 0.0,
            m03: 0.0,
            m10: 0.0,
            m11: 1.0,
            m12: 0.0,
            m13: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 0.0,
            m30: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
        }
    }

    #[must_use]
    pub fn multiply(self, other: Self) -> Self {
        Self {
            m00: (self.m00 * other.m00)
                + (self.m01 * other.m10)
                + (self.m02 * other.m20)
                + (self.m03 * other.m30),
            m01: (self.m00 * other.m01)
                + (self.m01 * other.m11)
                + (self.m02 * other.m21)
                + (self.m03 * other.m31),
            m02: (self.m00 * other.m02)
                + (self.m01 * other.m12)
                + (self.m02 * other.m22)
                + (self.m03 * other.m32),
            m03: (self.m00 * other.m03)
                + (self.m01 * other.m13)
                + (self.m02 * other.m23)
                + (self.m03 * other.m33),
            m10: (self.m10 * other.m00)
                + (self.m11 * other.m10)
                + (self.m12 * other.m20)
                + (self.m13 * other.m30),
            m11: (self.m10 * other.m01)
                + (self.m11 * other.m11)
                + (self.m12 * other.m21)
                + (self.m13 * other.m31),
            m12: (self.m10 * other.m02)
                + (self.m11 * other.m12)
                + (self.m12 * other.m22)
                + (self.m13 * other.m32),
            m13: (self.m10 * other.m03)
                + (self.m11 * other.m13)
                + (self.m12 * other.m23)
                + (self.m13 * other.m33),
            m20: (self.m20 * other.m00)
                + (self.m21 * other.m10)
                + (self.m22 * other.m20)
                + (self.m23 * other.m30),
            m21: (self.m20 * other.m01)
                + (self.m21 * other.m11)
                + (self.m22 * other.m21)
                + (self.m23 * other.m31),
            m22: (self.m20 * other.m02)
                + (self.m21 * other.m12)
                + (self.m22 * other.m22)
                + (self.m23 * other.m32),
            m23: (self.m20 * other.m03)
                + (self.m21 * other.m13)
                + (self.m22 * other.m23)
                + (self.m23 * other.m33),
            m30: (self.m30 * other.m00)
                + (self.m31 * other.m10)
                + (self.m32 * other.m20)
                + (self.m33 * other.m30),
            m31: (self.m30 * other.m01)
                + (self.m31 * other.m11)
                + (self.m32 * other.m21)
                + (self.m33 * other.m31),
            m32: (self.m30 * other.m02)
                + (self.m31 * other.m12)
                + (self.m32 * other.m22)
                + (self.m33 * other.m32),
            m33: (self.m30 * other.m03)
                + (self.m31 * other.m13)
                + (self.m32 * other.m23)
                + (self.m33 * other.m33),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    pub colors: [Rgba; 256],
}

impl Palette {
    #[must_use]
    pub fn raw_default() -> Self {
        let mut colors = [Rgba::default(); 256];
        for (index, chunk) in DEFAULT_PALETTE_RAW.chunks_exact(4).enumerate() {
            colors[index] = Rgba {
                r: chunk[0],
                g: chunk[1],
                b: chunk[2],
                a: chunk[3],
            };
        }
        Self { colors }
    }

    #[must_use]
    pub fn default_scene_palette() -> Self {
        let mut palette = Self::raw_default();
        let last = palette.colors[255];
        for index in (1..256).rev() {
            palette.colors[index] = palette.colors[index - 1];
        }
        palette.colors[0] = last;
        palette.colors[0].a = 0;
        palette
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::default_scene_palette()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MaterialType {
    #[default]
    Diffuse,
    Metal,
    Glass,
    Emit,
    Blend,
    Media,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MediaType {
    #[default]
    Absorb,
    Scatter,
    Emit,
    Sss,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Material {
    pub material_type: MaterialType,
    pub media_type: MediaType,
    pub metal: Option<f32>,
    pub rough: Option<f32>,
    pub spec: Option<f32>,
    pub ior: Option<f32>,
    pub att: Option<f32>,
    pub flux: Option<f32>,
    pub emit: Option<f32>,
    pub ldr: Option<f32>,
    pub trans: Option<f32>,
    pub alpha: Option<f32>,
    pub d: Option<f32>,
    pub sp: Option<f32>,
    pub g: Option<f32>,
    pub media: Option<f32>,
}

impl Material {
    #[must_use]
    pub fn has_content(&self) -> bool {
        self.metal.is_some()
            || self.rough.is_some()
            || self.spec.is_some()
            || self.ior.is_some()
            || self.att.is_some()
            || self.flux.is_some()
            || self.emit.is_some()
            || self.ldr.is_some()
            || self.trans.is_some()
            || self.alpha.is_some()
            || self.d.is_some()
            || self.sp.is_some()
            || self.g.is_some()
            || self.media.is_some()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CameraMode {
    #[default]
    Perspective,
    Free,
    Pano,
    Orthographic,
    Isometric,
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    pub camera_id: u32,
    pub mode: CameraMode,
    pub focus: [f32; 3],
    pub angle: [f32; 3],
    pub radius: f32,
    pub frustum: f32,
    pub fov: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sun {
    pub intensity: f32,
    pub area: f32,
    pub angle: [f32; 2],
    pub rgba: Rgba,
    pub disk: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
    pub voxels: Vec<u8>,
}

impl Model {
    pub fn voxel_count(&self) -> Result<usize, VoxError> {
        let size_x = usize::try_from(self.size_x)
            .map_err(|_| VoxError::InvalidData("model size_x does not fit usize".into()))?;
        let size_y = usize::try_from(self.size_y)
            .map_err(|_| VoxError::InvalidData("model size_y does not fit usize".into()))?;
        let size_z = usize::try_from(self.size_z)
            .map_err(|_| VoxError::InvalidData("model size_z does not fit usize".into()))?;
        size_x
            .checked_mul(size_y)
            .and_then(|value| value.checked_mul(size_z))
            .ok_or_else(|| VoxError::InvalidData("model voxel count overflow".into()))
    }

    #[must_use]
    pub fn solid_voxel_count(&self) -> usize {
        self.voxels.iter().filter(|&&value| value != 0).count()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyframeTransform {
    pub frame_index: u32,
    pub transform: Transform,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyframeModel {
    pub frame_index: u32,
    pub model_index: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimTransform {
    pub keyframes: Vec<KeyframeTransform>,
    pub looped: bool,
}

impl AnimTransform {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnimModel {
    pub keyframes: Vec<KeyframeModel>,
    pub looped: bool,
}

impl AnimModel {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instance {
    pub name: Option<String>,
    pub transform: Transform,
    pub model_index: usize,
    pub layer_index: Option<usize>,
    pub group_index: Option<usize>,
    pub hidden: bool,
    pub transform_anim: AnimTransform,
    pub model_anim: AnimModel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer {
    pub name: Option<String>,
    pub color: Rgba,
    pub hidden: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub name: Option<String>,
    pub transform: Transform,
    pub parent_group_index: Option<usize>,
    pub layer_index: Option<usize>,
    pub hidden: bool,
    pub transform_anim: AnimTransform,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub file_version: u32,
    pub models: Vec<Model>,
    pub instances: Vec<Instance>,
    pub layers: Vec<Layer>,
    pub groups: Vec<Group>,
    pub color_names: Vec<String>,
    pub palette: Palette,
    pub materials: [Material; 256],
    pub cameras: Vec<Camera>,
    pub sun: Option<Sun>,
    pub anim_range_start: u32,
    pub anim_range_end: u32,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            file_version: 150,
            models: Vec::new(),
            instances: Vec::new(),
            layers: Vec::new(),
            groups: Vec::new(),
            color_names: Vec::new(),
            palette: Palette::default(),
            materials: array::from_fn(|_| Material::default()),
            cameras: Vec::new(),
            sun: None,
            anim_range_start: 0,
            anim_range_end: 30,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadOptions {
    pub preserve_groups: bool,
    pub preserve_keyframes: bool,
    pub keep_empty_models_instances: bool,
    pub keep_duplicate_models: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VoxError {
    UnexpectedEof,
    InvalidHeader,
    UnsupportedVersion(u32),
    InvalidChunk(&'static str),
    InvalidNodeGraph(&'static str),
    InvalidData(String),
    IndexOutOfBounds {
        kind: &'static str,
        index: usize,
        len: usize,
    },
    TooManyRequiredColors(usize),
    WriteCancelled,
    FileTooLarge,
    #[cfg(feature = "std")]
    IoErrorKind(std::io::ErrorKind),
}

impl fmt::Display for VoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("unexpected end of file"),
            Self::InvalidHeader => f.write_str("invalid .vox header"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported .vox version {version}")
            }
            Self::InvalidChunk(name) => write!(f, "invalid chunk {name}"),
            Self::InvalidNodeGraph(message) => f.write_str(message),
            Self::InvalidData(message) => f.write_str(message),
            Self::IndexOutOfBounds { kind, index, len } => {
                write!(f, "{kind} index {index} is out of bounds for length {len}")
            }
            Self::TooManyRequiredColors(count) => {
                write!(f, "required color count {count} exceeds 255")
            }
            Self::WriteCancelled => f.write_str("scene write cancelled"),
            Self::FileTooLarge => f.write_str("generated file exceeds 4 GiB"),
            #[cfg(feature = "std")]
            Self::IoErrorKind(kind) => write!(f, "i/o error while writing scene: {kind}"),
        }
    }
}

impl Error for VoxError {}

impl Scene {
    pub fn read(bytes: &[u8]) -> Result<Self, VoxError> {
        crate::codec::read_scene(bytes, ReadOptions::default())
    }

    pub fn read_with_options(bytes: &[u8], options: ReadOptions) -> Result<Self, VoxError> {
        crate::codec::read_scene(bytes, options)
    }

    pub fn write(&self) -> Result<Vec<u8>, VoxError> {
        crate::codec::write_scene_with_progress(self, |_| true)
    }

    pub fn write_with_progress<F>(&self, progress: F) -> Result<Vec<u8>, VoxError>
    where
        F: FnMut(f32) -> bool,
    {
        crate::codec::write_scene_with_progress(self, progress)
    }

    #[cfg(feature = "std")]
    pub fn write_to<W>(&self, writer: &mut W) -> Result<(), VoxError>
    where
        W: std::io::Write,
    {
        crate::codec::write_scene_to_writer(self, writer, |_| true)
    }

    #[cfg(feature = "std")]
    pub fn write_to_with_progress<W, F>(&self, writer: &mut W, progress: F) -> Result<(), VoxError>
    where
        W: std::io::Write,
        F: FnMut(f32) -> bool,
    {
        crate::codec::write_scene_to_writer(self, writer, progress)
    }

    pub fn merge(scenes: &[&Scene], required_colors: &[Rgba]) -> Result<Self, VoxError> {
        crate::codec::merge_scenes(scenes, required_colors)
    }

    pub fn sample_instance_transform_global(
        &self,
        instance_index: usize,
        frame_index: u32,
    ) -> Result<Transform, VoxError> {
        let instance = self
            .instances
            .get(instance_index)
            .ok_or(VoxError::IndexOutOfBounds {
                kind: "instance",
                index: instance_index,
                len: self.instances.len(),
            })?;

        let mut transform = instance.sample_transform_local(frame_index);
        let mut group_index = instance.group_index;
        while let Some(index) = group_index {
            let group = self.groups.get(index).ok_or(VoxError::IndexOutOfBounds {
                kind: "group",
                index,
                len: self.groups.len(),
            })?;
            transform = transform.multiply(group.sample_transform_local(frame_index));
            group_index = group.parent_group_index;
        }

        Ok(transform)
    }

    pub fn sample_group_transform_global(
        &self,
        group_index: usize,
        frame_index: u32,
    ) -> Result<Transform, VoxError> {
        let mut group = self
            .groups
            .get(group_index)
            .ok_or(VoxError::IndexOutOfBounds {
                kind: "group",
                index: group_index,
                len: self.groups.len(),
            })?;
        let mut transform = group.sample_transform_local(frame_index);
        let mut parent = group.parent_group_index;
        while let Some(index) = parent {
            group = self.groups.get(index).ok_or(VoxError::IndexOutOfBounds {
                kind: "group",
                index,
                len: self.groups.len(),
            })?;
            transform = transform.multiply(group.sample_transform_local(frame_index));
            parent = group.parent_group_index;
        }

        Ok(transform)
    }
}

impl Instance {
    #[must_use]
    pub fn sample_model_index(&self, frame_index: u32) -> usize {
        if self.model_anim.keyframes.is_empty() {
            self.model_index
        } else {
            sample_anim_model(&self.model_anim, frame_index)
        }
    }

    #[must_use]
    pub fn sample_transform_local(&self, frame_index: u32) -> Transform {
        if self.transform_anim.keyframes.is_empty() {
            self.transform
        } else {
            sample_anim_transform(&self.transform_anim, frame_index)
        }
    }
}

impl Group {
    #[must_use]
    pub fn sample_transform_local(&self, frame_index: u32) -> Transform {
        if self.transform_anim.keyframes.is_empty() {
            self.transform
        } else {
            sample_anim_transform(&self.transform_anim, frame_index)
        }
    }
}

impl Camera {
    #[must_use]
    pub fn to_transform(&self) -> Transform {
        crate::codec::camera_to_transform(self)
    }
}

pub(crate) fn compute_looped_frame_index(
    first_loop_frame: u32,
    last_loop_frame: u32,
    frame_index: u32,
) -> u32 {
    let loop_len = 1 + last_loop_frame - first_loop_frame;
    if frame_index >= first_loop_frame {
        first_loop_frame + ((frame_index - first_loop_frame) % loop_len)
    } else {
        let frames_since_first_frame = first_loop_frame - frame_index - 1;
        last_loop_frame - (frames_since_first_frame % loop_len)
    }
}

pub(crate) fn sample_anim_transform(anim: &AnimTransform, frame_index: u32) -> Transform {
    let keyframes = &anim.keyframes;
    if keyframes.len() == 1 {
        return keyframes[0].transform;
    }

    let frame_index = if anim.looped {
        compute_looped_frame_index(
            keyframes[0].frame_index,
            keyframes[keyframes.len() - 1].frame_index,
            frame_index,
        )
    } else {
        frame_index
    };

    if frame_index <= keyframes[0].frame_index {
        return keyframes[0].transform;
    }
    if frame_index >= keyframes[keyframes.len() - 1].frame_index {
        return keyframes[keyframes.len() - 1].transform;
    }

    for index in (0..keyframes.len() - 1).rev() {
        if frame_index >= keyframes[index].frame_index {
            let current = &keyframes[index];
            let next = &keyframes[index + 1];
            let current_frame = current.frame_index;
            let next_frame = next.frame_index;
            let t = (frame_index - current_frame) as f32 / (next_frame - current_frame) as f32;
            let t_inv = 1.0 - t;
            let mut transform = current.transform;
            transform.m30 =
                ((next.transform.m30 * t) + (current.transform.m30 * t_inv)) as i32 as f32;
            transform.m31 =
                ((next.transform.m31 * t) + (current.transform.m31 * t_inv)) as i32 as f32;
            transform.m32 =
                ((next.transform.m32 * t) + (current.transform.m32 * t_inv)) as i32 as f32;
            return transform;
        }
    }

    keyframes[0].transform
}

pub(crate) fn sample_anim_model(anim: &AnimModel, frame_index: u32) -> usize {
    let keyframes = &anim.keyframes;
    if keyframes.len() == 1 {
        return keyframes[0].model_index;
    }

    let frame_index = if anim.looped {
        compute_looped_frame_index(
            keyframes[0].frame_index,
            keyframes[keyframes.len() - 1].frame_index,
            frame_index,
        )
    } else {
        frame_index
    };

    if frame_index <= keyframes[0].frame_index {
        return keyframes[0].model_index;
    }
    if frame_index >= keyframes[keyframes.len() - 1].frame_index {
        return keyframes[keyframes.len() - 1].model_index;
    }

    for index in (0..keyframes.len() - 1).rev() {
        if frame_index >= keyframes[index].frame_index {
            return keyframes[index].model_index;
        }
    }

    keyframes[0].model_index
}

pub(crate) const CHUNK_HEADER_LEN: usize = 12;
pub(crate) const INVALID_U32_INDEX: u32 = u32::MAX;

const DEFAULT_PALETTE_RAW: [u8; 1024] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xcc, 0xff, 0xff, 0xff, 0x99, 0xff, 0xff, 0xff, 0x66, 0xff,
    0xff, 0xff, 0x33, 0xff, 0xff, 0xff, 0x00, 0xff, 0xff, 0xcc, 0xff, 0xff, 0xff, 0xcc, 0xcc, 0xff,
    0xff, 0xcc, 0x99, 0xff, 0xff, 0xcc, 0x66, 0xff, 0xff, 0xcc, 0x33, 0xff, 0xff, 0xcc, 0x00, 0xff,
    0xff, 0x99, 0xff, 0xff, 0xff, 0x99, 0xcc, 0xff, 0xff, 0x99, 0x99, 0xff, 0xff, 0x99, 0x66, 0xff,
    0xff, 0x99, 0x33, 0xff, 0xff, 0x99, 0x00, 0xff, 0xff, 0x66, 0xff, 0xff, 0xff, 0x66, 0xcc, 0xff,
    0xff, 0x66, 0x99, 0xff, 0xff, 0x66, 0x66, 0xff, 0xff, 0x66, 0x33, 0xff, 0xff, 0x66, 0x00, 0xff,
    0xff, 0x33, 0xff, 0xff, 0xff, 0x33, 0xcc, 0xff, 0xff, 0x33, 0x99, 0xff, 0xff, 0x33, 0x66, 0xff,
    0xff, 0x33, 0x33, 0xff, 0xff, 0x33, 0x00, 0xff, 0xff, 0x00, 0xff, 0xff, 0xff, 0x00, 0xcc, 0xff,
    0xff, 0x00, 0x99, 0xff, 0xff, 0x00, 0x66, 0xff, 0xff, 0x00, 0x33, 0xff, 0xff, 0x00, 0x00, 0xff,
    0xcc, 0xff, 0xff, 0xff, 0xcc, 0xff, 0xcc, 0xff, 0xcc, 0xff, 0x99, 0xff, 0xcc, 0xff, 0x66, 0xff,
    0xcc, 0xff, 0x33, 0xff, 0xcc, 0xff, 0x00, 0xff, 0xcc, 0xcc, 0xff, 0xff, 0xcc, 0xcc, 0xcc, 0xff,
    0xcc, 0xcc, 0x99, 0xff, 0xcc, 0xcc, 0x66, 0xff, 0xcc, 0xcc, 0x33, 0xff, 0xcc, 0xcc, 0x00, 0xff,
    0xcc, 0x99, 0xff, 0xff, 0xcc, 0x99, 0xcc, 0xff, 0xcc, 0x99, 0x99, 0xff, 0xcc, 0x99, 0x66, 0xff,
    0xcc, 0x99, 0x33, 0xff, 0xcc, 0x99, 0x00, 0xff, 0xcc, 0x66, 0xff, 0xff, 0xcc, 0x66, 0xcc, 0xff,
    0xcc, 0x66, 0x99, 0xff, 0xcc, 0x66, 0x66, 0xff, 0xcc, 0x66, 0x33, 0xff, 0xcc, 0x66, 0x00, 0xff,
    0xcc, 0x33, 0xff, 0xff, 0xcc, 0x33, 0xcc, 0xff, 0xcc, 0x33, 0x99, 0xff, 0xcc, 0x33, 0x66, 0xff,
    0xcc, 0x33, 0x33, 0xff, 0xcc, 0x33, 0x00, 0xff, 0xcc, 0x00, 0xff, 0xff, 0xcc, 0x00, 0xcc, 0xff,
    0xcc, 0x00, 0x99, 0xff, 0xcc, 0x00, 0x66, 0xff, 0xcc, 0x00, 0x33, 0xff, 0xcc, 0x00, 0x00, 0xff,
    0x99, 0xff, 0xff, 0xff, 0x99, 0xff, 0xcc, 0xff, 0x99, 0xff, 0x99, 0xff, 0x99, 0xff, 0x66, 0xff,
    0x99, 0xff, 0x33, 0xff, 0x99, 0xff, 0x00, 0xff, 0x99, 0xcc, 0xff, 0xff, 0x99, 0xcc, 0xcc, 0xff,
    0x99, 0xcc, 0x99, 0xff, 0x99, 0xcc, 0x66, 0xff, 0x99, 0xcc, 0x33, 0xff, 0x99, 0xcc, 0x00, 0xff,
    0x99, 0x99, 0xff, 0xff, 0x99, 0x99, 0xcc, 0xff, 0x99, 0x99, 0x99, 0xff, 0x99, 0x99, 0x66, 0xff,
    0x99, 0x99, 0x33, 0xff, 0x99, 0x99, 0x00, 0xff, 0x99, 0x66, 0xff, 0xff, 0x99, 0x66, 0xcc, 0xff,
    0x99, 0x66, 0x99, 0xff, 0x99, 0x66, 0x66, 0xff, 0x99, 0x66, 0x33, 0xff, 0x99, 0x66, 0x00, 0xff,
    0x99, 0x33, 0xff, 0xff, 0x99, 0x33, 0xcc, 0xff, 0x99, 0x33, 0x99, 0xff, 0x99, 0x33, 0x66, 0xff,
    0x99, 0x33, 0x33, 0xff, 0x99, 0x33, 0x00, 0xff, 0x99, 0x00, 0xff, 0xff, 0x99, 0x00, 0xcc, 0xff,
    0x99, 0x00, 0x99, 0xff, 0x99, 0x00, 0x66, 0xff, 0x99, 0x00, 0x33, 0xff, 0x99, 0x00, 0x00, 0xff,
    0x66, 0xff, 0xff, 0xff, 0x66, 0xff, 0xcc, 0xff, 0x66, 0xff, 0x99, 0xff, 0x66, 0xff, 0x66, 0xff,
    0x66, 0xff, 0x33, 0xff, 0x66, 0xff, 0x00, 0xff, 0x66, 0xcc, 0xff, 0xff, 0x66, 0xcc, 0xcc, 0xff,
    0x66, 0xcc, 0x99, 0xff, 0x66, 0xcc, 0x66, 0xff, 0x66, 0xcc, 0x33, 0xff, 0x66, 0xcc, 0x00, 0xff,
    0x66, 0x99, 0xff, 0xff, 0x66, 0x99, 0xcc, 0xff, 0x66, 0x99, 0x99, 0xff, 0x66, 0x99, 0x66, 0xff,
    0x66, 0x99, 0x33, 0xff, 0x66, 0x99, 0x00, 0xff, 0x66, 0x66, 0xff, 0xff, 0x66, 0x66, 0xcc, 0xff,
    0x66, 0x66, 0x99, 0xff, 0x66, 0x66, 0x66, 0xff, 0x66, 0x66, 0x33, 0xff, 0x66, 0x66, 0x00, 0xff,
    0x66, 0x33, 0xff, 0xff, 0x66, 0x33, 0xcc, 0xff, 0x66, 0x33, 0x99, 0xff, 0x66, 0x33, 0x66, 0xff,
    0x66, 0x33, 0x33, 0xff, 0x66, 0x33, 0x00, 0xff, 0x66, 0x00, 0xff, 0xff, 0x66, 0x00, 0xcc, 0xff,
    0x66, 0x00, 0x99, 0xff, 0x66, 0x00, 0x66, 0xff, 0x66, 0x00, 0x33, 0xff, 0x66, 0x00, 0x00, 0xff,
    0x33, 0xff, 0xff, 0xff, 0x33, 0xff, 0xcc, 0xff, 0x33, 0xff, 0x99, 0xff, 0x33, 0xff, 0x66, 0xff,
    0x33, 0xff, 0x33, 0xff, 0x33, 0xff, 0x00, 0xff, 0x33, 0xcc, 0xff, 0xff, 0x33, 0xcc, 0xcc, 0xff,
    0x33, 0xcc, 0x99, 0xff, 0x33, 0xcc, 0x66, 0xff, 0x33, 0xcc, 0x33, 0xff, 0x33, 0xcc, 0x00, 0xff,
    0x33, 0x99, 0xff, 0xff, 0x33, 0x99, 0xcc, 0xff, 0x33, 0x99, 0x99, 0xff, 0x33, 0x99, 0x66, 0xff,
    0x33, 0x99, 0x33, 0xff, 0x33, 0x99, 0x00, 0xff, 0x33, 0x66, 0xff, 0xff, 0x33, 0x66, 0xcc, 0xff,
    0x33, 0x66, 0x99, 0xff, 0x33, 0x66, 0x66, 0xff, 0x33, 0x66, 0x33, 0xff, 0x33, 0x66, 0x00, 0xff,
    0x33, 0x33, 0xff, 0xff, 0x33, 0x33, 0xcc, 0xff, 0x33, 0x33, 0x99, 0xff, 0x33, 0x33, 0x66, 0xff,
    0x33, 0x33, 0x33, 0xff, 0x33, 0x33, 0x00, 0xff, 0x33, 0x00, 0xff, 0xff, 0x33, 0x00, 0xcc, 0xff,
    0x33, 0x00, 0x99, 0xff, 0x33, 0x00, 0x66, 0xff, 0x33, 0x00, 0x33, 0xff, 0x33, 0x00, 0x00, 0xff,
    0x00, 0xff, 0xff, 0xff, 0x00, 0xff, 0xcc, 0xff, 0x00, 0xff, 0x99, 0xff, 0x00, 0xff, 0x66, 0xff,
    0x00, 0xff, 0x33, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xcc, 0xff, 0xff, 0x00, 0xcc, 0xcc, 0xff,
    0x00, 0xcc, 0x99, 0xff, 0x00, 0xcc, 0x66, 0xff, 0x00, 0xcc, 0x33, 0xff, 0x00, 0xcc, 0x00, 0xff,
    0x00, 0x99, 0xff, 0xff, 0x00, 0x99, 0xcc, 0xff, 0x00, 0x99, 0x99, 0xff, 0x00, 0x99, 0x66, 0xff,
    0x00, 0x99, 0x33, 0xff, 0x00, 0x99, 0x00, 0xff, 0x00, 0x66, 0xff, 0xff, 0x00, 0x66, 0xcc, 0xff,
    0x00, 0x66, 0x99, 0xff, 0x00, 0x66, 0x66, 0xff, 0x00, 0x66, 0x33, 0xff, 0x00, 0x66, 0x00, 0xff,
    0x00, 0x33, 0xff, 0xff, 0x00, 0x33, 0xcc, 0xff, 0x00, 0x33, 0x99, 0xff, 0x00, 0x33, 0x66, 0xff,
    0x00, 0x33, 0x33, 0xff, 0x00, 0x33, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xcc, 0xff,
    0x00, 0x00, 0x99, 0xff, 0x00, 0x00, 0x66, 0xff, 0x00, 0x00, 0x33, 0xff, 0xee, 0x00, 0x00, 0xff,
    0xdd, 0x00, 0x00, 0xff, 0xbb, 0x00, 0x00, 0xff, 0xaa, 0x00, 0x00, 0xff, 0x88, 0x00, 0x00, 0xff,
    0x77, 0x00, 0x00, 0xff, 0x55, 0x00, 0x00, 0xff, 0x44, 0x00, 0x00, 0xff, 0x22, 0x00, 0x00, 0xff,
    0x11, 0x00, 0x00, 0xff, 0x00, 0xee, 0x00, 0xff, 0x00, 0xdd, 0x00, 0xff, 0x00, 0xbb, 0x00, 0xff,
    0x00, 0xaa, 0x00, 0xff, 0x00, 0x88, 0x00, 0xff, 0x00, 0x77, 0x00, 0xff, 0x00, 0x55, 0x00, 0xff,
    0x00, 0x44, 0x00, 0xff, 0x00, 0x22, 0x00, 0xff, 0x00, 0x11, 0x00, 0xff, 0x00, 0x00, 0xee, 0xff,
    0x00, 0x00, 0xdd, 0xff, 0x00, 0x00, 0xbb, 0xff, 0x00, 0x00, 0xaa, 0xff, 0x00, 0x00, 0x88, 0xff,
    0x00, 0x00, 0x77, 0xff, 0x00, 0x00, 0x55, 0xff, 0x00, 0x00, 0x44, 0xff, 0x00, 0x00, 0x22, 0xff,
    0x00, 0x00, 0x11, 0xff, 0xee, 0xee, 0xee, 0xff, 0xdd, 0xdd, 0xdd, 0xff, 0xbb, 0xbb, 0xbb, 0xff,
    0xaa, 0xaa, 0xaa, 0xff, 0x88, 0x88, 0x88, 0xff, 0x77, 0x77, 0x77, 0xff, 0x55, 0x55, 0x55, 0xff,
    0x44, 0x44, 0x44, 0xff, 0x22, 0x22, 0x22, 0xff, 0x11, 0x11, 0x11, 0xff, 0x00, 0x00, 0x00, 0xff,
];
