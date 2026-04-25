use alloc::string::String;
use alloc::vec::Vec;
use core::array;
use core::error::Error;
use core::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// An RGBA color with 8-bit channels.
pub struct Rgba {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel.
    pub a: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// A 4x4 transform matrix stored in row-major order.
pub struct Transform {
    /// Row 0, column 0.
    pub m00: f32,
    /// Row 0, column 1.
    pub m01: f32,
    /// Row 0, column 2.
    pub m02: f32,
    /// Row 0, column 3.
    pub m03: f32,
    /// Row 1, column 0.
    pub m10: f32,
    /// Row 1, column 1.
    pub m11: f32,
    /// Row 1, column 2.
    pub m12: f32,
    /// Row 1, column 3.
    pub m13: f32,
    /// Row 2, column 0.
    pub m20: f32,
    /// Row 2, column 1.
    pub m21: f32,
    /// Row 2, column 2.
    pub m22: f32,
    /// Row 2, column 3.
    pub m23: f32,
    /// Row 3, column 0.
    pub m30: f32,
    /// Row 3, column 1.
    pub m31: f32,
    /// Row 3, column 2.
    pub m32: f32,
    /// Row 3, column 3.
    pub m33: f32,
}

impl Transform {
    /// Returns the identity transform.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::Transform;
    ///
    /// let transform = Transform::identity();
    /// assert_eq!(transform.m00, 1.0);
    /// assert_eq!(transform.m33, 1.0);
    /// ```
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

    /// Composes this transform with `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::Transform;
    ///
    /// let translation = Transform {
    ///     m30: 4.0,
    ///     m31: -2.0,
    ///     m32: 1.0,
    ///     ..Transform::identity()
    /// };
    /// let combined = Transform::identity().multiply(translation);
    /// assert_eq!(combined.m30, 4.0);
    /// assert_eq!(combined.m31, -2.0);
    /// assert_eq!(combined.m32, 1.0);
    /// ```
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
/// A fixed 256-entry palette.
pub struct Palette {
    /// Palette entries indexed from `0` to `255`.
    pub colors: [Rgba; 256],
}

impl Palette {
    /// Returns MagicaVoxel's raw default palette.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::Palette;
    ///
    /// let palette = Palette::raw_default();
    /// assert_eq!(palette.colors.len(), 256);
    /// ```
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

    /// Returns the scene palette used by [`Default`] for [`Palette`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::Palette;
    ///
    /// let palette = Palette::default_scene_palette();
    /// assert_eq!(palette.colors[0].a, 0);
    /// ```
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
/// Material families supported by scene materials.
pub enum MaterialType {
    #[default]
    /// The default diffuse material.
    Diffuse,
    /// A metallic material.
    Metal,
    /// A glass-like material.
    Glass,
    /// An emissive material.
    Emit,
    /// A blended material.
    Blend,
    /// A volumetric medium material.
    Media,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Media interaction modes supported by scene materials.
pub enum MediaType {
    #[default]
    /// The medium absorbs light.
    Absorb,
    /// The medium scatters light.
    Scatter,
    /// The medium emits light.
    Emit,
    /// A subsurface-scattering medium.
    Sss,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Material properties attached to a palette entry.
pub struct Material {
    /// Material family.
    pub material_type: MaterialType,
    /// Media interaction mode.
    pub media_type: MediaType,
    /// Optional value for the `_metal` property.
    pub metal: Option<f32>,
    /// Optional value for the `_rough` property.
    pub rough: Option<f32>,
    /// Optional value for the `_spec` property.
    pub spec: Option<f32>,
    /// Optional value for the `_ior` property.
    pub ior: Option<f32>,
    /// Optional value for the `_att` property.
    pub att: Option<f32>,
    /// Optional value for the `_flux` property.
    pub flux: Option<f32>,
    /// Optional value for the `_emit` property.
    pub emit: Option<f32>,
    /// Optional value for the `_ldr` property.
    pub ldr: Option<f32>,
    /// Optional value for the `_trans` property.
    pub trans: Option<f32>,
    /// Optional value for the `_alpha` property.
    pub alpha: Option<f32>,
    /// Optional value for the `_d` property.
    pub d: Option<f32>,
    /// Optional value for the `_sp` property.
    pub sp: Option<f32>,
    /// Optional value for the `_g` property.
    pub g: Option<f32>,
    /// Optional value for the `_media` property.
    pub media: Option<f32>,
}

impl Material {
    /// Returns `true` if any material property is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::Material;
    ///
    /// let mut material = Material::default();
    /// assert!(!material.has_content());
    /// material.alpha = Some(0.5);
    /// assert!(material.has_content());
    /// ```
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
/// Camera modes supported by MagicaVoxel.
pub enum CameraMode {
    #[default]
    /// Perspective projection.
    Perspective,
    /// Free-form camera movement.
    Free,
    /// Panorama projection.
    Pano,
    /// Orthographic projection.
    Orthographic,
    /// Isometric projection.
    Isometric,
    /// A mode not recognized by this crate.
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
/// A camera definition stored in a scene.
pub struct Camera {
    /// Stable camera identifier.
    pub camera_id: u32,
    /// Camera projection mode.
    pub mode: CameraMode,
    /// Focus point in scene space.
    pub focus: [f32; 3],
    /// Euler angles in degrees.
    pub angle: [f32; 3],
    /// Distance from the focus point.
    pub radius: f32,
    /// Frustum parameter used by some camera modes.
    pub frustum: f32,
    /// Field of view in degrees.
    pub fov: i32,
}

#[derive(Clone, Debug, PartialEq)]
/// Global sun and sky lighting parameters.
pub struct Sun {
    /// Light intensity.
    pub intensity: f32,
    /// Light area.
    pub area: f32,
    /// Sun angles in degrees.
    pub angle: [f32; 2],
    /// Light color.
    pub rgba: Rgba,
    /// Whether the light renders as a disk.
    pub disk: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A voxel model with dimensions and linear voxel data.
pub struct Model {
    /// Size along the x axis.
    pub size_x: u32,
    /// Size along the y axis.
    pub size_y: u32,
    /// Size along the z axis.
    pub size_z: u32,
    /// Voxel color indices stored in x-major order, then y, then z.
    pub voxels: Vec<u8>,
}

impl Model {
    /// Returns the number of voxel slots implied by the model dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::Model;
    ///
    /// let model = Model {
    ///     size_x: 2,
    ///     size_y: 2,
    ///     size_z: 1,
    ///     voxels: vec![0; 4],
    /// };
    /// assert_eq!(model.voxel_count().unwrap(), 4);
    /// ```
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

    /// Counts voxels whose color index is not zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::Model;
    ///
    /// let model = Model {
    ///     size_x: 2,
    ///     size_y: 2,
    ///     size_z: 1,
    ///     voxels: vec![0, 1, 0, 2],
    /// };
    /// assert_eq!(model.solid_voxel_count(), 2);
    /// ```
    #[must_use]
    pub fn solid_voxel_count(&self) -> usize {
        self.voxels.iter().filter(|&&value| value != 0).count()
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A transform sampled at a specific animation frame.
pub struct KeyframeTransform {
    /// Frame at which this transform applies.
    pub frame_index: u32,
    /// Transform value for the frame.
    pub transform: Transform,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A model index sampled at a specific animation frame.
pub struct KeyframeModel {
    /// Frame at which this model index applies.
    pub frame_index: u32,
    /// Model index for the frame.
    pub model_index: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Transform keyframes and looping state for a node.
pub struct AnimTransform {
    /// Ordered transform keyframes.
    pub keyframes: Vec<KeyframeTransform>,
    /// Whether animation should loop.
    pub looped: bool,
}

impl AnimTransform {
    /// Returns `true` when no transform keyframes are stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::AnimTransform;
    ///
    /// assert!(AnimTransform::default().is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Model keyframes and looping state for a node.
pub struct AnimModel {
    /// Ordered model keyframes.
    pub keyframes: Vec<KeyframeModel>,
    /// Whether animation should loop.
    pub looped: bool,
}

impl AnimModel {
    /// Returns `true` when no model keyframes are stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::AnimModel;
    ///
    /// assert!(AnimModel::default().is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A scene instance referencing one model and optional parents in the scene graph.
pub struct Instance {
    /// Optional instance name.
    pub name: Option<String>,
    /// Local transform.
    pub transform: Transform,
    /// Index into `Scene::models`.
    pub model_index: usize,
    /// Optional index into `Scene::layers`.
    pub layer_index: Option<usize>,
    /// Optional index into `Scene::groups`.
    pub group_index: Option<usize>,
    /// Whether the instance is hidden.
    pub hidden: bool,
    /// Transform animation for the instance.
    pub transform_anim: AnimTransform,
    /// Model selection animation for the instance.
    pub model_anim: AnimModel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A logical layer in the scene graph.
pub struct Layer {
    /// Optional layer name.
    pub name: Option<String>,
    /// Layer display color.
    pub color: Rgba,
    /// Whether the layer is hidden.
    pub hidden: bool,
}

#[derive(Clone, Debug, PartialEq)]
/// A group node in the scene graph.
pub struct Group {
    /// Optional group name.
    pub name: Option<String>,
    /// Local transform.
    pub transform: Transform,
    /// Optional parent group index.
    pub parent_group_index: Option<usize>,
    /// Optional layer index.
    pub layer_index: Option<usize>,
    /// Whether the group is hidden.
    pub hidden: bool,
    /// Transform animation for the group.
    pub transform_anim: AnimTransform,
}

#[derive(Clone, Debug, PartialEq)]
/// The complete in-memory representation of a `.vox` file.
pub struct Scene {
    /// File version reported by the source file.
    pub file_version: u32,
    /// Voxel models referenced by the scene graph.
    pub models: Vec<Model>,
    /// Scene instances referencing models and groups.
    pub instances: Vec<Instance>,
    /// Scene layers.
    pub layers: Vec<Layer>,
    /// Scene groups.
    pub groups: Vec<Group>,
    /// Optional color names from `NOTE` chunks.
    pub color_names: Vec<String>,
    /// Active palette.
    pub palette: Palette,
    /// Per-palette-entry material data.
    pub materials: [Material; 256],
    /// Cameras defined in the scene.
    pub cameras: Vec<Camera>,
    /// Optional sun/light definition.
    pub sun: Option<Sun>,
    /// Start frame for animation playback.
    pub anim_range_start: u32,
    /// End frame for animation playback.
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
/// Options that control how scenes are normalized while reading.
pub struct ReadOptions {
    /// Preserve group hierarchy instead of flattening it.
    pub preserve_groups: bool,
    /// Preserve animation keyframes instead of baking them.
    pub preserve_keyframes: bool,
    /// Keep empty model instances instead of dropping them.
    pub keep_empty_models_instances: bool,
    /// Keep duplicate models instead of deduplicating them.
    pub keep_duplicate_models: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Errors that can be produced while reading or writing `.vox` data.
pub enum VoxError {
    /// The input ended unexpectedly.
    UnexpectedEof,
    /// The file header is not valid `.vox` data.
    InvalidHeader,
    /// The file version is not supported.
    UnsupportedVersion(u32),
    /// A chunk is malformed.
    InvalidChunk(&'static str),
    /// The scene graph is malformed.
    InvalidNodeGraph(&'static str),
    /// The file contains invalid data.
    InvalidData(String),
    /// A referenced index is out of bounds.
    IndexOutOfBounds {
        /// The kind of index that was out of bounds.
        kind: &'static str,
        /// The invalid index value.
        index: usize,
        /// The length of the collection that was indexed.
        len: usize,
    },
    /// The requested required-color set is too large.
    TooManyRequiredColors(usize),
    /// Scene writing was cancelled by progress reporting.
    WriteCancelled,
    /// The generated file would exceed the 4 GiB `.vox` limit.
    FileTooLarge,
    #[cfg(feature = "std")]
    /// A standard I/O error kind encountered while reading or writing.
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
            Self::IoErrorKind(kind) => write!(f, "i/o error: {kind}"),
        }
    }
}

impl Error for VoxError {}

impl Scene {
    /// Reads a `.vox` file from any `Read` implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use vox_rs::Scene;
    ///
    /// let bytes: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/test_groups.vox"));
    /// let mut cursor = Cursor::new(bytes);
    /// let scene = Scene::read(&mut cursor).unwrap();
    /// assert!(!scene.groups.is_empty());
    /// ```
    #[cfg(feature = "std")]
    pub fn read<R>(reader: &mut R) -> Result<Self, VoxError>
    where
        R: std::io::Read,
    {
        crate::codec::read_scene_from_reader(reader, ReadOptions::default())
    }

    /// Reads a `.vox` file from any `Read` implementation using explicit options.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use vox_rs::{ReadOptions, Scene};
    ///
    /// let bytes: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/test_groups.vox"));
    /// let mut cursor = Cursor::new(bytes);
    /// let scene = Scene::read_with_options(
    ///     &mut cursor,
    ///     ReadOptions {
    ///         preserve_groups: true,
    ///         preserve_keyframes: true,
    ///         ..ReadOptions::default()
    ///     },
    /// ).unwrap();
    /// assert!(!scene.groups.is_empty());
    /// ```
    #[cfg(feature = "std")]
    pub fn read_with_options<R>(reader: &mut R, options: ReadOptions) -> Result<Self, VoxError>
    where
        R: std::io::Read,
    {
        crate::codec::read_scene_from_reader(reader, options)
    }

    /// Reads a `.vox` file from an in-memory byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::Scene;
    ///
    /// let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/test_groups.vox"));
    /// let scene = Scene::read_bytes(bytes).unwrap();
    /// assert!(!scene.groups.is_empty());
    /// ```
    pub fn read_bytes(bytes: &[u8]) -> Result<Self, VoxError> {
        crate::codec::read_scene(bytes, ReadOptions::default())
    }

    /// Reads a `.vox` file from an in-memory byte slice using explicit options.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::{ReadOptions, Scene};
    ///
    /// let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/test_groups.vox"));
    /// let scene = Scene::read_bytes_with_options(
    ///     bytes,
    ///     ReadOptions {
    ///         preserve_groups: true,
    ///         preserve_keyframes: true,
    ///         ..ReadOptions::default()
    ///     },
    /// ).unwrap();
    /// assert!(!scene.groups.is_empty());
    /// ```
    pub fn read_bytes_with_options(bytes: &[u8], options: ReadOptions) -> Result<Self, VoxError> {
        crate::codec::read_scene(bytes, options)
    }

    /// Serializes the scene into a new byte vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::Scene;
    ///
    /// let scene = Scene::read_bytes(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/test_groups.vox"))).unwrap();
    /// let bytes = scene.write().unwrap();
    /// assert!(bytes.starts_with(b"VOX "));
    /// ```
    pub fn write(&self) -> Result<Vec<u8>, VoxError> {
        crate::codec::write_scene_with_progress(self, |_| true)
    }

    /// Serializes the scene into a new byte vector while reporting progress.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::Scene;
    ///
    /// let scene = Scene::read_bytes(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/test_groups.vox"))).unwrap();
    /// let bytes = scene.write_with_progress(|_| true).unwrap();
    /// assert!(bytes.starts_with(b"VOX "));
    /// ```
    pub fn write_with_progress<F>(&self, progress: F) -> Result<Vec<u8>, VoxError>
    where
        F: FnMut(f32) -> bool,
    {
        crate::codec::write_scene_with_progress(self, progress)
    }

    /// Writes the scene directly into any `Write` implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use vox_rs::Scene;
    ///
    /// let scene = Scene::read_bytes(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/test_groups.vox"))).unwrap();
    /// let mut out = Cursor::new(Vec::new());
    /// scene.write_to(&mut out).unwrap();
    /// assert!(out.into_inner().starts_with(b"VOX "));
    /// ```
    #[cfg(feature = "std")]
    pub fn write_to<W>(&self, writer: &mut W) -> Result<(), VoxError>
    where
        W: std::io::Write,
    {
        crate::codec::write_scene_to_writer(self, writer, |_| true)
    }

    /// Writes the scene directly into any `Write` implementation while reporting progress.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use vox_rs::Scene;
    ///
    /// let scene = Scene::read_bytes(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/test_groups.vox"))).unwrap();
    /// let mut out = Cursor::new(Vec::new());
    /// scene.write_to_with_progress(&mut out, |_| true).unwrap();
    /// assert!(out.into_inner().starts_with(b"VOX "));
    /// ```
    #[cfg(feature = "std")]
    pub fn write_to_with_progress<W, F>(&self, writer: &mut W, progress: F) -> Result<(), VoxError>
    where
        W: std::io::Write,
        F: FnMut(f32) -> bool,
    {
        crate::codec::write_scene_to_writer(self, writer, progress)
    }

    /// Merges several scenes into one scene with an optional required color set.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::{AnimModel, AnimTransform, Instance, Model, Scene, Transform};
    ///
    /// let left = Scene {
    ///     models: vec![Model {
    ///         size_x: 1,
    ///         size_y: 1,
    ///         size_z: 1,
    ///         voxels: vec![1],
    ///     }],
    ///     instances: vec![Instance {
    ///         name: None,
    ///         transform: Transform::identity(),
    ///         model_index: 0,
    ///         layer_index: None,
    ///         group_index: None,
    ///         hidden: false,
    ///         transform_anim: AnimTransform::default(),
    ///         model_anim: AnimModel::default(),
    ///     }],
    ///     ..Scene::default()
    /// };
    /// let right = Scene {
    ///     models: vec![Model {
    ///         size_x: 1,
    ///         size_y: 1,
    ///         size_z: 1,
    ///         voxels: vec![2],
    ///     }],
    ///     instances: vec![Instance {
    ///         name: None,
    ///         transform: Transform {
    ///             m30: 1.0,
    ///             ..Transform::identity()
    ///         },
    ///         model_index: 0,
    ///         layer_index: None,
    ///         group_index: None,
    ///         hidden: false,
    ///         transform_anim: AnimTransform::default(),
    ///         model_anim: AnimModel::default(),
    ///     }],
    ///     ..Scene::default()
    /// };
    ///
    /// let merged = Scene::merge(&[&left, &right], &[]).unwrap();
    /// assert_eq!(merged.instances.len(), 2);
    /// ```
    pub fn merge(scenes: &[&Scene], required_colors: &[Rgba]) -> Result<Self, VoxError> {
        crate::codec::merge_scenes(scenes, required_colors)
    }

    /// Returns the global transform of an instance at the given animation frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::{AnimModel, AnimTransform, Group, Instance, Model, Scene, Transform};
    ///
    /// let scene = Scene {
    ///     models: vec![Model {
    ///         size_x: 1,
    ///         size_y: 1,
    ///         size_z: 1,
    ///         voxels: vec![1],
    ///     }],
    ///     groups: vec![Group {
    ///         name: None,
    ///         transform: Transform::identity(),
    ///         parent_group_index: None,
    ///         layer_index: None,
    ///         hidden: false,
    ///         transform_anim: AnimTransform::default(),
    ///     }],
    ///     instances: vec![Instance {
    ///         name: None,
    ///         transform: Transform {
    ///             m30: 2.0,
    ///             ..Transform::identity()
    ///         },
    ///         model_index: 0,
    ///         layer_index: None,
    ///         group_index: Some(0),
    ///         hidden: false,
    ///         transform_anim: AnimTransform::default(),
    ///         model_anim: AnimModel::default(),
    ///     }],
    ///     ..Scene::default()
    /// };
    ///
    /// let transform = scene.sample_instance_transform_global(0, 0).unwrap();
    /// assert_eq!(transform.m30, 2.0);
    /// ```
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

    /// Returns the global transform of a group at the given animation frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::{AnimTransform, Group, Scene, Transform};
    ///
    /// let scene = Scene {
    ///     groups: vec![Group {
    ///         name: None,
    ///         transform: Transform {
    ///             m30: 3.0,
    ///             ..Transform::identity()
    ///         },
    ///         parent_group_index: None,
    ///         layer_index: None,
    ///         hidden: false,
    ///         transform_anim: AnimTransform::default(),
    ///     }],
    ///     ..Scene::default()
    /// };
    ///
    /// let transform = scene.sample_group_transform_global(0, 0).unwrap();
    /// assert_eq!(transform.m30, 3.0);
    /// ```
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
    /// Returns the model index for the given animation frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::{AnimModel, Instance, KeyframeModel, Transform};
    ///
    /// let instance = Instance {
    ///     name: None,
    ///     transform: Transform::identity(),
    ///     model_index: 2,
    ///     layer_index: None,
    ///     group_index: None,
    ///     hidden: false,
    ///     transform_anim: Default::default(),
    ///     model_anim: AnimModel {
    ///         keyframes: vec![KeyframeModel {
    ///             frame_index: 0,
    ///             model_index: 7,
    ///         }],
    ///         looped: false,
    ///     },
    /// };
    ///
    /// assert_eq!(instance.sample_model_index(42), 7);
    /// ```
    #[must_use]
    pub fn sample_model_index(&self, frame_index: u32) -> usize {
        if self.model_anim.keyframes.is_empty() {
            self.model_index
        } else {
            sample_anim_model(&self.model_anim, frame_index)
        }
    }

    /// Returns the local transform for the given animation frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::{AnimModel, AnimTransform, Instance, KeyframeTransform, Transform};
    ///
    /// let instance = Instance {
    ///     name: None,
    ///     transform: Transform::identity(),
    ///     model_index: 0,
    ///     layer_index: None,
    ///     group_index: None,
    ///     hidden: false,
    ///     transform_anim: AnimTransform {
    ///         keyframes: vec![
    ///             KeyframeTransform {
    ///                 frame_index: 0,
    ///                 transform: Transform::identity(),
    ///             },
    ///             KeyframeTransform {
    ///                 frame_index: 10,
    ///                 transform: Transform {
    ///                     m30: 10.0,
    ///                     ..Transform::identity()
    ///                 },
    ///             },
    ///         ],
    ///         looped: false,
    ///     },
    ///     model_anim: AnimModel::default(),
    /// };
    ///
    /// assert_eq!(instance.sample_transform_local(5).m30, 5.0);
    /// ```
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
    /// Returns the local transform for the given animation frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::{AnimTransform, Group, KeyframeTransform, Transform};
    ///
    /// let group = Group {
    ///     name: None,
    ///     transform: Transform::identity(),
    ///     parent_group_index: None,
    ///     layer_index: None,
    ///     hidden: false,
    ///     transform_anim: AnimTransform {
    ///         keyframes: vec![
    ///             KeyframeTransform {
    ///                 frame_index: 0,
    ///                 transform: Transform::identity(),
    ///             },
    ///             KeyframeTransform {
    ///                 frame_index: 10,
    ///                 transform: Transform {
    ///                     m30: 10.0,
    ///                     ..Transform::identity()
    ///                 },
    ///             },
    ///         ],
    ///         looped: false,
    ///     },
    /// };
    ///
    /// assert_eq!(group.sample_transform_local(5).m30, 5.0);
    /// ```
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
    /// Converts this camera to a transform matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// use vox_rs::{Camera, CameraMode};
    ///
    /// let camera = Camera {
    ///     camera_id: 0,
    ///     mode: CameraMode::Perspective,
    ///     focus: [0.0, 0.0, 0.0],
    ///     angle: [0.0, 0.0, 0.0],
    ///     radius: 10.0,
    ///     frustum: 0.0,
    ///     fov: 45,
    /// };
    ///
    /// let transform = camera.to_transform();
    /// assert!((transform.m32 + 10.0).abs() < 1e-4);
    /// ```
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
