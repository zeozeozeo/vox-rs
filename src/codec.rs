use alloc::borrow::ToOwned;
#[cfg(feature = "no_std")]
use alloc::collections::BTreeMap as DictMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::array;
use core::f32::consts::PI;
#[cfg(feature = "std")]
use std::collections::HashMap as DictMap;
#[cfg(feature = "std")]
use std::io::Read;

use crate::types::{
    AnimModel, AnimTransform, CHUNK_HEADER_LEN, Camera, CameraMode, Group, INVALID_U32_INDEX,
    Instance, KeyframeModel, KeyframeTransform, Layer, Material, MaterialType, MediaType, Model,
    Palette, ReadOptions, Rgba, Scene, Sun, Transform, VoxError,
};

const fn chunk_id(bytes: [u8; 4]) -> u32 {
    (bytes[0] as u32)
        | ((bytes[1] as u32) << 8)
        | ((bytes[2] as u32) << 16)
        | ((bytes[3] as u32) << 24)
}

const CHUNK_ID_VOX_: u32 = chunk_id(*b"VOX ");
const CHUNK_ID_MAIN: u32 = chunk_id(*b"MAIN");
const CHUNK_ID_SIZE: u32 = chunk_id(*b"SIZE");
const CHUNK_ID_XYZI: u32 = chunk_id(*b"XYZI");
const CHUNK_ID_RGBA: u32 = chunk_id(*b"RGBA");
const CHUNK_ID_NTRN: u32 = chunk_id(*b"nTRN");
const CHUNK_ID_NGRP: u32 = chunk_id(*b"nGRP");
const CHUNK_ID_NSHP: u32 = chunk_id(*b"nSHP");
const CHUNK_ID_IMAP: u32 = chunk_id(*b"IMAP");
const CHUNK_ID_LAYR: u32 = chunk_id(*b"LAYR");
const CHUNK_ID_MATL: u32 = chunk_id(*b"MATL");
const CHUNK_ID_MATT: u32 = chunk_id(*b"MATT");
const CHUNK_ID_ROBJ: u32 = chunk_id(*b"rOBJ");
const CHUNK_ID_RCAM: u32 = chunk_id(*b"rCAM");
const CHUNK_ID_NOTE: u32 = chunk_id(*b"NOTE");
const CHUNK_ID_META: u32 = chunk_id(*b"META");

#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn negate(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }

    fn normalize(self) -> Self {
        let inv_len = 1.0 / sqrt_f32(self.x * self.x + self.y * self.y + self.z * self.z);
        Self::new(self.x * inv_len, self.y * inv_len, self.z * inv_len)
    }

    fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

#[cfg(feature = "no_std")]
fn wrap_angle_radians(mut value: f32) -> f32 {
    let two_pi = 2.0 * PI;
    while value > PI {
        value -= two_pi;
    }
    while value < -PI {
        value += two_pi;
    }
    value
}

#[cfg(feature = "no_std")]
fn sqrt_f32(value: f32) -> f32 {
    if value <= 0.0 {
        return 0.0;
    }
    let mut guess = if value >= 1.0 { value } else { 1.0 };
    for _ in 0..8 {
        guess = 0.5 * (guess + value / guess);
    }
    guess
}

#[cfg(not(feature = "no_std"))]
fn sqrt_f32(value: f32) -> f32 {
    value.sqrt()
}

#[cfg(feature = "no_std")]
fn sin_f32(value: f32) -> f32 {
    let x = wrap_angle_radians(value);
    let x2 = x * x;
    x * (1.0 - (x2 / 6.0) + ((x2 * x2) / 120.0) - ((x2 * x2 * x2) / 5040.0))
}

#[cfg(not(feature = "no_std"))]
fn sin_f32(value: f32) -> f32 {
    value.sin()
}

#[cfg(feature = "no_std")]
fn cos_f32(value: f32) -> f32 {
    let x = wrap_angle_radians(value);
    let x2 = x * x;
    1.0 - (x2 / 2.0) + ((x2 * x2) / 24.0) - ((x2 * x2 * x2) / 720.0)
}

#[cfg(not(feature = "no_std"))]
fn cos_f32(value: f32) -> f32 {
    value.cos()
}

#[derive(Default)]
struct ByteReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> ByteReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], VoxError> {
        if self.remaining() < len {
            return Err(VoxError::UnexpectedEof);
        }
        let start = self.offset;
        self.offset += len;
        Ok(&self.data[start..start + len])
    }
}

trait VoxRead {
    fn read_exact_into(&mut self, buf: &mut [u8]) -> Result<(), VoxError>;
    fn read_exact_or_eof(&mut self, buf: &mut [u8]) -> Result<bool, VoxError>;

    fn read_u8(&mut self) -> Result<u8, VoxError> {
        let mut buf = [0u8; 1];
        self.read_exact_into(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u32(&mut self) -> Result<u32, VoxError> {
        let mut buf = [0u8; 4];
        self.read_exact_into(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_i32(&mut self) -> Result<i32, VoxError> {
        let mut buf = [0u8; 4];
        self.read_exact_into(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    fn read_f32(&mut self) -> Result<f32, VoxError> {
        let mut buf = [0u8; 4];
        self.read_exact_into(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    fn read_string_with_length(&mut self) -> Result<String, VoxError> {
        let len = usize::try_from(self.read_u32()?)
            .map_err(|_| VoxError::InvalidData("string length overflow".into()))?;
        let mut bytes = vec![0u8; len];
        self.read_exact_into(&mut bytes)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn read_u32_or_eof(&mut self) -> Result<Option<u32>, VoxError> {
        let mut buf = [0u8; 4];
        if self.read_exact_or_eof(&mut buf)? {
            Ok(Some(u32::from_le_bytes(buf)))
        } else {
            Ok(None)
        }
    }
}

impl VoxRead for ByteReader<'_> {
    fn read_exact_into(&mut self, buf: &mut [u8]) -> Result<(), VoxError> {
        buf.copy_from_slice(self.read_exact(buf.len())?);
        Ok(())
    }

    fn read_exact_or_eof(&mut self, buf: &mut [u8]) -> Result<bool, VoxError> {
        if self.remaining() == 0 {
            return Ok(false);
        }
        self.read_exact_into(buf)?;
        Ok(true)
    }
}

struct LimitedReader<'a, R> {
    inner: &'a mut R,
    remaining: usize,
}

impl<'a, R> LimitedReader<'a, R> {
    fn new(inner: &'a mut R, remaining: usize) -> Self {
        Self { inner, remaining }
    }

    fn skip_remaining(&mut self) -> Result<(), VoxError>
    where
        R: VoxRead,
    {
        let mut buf = [0u8; 8192];
        while self.remaining > 0 {
            let len = self.remaining.min(buf.len());
            self.read_exact_into(&mut buf[..len])?;
        }
        Ok(())
    }
}

impl<R> VoxRead for LimitedReader<'_, R>
where
    R: VoxRead,
{
    fn read_exact_into(&mut self, buf: &mut [u8]) -> Result<(), VoxError> {
        if buf.len() > self.remaining {
            return Err(VoxError::UnexpectedEof);
        }
        self.inner.read_exact_into(buf)?;
        self.remaining -= buf.len();
        Ok(())
    }

    fn read_exact_or_eof(&mut self, buf: &mut [u8]) -> Result<bool, VoxError> {
        if self.remaining == 0 {
            return Ok(false);
        }
        self.read_exact_into(buf)?;
        Ok(true)
    }
}

#[cfg(feature = "std")]
struct IoReader<'a, R> {
    inner: &'a mut R,
}

#[cfg(feature = "std")]
impl<'a, R> IoReader<'a, R> {
    fn new(inner: &'a mut R) -> Self {
        Self { inner }
    }
}

#[cfg(feature = "std")]
impl<R> VoxRead for IoReader<'_, R>
where
    R: Read,
{
    fn read_exact_into(&mut self, buf: &mut [u8]) -> Result<(), VoxError> {
        self.inner.read_exact(buf).map_err(|error| {
            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                VoxError::UnexpectedEof
            } else {
                VoxError::IoErrorKind(error.kind())
            }
        })
    }

    fn read_exact_or_eof(&mut self, buf: &mut [u8]) -> Result<bool, VoxError> {
        let mut offset = 0usize;
        while offset < buf.len() {
            match self.inner.read(&mut buf[offset..]) {
                Ok(0) if offset == 0 => return Ok(false),
                Ok(0) => return Err(VoxError::UnexpectedEof),
                Ok(len) => offset += len,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(error) => return Err(VoxError::IoErrorKind(error.kind())),
            }
        }
        Ok(true)
    }
}

#[derive(Default)]
struct ByteWriter {
    data: Vec<u8>,
}

impl ByteWriter {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn into_inner(self) -> Vec<u8> {
        self.data
    }

    fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    fn write_bytes(&mut self, value: &[u8]) {
        self.data.extend_from_slice(value);
    }

    fn write_string(&mut self, value: &str) -> Result<(), VoxError> {
        let len = u32::try_from(value.len())
            .map_err(|_| VoxError::InvalidData("string length exceeds u32".into()))?;
        self.write_u32(len);
        self.write_bytes(value.as_bytes());
        Ok(())
    }

    fn patch_u32(&mut self, offset: usize, value: u32) {
        self.data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn begin_chunk(&mut self, id: u32) -> usize {
        let offset = self.data.len();
        self.write_u32(id);
        self.write_u32(0);
        self.write_u32(0);
        offset
    }

    fn finish_chunk(&mut self, offset: usize) -> Result<(), VoxError> {
        let chunk_size = self
            .data
            .len()
            .checked_sub(offset + CHUNK_HEADER_LEN)
            .ok_or_else(|| VoxError::InvalidData("invalid chunk size".into()))?;
        let chunk_size = u32::try_from(chunk_size).map_err(|_| VoxError::FileTooLarge)?;
        self.patch_u32(offset + 4, chunk_size);
        Ok(())
    }
}

#[derive(Default)]
struct Dict {
    entries: DictMap<String, String>,
}

impl Dict {
    fn read<R>(reader: &mut R) -> Result<Self, VoxError>
    where
        R: VoxRead,
    {
        let num_pairs = usize::try_from(reader.read_u32()?)
            .map_err(|_| VoxError::InvalidData("dictionary pair count overflow".into()))?;
        let mut entries = DictMap::new();
        for _ in 0..num_pairs {
            let mut key = reader.read_string_with_length()?;
            key.make_ascii_lowercase();
            let value = reader.read_string_with_length()?;
            entries.insert(key, value);
        }
        Ok(Self { entries })
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .get(&key.to_ascii_lowercase())
            .map(|value| value.as_str())
    }

    fn get_bool(&self, key: &str, default: bool) -> bool {
        self.get(key).map_or(default, |value| value == "1")
    }

    fn get_u32(&self, key: &str, default: u32) -> u32 {
        self.get(key)
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(default)
    }
}

#[derive(Clone, Debug)]
enum TempNode {
    Invalid,
    Transform(TempTransformNode),
    Group(TempGroupNode),
    Shape(TempShapeNode),
}

#[derive(Clone, Debug)]
struct TempTransformNode {
    name: Option<String>,
    transform: Transform,
    child_node_id: u32,
    layer_index: Option<usize>,
    hidden: bool,
    keyframes: Vec<KeyframeTransform>,
    looped: bool,
}

#[derive(Clone, Debug)]
struct TempGroupNode {
    children: Vec<u32>,
}

#[derive(Clone, Debug)]
struct TempShapeNode {
    model_id: usize,
    keyframes: Vec<KeyframeModel>,
    looped: bool,
}

fn ensure_node_slot(nodes: &mut Vec<TempNode>, index: usize) {
    if nodes.len() <= index {
        nodes.resize_with(index + 1, || TempNode::Invalid);
    }
}

fn index_from_u32(raw: u32) -> Option<usize> {
    if raw == INVALID_U32_INDEX {
        None
    } else {
        Some(raw as usize)
    }
}

fn u32_from_index(index: Option<usize>) -> Result<u32, VoxError> {
    match index {
        Some(index) => {
            u32::try_from(index).map_err(|_| VoxError::InvalidData("index exceeds u32".into()))
        }
        None => Ok(INVALID_U32_INDEX),
    }
}

fn usize_to_u32(value: usize) -> Result<u32, VoxError> {
    u32::try_from(value).map_err(|_| VoxError::InvalidData("value exceeds u32".into()))
}

fn parse_vec<T>(value: &str) -> Result<Vec<T>, VoxError>
where
    T: core::str::FromStr,
{
    value
        .split_whitespace()
        .map(|part| {
            part.parse::<T>().map_err(|_| {
                VoxError::InvalidData(format!("failed to parse value \"{part}\" from \"{value}\""))
            })
        })
        .collect()
}

fn parse_transform(
    rotation_string: Option<&str>,
    translation_string: Option<&str>,
) -> Result<Transform, VoxError> {
    const ROW2_INDEX: [u32; 8] = [u32::MAX, u32::MAX, u32::MAX, 2, u32::MAX, 1, 0, u32::MAX];
    const AXES: [Vec3; 4] = [
        Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    ];

    let mut transform = Transform::identity();
    if let Some(rotation_string) = rotation_string {
        let bits = rotation_string.parse::<u32>().map_err(|_| {
            VoxError::InvalidData(format!("invalid rotation value {rotation_string}"))
        })?;
        let row0_index = bits & 3;
        let row1_index = (bits >> 2) & 3;
        let row2_index = ROW2_INDEX[((1 << row0_index) | (1 << row1_index)) as usize];
        if row2_index == u32::MAX {
            return Err(VoxError::InvalidData("invalid packed rotation bits".into()));
        }

        let mut row0 = AXES[row0_index as usize];
        let mut row1 = AXES[row1_index as usize];
        let mut row2 = AXES[row2_index as usize];
        if bits & (1 << 4) != 0 {
            row0 = row0.negate();
        }
        if bits & (1 << 5) != 0 {
            row1 = row1.negate();
        }
        if bits & (1 << 6) != 0 {
            row2 = row2.negate();
        }

        transform.m00 = row0.x;
        transform.m01 = row1.x;
        transform.m02 = row2.x;
        transform.m10 = row0.y;
        transform.m11 = row1.y;
        transform.m12 = row2.y;
        transform.m20 = row0.z;
        transform.m21 = row1.z;
        transform.m22 = row2.z;
    }

    if let Some(translation_string) = translation_string {
        let values = parse_vec::<i32>(translation_string)?;
        if values.len() != 3 {
            return Err(VoxError::InvalidData(format!(
                "invalid translation value {translation_string}"
            )));
        }
        transform.m30 = values[0] as f32;
        transform.m31 = values[1] as f32;
        transform.m32 = values[2] as f32;
    }

    Ok(transform)
}

fn get_rotation_axis_bits(vec: Vec3) -> Result<(u8, bool), VoxError> {
    let values = [vec.x, vec.y, vec.z];
    let mut index = None;
    let mut negative = false;
    for (axis, value) in values.into_iter().enumerate() {
        if value == 1.0 || value == -1.0 {
            index = Some(axis as u8);
            negative = value < 0.0;
        } else if value != 0.0 {
            return Err(VoxError::InvalidData(
                "rotation vector must be cardinal".into(),
            ));
        }
    }
    index
        .map(|index| (index, negative))
        .ok_or_else(|| VoxError::InvalidData("rotation vector cannot be zero".into()))
}

fn make_packed_rotation_from_transform(transform: &Transform) -> Result<u8, VoxError> {
    let row0 = Vec3::new(transform.m00, transform.m10, transform.m20);
    let row1 = Vec3::new(transform.m01, transform.m11, transform.m21);
    let row2 = Vec3::new(transform.m02, transform.m12, transform.m22);
    let (row0_index, row0_negative) = get_rotation_axis_bits(row0)?;
    let (row1_index, row1_negative) = get_rotation_axis_bits(row1)?;
    let (row2_index, row2_negative) = get_rotation_axis_bits(row2)?;
    let orthogonal_mask = (1 << row0_index) | (1 << row1_index) | (1 << row2_index);
    if orthogonal_mask != 0b111 {
        return Err(VoxError::InvalidData(
            "transform rows are not orthogonal cardinal axes".into(),
        ));
    }
    Ok(row0_index
        | (row1_index << 2)
        | if row0_negative { 1 << 4 } else { 0 }
        | if row1_negative { 1 << 5 } else { 0 }
        | if row2_negative { 1 << 6 } else { 0 })
}

fn model_hash(data: &[u8]) -> u32 {
    let mut hash = 0u32;
    for &value in data {
        hash = u32::from(value).wrapping_add(hash.wrapping_mul(65559));
    }
    hash
}

fn models_equal(lhs: &Model, rhs: &Model) -> bool {
    lhs.size_x == rhs.size_x
        && lhs.size_y == rhs.size_y
        && lhs.size_z == rhs.size_z
        && model_hash(&lhs.voxels) == model_hash(&rhs.voxels)
        && lhs.voxels == rhs.voxels
}

fn transform_look_at(eye: Vec3, target: Vec3, up: Vec3) -> Transform {
    let cam_forward = target.sub(eye).normalize();
    let cam_right = up.cross(cam_forward).normalize();
    let cam_up = cam_forward.cross(cam_right);

    Transform {
        m00: cam_right.x,
        m01: cam_up.x,
        m02: -cam_forward.x,
        m03: 0.0,
        m10: cam_right.y,
        m11: cam_up.y,
        m12: -cam_forward.y,
        m13: 0.0,
        m20: cam_right.z,
        m21: cam_up.z,
        m22: -cam_forward.z,
        m23: 0.0,
        m30: -cam_right.dot(eye),
        m31: -cam_up.dot(eye),
        m32: cam_forward.dot(eye),
        m33: 1.0,
    }
}

pub(crate) fn camera_to_transform(camera: &Camera) -> Transform {
    let focus = Vec3::new(camera.focus[0], camera.focus[1], camera.focus[2]);
    let yaw = camera.angle[1] * (PI / 180.0);
    let pitch = camera.angle[0] * (PI / 180.0);
    let camera_pos = Vec3::new(
        focus.x + camera.radius * cos_f32(pitch) * sin_f32(yaw),
        focus.y + camera.radius * sin_f32(pitch),
        focus.z + camera.radius * cos_f32(pitch) * cos_f32(yaw),
    );
    transform_look_at(camera_pos, focus, Vec3::new(0.0, 1.0, 0.0))
}

fn parse_material_type(value: Option<&str>) -> MaterialType {
    match value {
        Some("_metal") => MaterialType::Metal,
        Some("_glass") => MaterialType::Glass,
        Some("_emit") => MaterialType::Emit,
        Some("_blend") => MaterialType::Blend,
        Some("_media") => MaterialType::Media,
        _ => MaterialType::Diffuse,
    }
}

fn parse_media_type(value: Option<&str>) -> MediaType {
    match value {
        Some("_scatter") => MediaType::Scatter,
        Some("_emit") => MediaType::Emit,
        Some("_sss") => MediaType::Sss,
        _ => MediaType::Absorb,
    }
}

fn parse_float(dict: &Dict, key: &str) -> Result<Option<f32>, VoxError> {
    dict.get(key)
        .map(|value| {
            value.parse::<f32>().map_err(|_| {
                VoxError::InvalidData(format!("invalid float value \"{value}\" for {key}"))
            })
        })
        .transpose()
}

fn parse_optional_name(value: Option<&str>) -> Option<String> {
    value.and_then(|value| (!value.is_empty()).then(|| value.to_owned()))
}

fn default_layer_from_chunk() -> Layer {
    Layer {
        name: None,
        color: Rgba {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        },
        hidden: false,
    }
}

fn default_layer_on_missing_scene_layer() -> Layer {
    Layer {
        name: None,
        color: Rgba::default(),
        hidden: false,
    }
}

fn default_root_group(layer_index: Option<usize>) -> Group {
    Group {
        name: None,
        transform: Transform::identity(),
        parent_group_index: None,
        layer_index,
        hidden: false,
        transform_anim: AnimTransform::default(),
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_instances_for_node(
    nodes: &[TempNode],
    node_id: u32,
    models: &[Option<Model>],
    instances: &mut Vec<Instance>,
    groups: &mut Vec<Group>,
    current_group: Option<usize>,
    current_transform: Option<&TempTransformNode>,
    preserve_keyframes: bool,
) -> Result<(), VoxError> {
    let node = nodes
        .get(node_id as usize)
        .ok_or(VoxError::InvalidNodeGraph("node id out of bounds"))?;
    match node {
        TempNode::Invalid => Err(VoxError::InvalidNodeGraph("encountered invalid node")),
        TempNode::Transform(transform_node) => generate_instances_for_node(
            nodes,
            transform_node.child_node_id,
            models,
            instances,
            groups,
            current_group,
            Some(transform_node),
            preserve_keyframes,
        ),
        TempNode::Group(group_node) => {
            let transform_node = current_transform.ok_or(VoxError::InvalidNodeGraph(
                "group node must be preceded by a transform node",
            ))?;
            let next_group_index = groups.len();
            groups.push(Group {
                name: transform_node.name.clone(),
                transform: transform_node.transform,
                parent_group_index: current_group,
                layer_index: transform_node.layer_index,
                hidden: transform_node.hidden,
                transform_anim: if preserve_keyframes {
                    AnimTransform {
                        keyframes: transform_node.keyframes.clone(),
                        looped: transform_node.looped,
                    }
                } else {
                    AnimTransform::default()
                },
            });

            for &child in &group_node.children {
                generate_instances_for_node(
                    nodes,
                    child,
                    models,
                    instances,
                    groups,
                    Some(next_group_index),
                    None,
                    preserve_keyframes,
                )?;
            }
            Ok(())
        }
        TempNode::Shape(shape_node) => {
            let transform_node = current_transform.ok_or(VoxError::InvalidNodeGraph(
                "shape node must be preceded by a transform node",
            ))?;
            let group_index = current_group.ok_or(VoxError::InvalidNodeGraph(
                "shape node must belong to a group",
            ))?;

            if shape_node.model_id >= models.len() {
                return Err(VoxError::InvalidNodeGraph(
                    "shape node references an unknown model",
                ));
            }
            if models[shape_node.model_id].is_none() {
                return Ok(());
            }

            instances.push(Instance {
                name: transform_node.name.clone(),
                transform: transform_node.transform,
                model_index: shape_node.model_id,
                layer_index: transform_node.layer_index,
                group_index: Some(group_index),
                hidden: transform_node.hidden,
                transform_anim: if preserve_keyframes {
                    AnimTransform {
                        keyframes: transform_node.keyframes.clone(),
                        looped: transform_node.looped,
                    }
                } else {
                    AnimTransform::default()
                },
                model_anim: if preserve_keyframes {
                    AnimModel {
                        keyframes: shape_node.keyframes.clone(),
                        looped: shape_node.looped,
                    }
                } else {
                    AnimModel::default()
                },
            });
            Ok(())
        }
    }
}

fn remap_material(material: &mut Material, property: &str, value: f32) {
    match property {
        "_metal" => material.metal = Some(value),
        "_rough" => material.rough = Some(value),
        "_spec" => material.spec = Some(value),
        "_ior" | "_ri" => material.ior = Some(value),
        "_att" => material.att = Some(value),
        "_flux" => material.flux = Some(value),
        "_emit" => material.emit = Some(value),
        "_ldr" => material.ldr = Some(value),
        "_trans" => material.trans = Some(value),
        "_alpha" => material.alpha = Some(value),
        "_d" => material.d = Some(value),
        "_sp" => material.sp = Some(value),
        "_g" => material.g = Some(value),
        "_media" => material.media = Some(value),
        _ => {}
    }
}

fn validate_scene_references(scene: &Scene) -> Result<(), VoxError> {
    for (index, group) in scene.groups.iter().enumerate() {
        if let Some(parent) = group.parent_group_index {
            if parent >= scene.groups.len() {
                return Err(VoxError::IndexOutOfBounds {
                    kind: "group",
                    index: parent,
                    len: scene.groups.len(),
                });
            }
            if parent == index {
                return Err(VoxError::InvalidData("group cannot parent itself".into()));
            }
        }
        if let Some(layer) = group.layer_index
            && layer >= scene.layers.len()
        {
            return Err(VoxError::IndexOutOfBounds {
                kind: "layer",
                index: layer,
                len: scene.layers.len(),
            });
        }
    }

    for instance in &scene.instances {
        if instance.model_index >= scene.models.len() {
            return Err(VoxError::IndexOutOfBounds {
                kind: "model",
                index: instance.model_index,
                len: scene.models.len(),
            });
        }
        if let Some(layer) = instance.layer_index
            && layer >= scene.layers.len()
        {
            return Err(VoxError::IndexOutOfBounds {
                kind: "layer",
                index: layer,
                len: scene.layers.len(),
            });
        }
        if let Some(group) = instance.group_index
            && group >= scene.groups.len()
        {
            return Err(VoxError::IndexOutOfBounds {
                kind: "group",
                index: group,
                len: scene.groups.len(),
            });
        }
        for keyframe in &instance.model_anim.keyframes {
            if keyframe.model_index >= scene.models.len() {
                return Err(VoxError::IndexOutOfBounds {
                    kind: "model",
                    index: keyframe.model_index,
                    len: scene.models.len(),
                });
            }
        }
    }

    Ok(())
}

fn normalize_scene_graph(scene: &Scene, ensure_layer: bool) -> Result<Scene, VoxError> {
    let mut normalized = scene.clone();

    if ensure_layer && normalized.layers.is_empty() {
        normalized
            .layers
            .push(default_layer_on_missing_scene_layer());
        for instance in &mut normalized.instances {
            if instance.layer_index.is_none() {
                instance.layer_index = Some(0);
            }
        }
        for group in &mut normalized.groups {
            if group.layer_index.is_none() {
                group.layer_index = Some(0);
            }
        }
    }

    if normalized.groups.is_empty() {
        if normalized
            .instances
            .iter()
            .any(|instance| instance.group_index.is_some())
        {
            return Err(VoxError::InvalidData(
                "instances reference groups but scene has no groups".into(),
            ));
        }
        let root_layer = if ensure_layer && !normalized.layers.is_empty() {
            Some(0)
        } else {
            None
        };
        normalized.groups.push(default_root_group(root_layer));
        for instance in &mut normalized.instances {
            instance.group_index = Some(0);
        }
        return Ok(normalized);
    }

    let root_groups: Vec<usize> = normalized
        .groups
        .iter()
        .enumerate()
        .filter_map(|(index, group)| group.parent_group_index.is_none().then_some(index))
        .collect();

    if root_groups.is_empty() {
        return Err(VoxError::InvalidData("scene has no root group".into()));
    }

    if root_groups.len() == 1 && root_groups[0] == 0 {
        for instance in &mut normalized.instances {
            if instance.group_index.is_none() {
                instance.group_index = Some(0);
            }
        }
        return Ok(normalized);
    }

    let root_layer = if ensure_layer && !normalized.layers.is_empty() {
        Some(0)
    } else {
        None
    };

    let mut remapped = Scene {
        file_version: normalized.file_version,
        models: normalized.models.clone(),
        instances: Vec::with_capacity(normalized.instances.len()),
        layers: normalized.layers.clone(),
        groups: Vec::with_capacity(normalized.groups.len() + 1),
        color_names: normalized.color_names.clone(),
        palette: normalized.palette.clone(),
        materials: normalized.materials.clone(),
        cameras: normalized.cameras.clone(),
        sun: normalized.sun.clone(),
        anim_range_start: normalized.anim_range_start,
        anim_range_end: normalized.anim_range_end,
    };
    remapped.groups.push(default_root_group(root_layer));

    for group in &normalized.groups {
        let mut group = group.clone();
        group.parent_group_index = group
            .parent_group_index
            .map(|parent| parent + 1)
            .or(Some(0));
        remapped.groups.push(group);
    }

    for instance in &normalized.instances {
        let mut instance = instance.clone();
        instance.group_index = instance.group_index.map(|group| group + 1).or(Some(0));
        remapped.instances.push(instance);
    }

    Ok(remapped)
}

fn write_dict_entries(
    writer: &mut ByteWriter,
    entries: &[(&str, Option<String>)],
) -> Result<(), VoxError> {
    let count = entries.iter().filter(|(_, value)| value.is_some()).count();
    writer.write_u32(usize_to_u32(count)?);
    for (key, value) in entries {
        if let Some(value) = value {
            writer.write_string(key)?;
            writer.write_string(value)?;
        }
    }
    Ok(())
}

fn write_transform_dict_entries(
    writer: &mut ByteWriter,
    transform: &Transform,
) -> Result<(), VoxError> {
    let rotation_bits = make_packed_rotation_from_transform(transform)?;
    let rotation = rotation_bits.to_string();
    let translation = format!(
        "{} {} {}",
        transform.m30 as i32, transform.m31 as i32, transform.m32 as i32
    );
    writer.write_string("_r")?;
    writer.write_string(&rotation)?;
    writer.write_string("_t")?;
    writer.write_string(&translation)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_ntrn_chunk(
    writer: &mut ByteWriter,
    node_id: u32,
    child_node_id: u32,
    name: Option<&str>,
    hidden: bool,
    transform: &Transform,
    layer_index: Option<usize>,
    transform_anim: &AnimTransform,
) -> Result<(), VoxError> {
    let offset = writer.begin_chunk(CHUNK_ID_NTRN);
    writer.write_u32(node_id);
    write_dict_entries(
        writer,
        &[
            ("_name", name.map(|value| value.to_owned())),
            ("_hidden", hidden.then(|| "1".to_owned())),
            ("_loop", transform_anim.looped.then(|| "1".to_owned())),
        ],
    )?;
    writer.write_u32(child_node_id);
    writer.write_u32(INVALID_U32_INDEX);
    writer.write_u32(u32_from_index(layer_index)?);
    if transform_anim.keyframes.is_empty() {
        writer.write_u32(1);
        writer.write_u32(2);
        write_transform_dict_entries(writer, transform)?;
    } else {
        writer.write_u32(usize_to_u32(transform_anim.keyframes.len())?);
        for keyframe in &transform_anim.keyframes {
            writer.write_u32(3);
            write_transform_dict_entries(writer, &keyframe.transform)?;
            writer.write_string("_f")?;
            writer.write_string(&keyframe.frame_index.to_string())?;
        }
    }
    writer.finish_chunk(offset)
}

fn scene_bounding_box_x(scene: &Scene) -> (i32, i32) {
    if scene.instances.is_empty() || scene.models.is_empty() {
        return (0, 0);
    }

    let mut min_x = i32::MAX / 2;
    let mut max_x = i32::MIN / 2;
    for instance in &scene.instances {
        let mut instance_transform = instance.transform;
        let mut parent = instance.group_index;
        while let Some(index) = parent {
            let group = &scene.groups[index];
            instance_transform = instance_transform.multiply(group.transform);
            parent = group.parent_group_index;
        }

        let model = &scene.models[instance.model_index];
        let max_dim = if instance_transform.m00 != 0.0 {
            model.size_x as i32
        } else if instance_transform.m10 != 0.0 {
            model.size_y as i32
        } else if instance_transform.m20 != 0.0 {
            model.size_z as i32
        } else {
            model.size_x as i32
        };
        let half_dim = max_dim / 2;
        let local_min = instance_transform.m30 as i32 - half_dim;
        let local_max = instance_transform.m30 as i32 + half_dim;
        min_x = min_x.min(local_min);
        max_x = max_x.max(local_max);
    }

    (min_x, max_x)
}

fn compute_scene_used_color_index_mask(scene: &Scene) -> [bool; 256] {
    let mut used = [false; 256];
    for model in &scene.models {
        for &voxel in &model.voxels {
            used[usize::from(voxel)] = true;
        }
    }
    used
}

fn find_exact_color_in_palette(
    palette: &[Rgba; 256],
    palette_count: usize,
    color: Rgba,
) -> Option<usize> {
    (1..palette_count).find(|&index| {
        let candidate = palette[index];
        candidate.r == color.r && candidate.g == color.g && candidate.b == color.b
    })
}

fn find_closest_color_in_palette(
    palette: &[Rgba; 256],
    palette_count: usize,
    color: Rgba,
) -> usize {
    let mut best_score = i32::MAX;
    let mut best_index = 1usize;
    for (index, candidate) in palette.iter().enumerate().take(palette_count).skip(1) {
        let r = i32::from(color.r) - i32::from(candidate.r);
        let g = i32::from(color.g) - i32::from(candidate.g);
        let b = i32::from(color.b) - i32::from(candidate.b);
        let score = r * r + g * g + b * b;
        if score < best_score {
            best_score = score;
            best_index = index;
        }
    }
    best_index
}

fn update_master_palette_and_materials_from_scene(
    master_palette: &mut [Rgba; 256],
    master_palette_count: &mut usize,
    scene: &Scene,
    master_materials: &mut [Material; 256],
) -> [u32; 256] {
    let used_mask = compute_scene_used_color_index_mask(scene);
    let mut mapping = [INVALID_U32_INDEX; 256];
    mapping[0] = 0;
    for color_index in 1..256 {
        if !used_mask[color_index] {
            continue;
        }

        let color = scene.palette.colors[color_index];
        let material = scene.materials[color_index].clone();
        let master_index = if let Some(existing) =
            find_exact_color_in_palette(master_palette, *master_palette_count, color)
        {
            existing
        } else if *master_palette_count < 256 {
            let index = *master_palette_count;
            master_palette[index] = color;
            master_materials[index] = material;
            *master_palette_count += 1;
            index
        } else {
            find_closest_color_in_palette(master_palette, *master_palette_count, color)
        };
        mapping[color_index] = master_index as u32;
    }
    mapping
}

pub(crate) fn read_scene(bytes: &[u8], options: ReadOptions) -> Result<Scene, VoxError> {
    let mut reader = ByteReader::new(bytes);
    read_scene_from_vox_read(&mut reader, options)
}

#[cfg(feature = "std")]
pub(crate) fn read_scene_from_reader<R>(
    reader: &mut R,
    options: ReadOptions,
) -> Result<Scene, VoxError>
where
    R: Read,
{
    let mut reader = IoReader::new(reader);
    read_scene_from_vox_read(&mut reader, options)
}

fn read_scene_from_vox_read<R>(reader: &mut R, options: ReadOptions) -> Result<Scene, VoxError>
where
    R: VoxRead,
{
    let file_header = reader.read_u32()?;
    let file_version = reader.read_u32()?;
    if file_header != CHUNK_ID_VOX_ {
        return Err(VoxError::InvalidHeader);
    }
    if file_version != 150 && file_version != 200 {
        return Err(VoxError::UnsupportedVersion(file_version));
    }

    let mut size_x = 0u32;
    let mut size_y = 0u32;
    let mut size_z = 0u32;

    let mut palette = Palette::raw_default();
    let mut materials: [Material; 256] = array::from_fn(|_| Material::default());
    let mut temp_models: Vec<Option<Model>> = Vec::new();
    let mut nodes: Vec<TempNode> = Vec::new();
    let mut layers: Vec<Layer> = Vec::new();
    let mut cameras: Vec<Camera> = Vec::new();
    let mut color_names: Vec<String> = Vec::new();
    let mut found_index_map = false;
    let mut index_map = [0u8; 256];
    let mut sun: Option<Sun> = None;
    let mut anim_range_start = 0u32;
    let mut anim_range_end = 30u32;

    while let Some(chunk_id) = reader.read_u32_or_eof()? {
        let chunk_size = usize::try_from(reader.read_u32()?)
            .map_err(|_| VoxError::InvalidData("chunk size overflow".into()))?;
        let _chunk_child_size = reader.read_u32()?;
        let mut payload_reader = LimitedReader::new(reader, chunk_size);

        match chunk_id {
            CHUNK_ID_MAIN => {}
            CHUNK_ID_SIZE => {
                size_x = payload_reader.read_u32()?;
                size_y = payload_reader.read_u32()?;
                size_z = payload_reader.read_u32()?;
                if size_x == 0 || size_y == 0 || size_z == 0 {
                    return Err(VoxError::InvalidData(
                        "SIZE chunk has zero dimension".into(),
                    ));
                }
            }
            CHUNK_ID_XYZI => {
                if size_x == 0 || size_y == 0 || size_z == 0 {
                    return Err(VoxError::InvalidChunk("XYZI"));
                }
                let num_voxels = usize::try_from(payload_reader.read_u32()?)
                    .map_err(|_| VoxError::InvalidData("voxel count overflow".into()))?;
                if num_voxels == 0 && !options.keep_empty_models_instances {
                    temp_models.push(None);
                    continue;
                }
                let voxel_count = usize::try_from(size_x)
                    .ok()
                    .zip(usize::try_from(size_y).ok())
                    .and_then(|(x, y)| usize::try_from(size_z).ok().map(|z| (x, y, z)))
                    .and_then(|(x, y, z)| x.checked_mul(y).and_then(|xy| xy.checked_mul(z)))
                    .ok_or_else(|| VoxError::InvalidData("voxel grid overflow".into()))?;
                let size_x_usize = usize::try_from(size_x)
                    .map_err(|_| VoxError::InvalidData("size_x overflow".into()))?;
                let size_y_usize = usize::try_from(size_y)
                    .map_err(|_| VoxError::InvalidData("size_y overflow".into()))?;
                let mut voxels = vec![0u8; voxel_count];
                for _ in 0..num_voxels {
                    let x = payload_reader.read_u8()?;
                    let y = payload_reader.read_u8()?;
                    let z = payload_reader.read_u8()?;
                    let color_index = payload_reader.read_u8()?;
                    if u32::from(x) < size_x && u32::from(y) < size_y && u32::from(z) < size_z {
                        let index = usize::from(x)
                            + usize::from(y) * size_x_usize
                            + usize::from(z) * size_x_usize * size_y_usize;
                        voxels[index] = color_index;
                    }
                }
                temp_models.push(Some(Model {
                    size_x,
                    size_y,
                    size_z,
                    voxels,
                }));
            }
            CHUNK_ID_RGBA => {
                for color in &mut palette.colors {
                    color.r = payload_reader.read_u8()?;
                    color.g = payload_reader.read_u8()?;
                    color.b = payload_reader.read_u8()?;
                    color.a = payload_reader.read_u8()?;
                }
            }
            CHUNK_ID_NTRN => {
                let node_id = usize::try_from(payload_reader.read_u32()?)
                    .map_err(|_| VoxError::InvalidData("node id overflow".into()))?;
                let dict = Dict::read(&mut payload_reader)?;
                let child_node_id = payload_reader.read_u32()?;
                let _reserved = payload_reader.read_u32()?;
                let layer_id = payload_reader.read_u32()?;
                let num_frames = usize::try_from(payload_reader.read_u32()?)
                    .map_err(|_| VoxError::InvalidData("frame count overflow".into()))?;
                if num_frames == 0 {
                    return Err(VoxError::InvalidData(
                        "nTRN must contain at least one frame".into(),
                    ));
                }
                let mut keyframes = Vec::with_capacity(num_frames);
                for _ in 0..num_frames {
                    let frame_dict = Dict::read(&mut payload_reader)?;
                    keyframes.push(KeyframeTransform {
                        frame_index: frame_dict.get_u32("_f", 0),
                        transform: parse_transform(frame_dict.get("_r"), frame_dict.get("_t"))?,
                    });
                }
                ensure_node_slot(&mut nodes, node_id);
                nodes[node_id] = TempNode::Transform(TempTransformNode {
                    name: parse_optional_name(dict.get("_name")),
                    transform: keyframes[0].transform,
                    child_node_id,
                    layer_index: index_from_u32(layer_id),
                    hidden: dict.get_bool("_hidden", false),
                    keyframes,
                    looped: dict.get_bool("_loop", false),
                });
            }
            CHUNK_ID_NGRP => {
                let node_id = usize::try_from(payload_reader.read_u32()?)
                    .map_err(|_| VoxError::InvalidData("node id overflow".into()))?;
                let _dict = Dict::read(&mut payload_reader)?;
                let child_count = usize::try_from(payload_reader.read_u32()?)
                    .map_err(|_| VoxError::InvalidData("child count overflow".into()))?;
                let mut children = Vec::with_capacity(child_count);
                for _ in 0..child_count {
                    children.push(payload_reader.read_u32()?);
                }
                ensure_node_slot(&mut nodes, node_id);
                nodes[node_id] = TempNode::Group(TempGroupNode { children });
            }
            CHUNK_ID_NSHP => {
                let node_id = usize::try_from(payload_reader.read_u32()?)
                    .map_err(|_| VoxError::InvalidData("node id overflow".into()))?;
                let dict = Dict::read(&mut payload_reader)?;
                let num_models = usize::try_from(payload_reader.read_u32()?)
                    .map_err(|_| VoxError::InvalidData("shape model count overflow".into()))?;
                if num_models == 0 {
                    return Err(VoxError::InvalidData(
                        "nSHP must contain at least one model".into(),
                    ));
                }
                let mut keyframes = Vec::with_capacity(num_models);
                for _ in 0..num_models {
                    let model_index = usize::try_from(payload_reader.read_u32()?)
                        .map_err(|_| VoxError::InvalidData("model index overflow".into()))?;
                    if model_index >= temp_models.len() {
                        return Err(VoxError::InvalidData(
                            "nSHP references an unknown model".into(),
                        ));
                    }
                    let model_dict = Dict::read(&mut payload_reader)?;
                    keyframes.push(KeyframeModel {
                        frame_index: model_dict.get_u32("_f", 0),
                        model_index,
                    });
                }
                ensure_node_slot(&mut nodes, node_id);
                nodes[node_id] = TempNode::Shape(TempShapeNode {
                    model_id: keyframes[0].model_index,
                    keyframes,
                    looped: dict.get_bool("_loop", false),
                });
            }
            CHUNK_ID_IMAP => {
                for entry in &mut index_map {
                    *entry = payload_reader.read_u8()?;
                }
                found_index_map = true;
            }
            CHUNK_ID_LAYR => {
                let layer_id = payload_reader.read_i32()?;
                if layer_id < 0 {
                    return Err(VoxError::InvalidData("negative layer id".into()));
                }
                let dict = Dict::read(&mut payload_reader)?;
                let _reserved = payload_reader.read_i32()?;
                let layer_id = layer_id as usize;
                if layers.len() <= layer_id {
                    layers.resize_with(layer_id + 1, default_layer_from_chunk);
                }
                let mut layer = default_layer_from_chunk();
                layer.name = parse_optional_name(dict.get("_name"));
                layer.hidden = dict.get_bool("_hidden", false);
                if let Some(color_string) = dict.get("_color") {
                    let rgb = parse_vec::<u32>(color_string)?;
                    if rgb.len() == 3 {
                        layer.color.r = rgb[0] as u8;
                        layer.color.g = rgb[1] as u8;
                        layer.color.b = rgb[2] as u8;
                    }
                }
                layers[layer_id] = layer;
            }
            CHUNK_ID_MATL => {
                let material_id = payload_reader.read_i32()? & 0xff;
                let material_id = material_id as usize;
                let dict = Dict::read(&mut payload_reader)?;
                let material = &mut materials[material_id];
                material.material_type = parse_material_type(dict.get("_type"));
                material.media_type = parse_media_type(dict.get("_media_type"));
                for key in [
                    "_metal", "_rough", "_spec", "_ior", "_att", "_flux", "_emit", "_ldr",
                    "_trans", "_alpha", "_d", "_sp", "_g", "_media",
                ] {
                    if let Some(value) = parse_float(&dict, key)? {
                        remap_material(material, key, value);
                    }
                }
                if let Some(value) = dict.get("_ri") {
                    let ri = value
                        .parse::<f32>()
                        .map_err(|_| VoxError::InvalidData(format!("invalid _ri value {value}")))?;
                    material.ior = Some(ri - 1.0);
                }
            }
            CHUNK_ID_MATT => {
                let material_id = (payload_reader.read_i32()? & 0xff) as usize;
                let material_type = payload_reader.read_i32()?;
                let material_weight = payload_reader.read_f32()?;
                let _property_bits = payload_reader.read_u32()?;
                let material = &mut materials[material_id];
                material.material_type = match material_type {
                    1 => MaterialType::Metal,
                    2 => MaterialType::Glass,
                    3 => MaterialType::Emit,
                    _ => MaterialType::Diffuse,
                };
                match material.material_type {
                    MaterialType::Metal => material.metal = Some(material_weight),
                    MaterialType::Glass => material.trans = Some(material_weight),
                    MaterialType::Emit => material.emit = Some(material_weight),
                    _ => {}
                }
            }
            CHUNK_ID_META => {
                let dict = Dict::read(&mut payload_reader)?;
                if let Some(anim_range) = dict.get("_anim_range") {
                    let values = parse_vec::<i32>(anim_range)?;
                    if values.len() == 2 && values[0] >= 0 && values[1] >= values[0] {
                        anim_range_start = values[0] as u32;
                        anim_range_end = values[1] as u32;
                    } else {
                        return Err(VoxError::InvalidData("invalid animation range".into()));
                    }
                }
            }
            CHUNK_ID_NOTE => {
                let count = usize::try_from(payload_reader.read_u32()?)
                    .map_err(|_| VoxError::InvalidData("note count overflow".into()))?;
                for _ in 0..count {
                    color_names.push(payload_reader.read_string_with_length()?);
                }
            }
            CHUNK_ID_RCAM => {
                let camera_id = payload_reader.read_u32()?;
                let dict = Dict::read(&mut payload_reader)?;
                let mut camera = Camera {
                    camera_id,
                    mode: match dict.get("_mode") {
                        Some("free") => CameraMode::Free,
                        Some("pano") => CameraMode::Pano,
                        Some("iso") => CameraMode::Isometric,
                        Some("orth") => CameraMode::Orthographic,
                        Some("pers") => CameraMode::Perspective,
                        _ => CameraMode::Unknown,
                    },
                    focus: [0.0; 3],
                    angle: [0.0; 3],
                    radius: 0.0,
                    frustum: 0.0,
                    fov: 0,
                };
                if let Some(value) = dict.get("_focus") {
                    let parsed = parse_vec::<f32>(value)?;
                    if parsed.len() == 3 {
                        camera.focus = [parsed[0], parsed[1], parsed[2]];
                    }
                }
                if let Some(value) = dict.get("_angle") {
                    let parsed = parse_vec::<f32>(value)?;
                    if parsed.len() == 3 {
                        camera.angle = [parsed[0], parsed[1], parsed[2]];
                    }
                }
                if let Some(value) = dict.get("_radius") {
                    camera.radius = value.parse::<f32>().map_err(|_| {
                        VoxError::InvalidData(format!("invalid radius value {value}"))
                    })?;
                }
                if let Some(value) = dict.get("_frustum") {
                    camera.frustum = value.parse::<f32>().map_err(|_| {
                        VoxError::InvalidData(format!("invalid frustum value {value}"))
                    })?;
                }
                if let Some(value) = dict.get("_fov") {
                    camera.fov = value
                        .parse::<i32>()
                        .map_err(|_| VoxError::InvalidData(format!("invalid fov value {value}")))?;
                }
                cameras.push(camera);
            }
            CHUNK_ID_ROBJ => {
                let dict = Dict::read(&mut payload_reader)?;
                if dict.get("_type") == Some("_inf") {
                    let mut parsed_sun = Sun {
                        intensity: 0.7,
                        area: 0.7,
                        angle: [50.0, 50.0],
                        rgba: Rgba {
                            r: 0xff,
                            g: 0xff,
                            b: 0xff,
                            a: 0xff,
                        },
                        disk: false,
                    };
                    if let Some(value) = dict.get("_i") {
                        parsed_sun.intensity = value.parse::<f32>().map_err(|_| {
                            VoxError::InvalidData(format!("invalid sun intensity {value}"))
                        })?;
                    }
                    if let Some(value) = dict.get("_area") {
                        parsed_sun.area = value.parse::<f32>().map_err(|_| {
                            VoxError::InvalidData(format!("invalid sun area {value}"))
                        })?;
                    }
                    if let Some(value) = dict.get("_angle") {
                        let parsed = parse_vec::<f32>(value)?;
                        if parsed.len() == 2 {
                            parsed_sun.angle = [parsed[0], parsed[1]];
                        }
                    }
                    if let Some(value) = dict.get("_k") {
                        let parsed = parse_vec::<u32>(value)?;
                        if parsed.len() == 3 {
                            parsed_sun.rgba.r = parsed[0] as u8;
                            parsed_sun.rgba.g = parsed[1] as u8;
                            parsed_sun.rgba.b = parsed[2] as u8;
                        }
                    }
                    parsed_sun.disk = dict.get_bool("_disk", false);
                    sun = Some(parsed_sun);
                }
            }
            _ => {}
        }
        payload_reader.skip_remaining()?;
    }

    let mut instances = Vec::new();
    let mut groups = Vec::new();
    if !nodes.is_empty() {
        if matches!(nodes.first(), Some(TempNode::Invalid) | None) {
            return Err(VoxError::InvalidNodeGraph("root node 0 is missing"));
        }
        generate_instances_for_node(
            &nodes,
            0,
            &temp_models,
            &mut instances,
            &mut groups,
            None,
            None,
            options.preserve_keyframes,
        )?;

        if !options.preserve_groups {
            if options.preserve_keyframes {
                for instance in &mut instances {
                    let mut frame_indices = BTreeSet::new();
                    if instance.transform_anim.keyframes.is_empty() {
                        frame_indices.insert(0);
                    } else {
                        for keyframe in &instance.transform_anim.keyframes {
                            frame_indices.insert(keyframe.frame_index);
                        }
                    }
                    let mut group_index = instance.group_index;
                    while let Some(index) = group_index {
                        let group = &groups[index];
                        for keyframe in &group.transform_anim.keyframes {
                            frame_indices.insert(keyframe.frame_index);
                        }
                        group_index = group.parent_group_index;
                    }

                    let new_keyframes = frame_indices
                        .into_iter()
                        .map(|frame_index| {
                            let mut transform = instance.sample_transform_local(frame_index);
                            let mut group_index = instance.group_index;
                            while let Some(index) = group_index {
                                let group = &groups[index];
                                transform =
                                    transform.multiply(group.sample_transform_local(frame_index));
                                group_index = group.parent_group_index;
                            }
                            KeyframeTransform {
                                frame_index,
                                transform,
                            }
                        })
                        .collect();
                    instance.transform_anim.keyframes = new_keyframes;
                }
            }

            for instance in &mut instances {
                let mut transform = instance.transform;
                let mut group_index = instance.group_index;
                while let Some(index) = group_index {
                    let group = &groups[index];
                    transform = transform.multiply(group.transform);
                    group_index = group.parent_group_index;
                }
                instance.transform = transform;
                instance.group_index = Some(0);
            }

            groups.clear();
            groups.push(default_root_group(Some(0)));
        }
    } else if temp_models.len() == 1 {
        instances.push(Instance {
            name: None,
            transform: Transform::identity(),
            model_index: 0,
            layer_index: Some(0),
            group_index: Some(0),
            hidden: false,
            transform_anim: AnimTransform::default(),
            model_anim: AnimModel::default(),
        });
        groups.push(default_root_group(Some(0)));
    }

    if layers.is_empty() {
        for instance in &mut instances {
            instance.layer_index = Some(0);
        }
        layers.push(default_layer_on_missing_scene_layer());
    }

    if found_index_map {
        let mut inverse = [0u8; 256];
        for (display, &actual) in index_map.iter().enumerate() {
            inverse[actual as usize] = display as u8;
        }

        let old_palette = palette.clone();
        for (index, color) in palette.colors.iter_mut().enumerate() {
            let remapped = index_map[index].wrapping_sub(1) as usize;
            *color = old_palette.colors[remapped];
        }

        let old_materials = materials.clone();
        for (index, material) in materials.iter_mut().enumerate() {
            let remapped_i = (index as u8).wrapping_sub(1) as usize;
            let remapped_index = index_map[remapped_i] as usize;
            *material = old_materials[remapped_index].clone();
        }

        for model in temp_models.iter_mut().flatten() {
            for voxel in &mut model.voxels {
                *voxel = 1u8.wrapping_add(inverse[*voxel as usize]);
            }
        }
    }

    {
        let last = palette.colors[255];
        for index in (1..256).rev() {
            palette.colors[index] = palette.colors[index - 1];
        }
        palette.colors[0] = last;
        palette.colors[0].a = 0;
    }

    if !options.keep_duplicate_models {
        for i in 0..temp_models.len() {
            if temp_models[i].is_none() {
                continue;
            }
            for j in i + 1..temp_models.len() {
                if temp_models[j].is_none() {
                    continue;
                }
                let (Some(lhs), Some(rhs)) = (temp_models[i].as_ref(), temp_models[j].as_ref())
                else {
                    continue;
                };
                let equal = models_equal(lhs, rhs);
                if !equal {
                    continue;
                }
                temp_models[j] = None;
                for instance in &mut instances {
                    if instance.model_index == j {
                        instance.model_index = i;
                    }
                    for keyframe in &mut instance.model_anim.keyframes {
                        if keyframe.model_index == j {
                            keyframe.model_index = i;
                        }
                    }
                }
            }
        }
    }

    if !options.keep_empty_models_instances {
        let mut remap = vec![None; temp_models.len()];
        let mut compacted = Vec::new();
        for (old_index, model) in temp_models.into_iter().enumerate() {
            if let Some(model) = model {
                remap[old_index] = Some(compacted.len());
                compacted.push(model);
            }
        }
        for instance in &mut instances {
            instance.model_index = remap[instance.model_index].ok_or_else(|| {
                VoxError::InvalidData("instance references an empty model".into())
            })?;
            for keyframe in &mut instance.model_anim.keyframes {
                keyframe.model_index = remap[keyframe.model_index].ok_or_else(|| {
                    VoxError::InvalidData("animation references an empty model".into())
                })?;
            }
        }
        temp_models = compacted.into_iter().map(Some).collect();
    }

    let models = temp_models.into_iter().flatten().collect();
    let scene = Scene {
        file_version,
        models,
        instances,
        layers,
        groups,
        color_names,
        palette,
        materials,
        cameras,
        sun,
        anim_range_start,
        anim_range_end,
    };
    validate_scene_references(&scene)?;
    Ok(scene)
}

pub(crate) fn write_scene_with_progress<F>(
    scene: &Scene,
    mut progress: F,
) -> Result<Vec<u8>, VoxError>
where
    F: FnMut(f32) -> bool,
{
    let scene = normalize_scene_graph(scene, true)?;
    validate_scene_references(&scene)?;

    let mut writer = ByteWriter::new();
    writer.write_u32(CHUNK_ID_VOX_);
    let version = if scene.file_version == 0 {
        150
    } else {
        scene.file_version
    };
    writer.write_u32(version);
    writer.write_u32(CHUNK_ID_MAIN);
    writer.write_u32(0);
    writer.write_u32(0);
    let main_child_offset = writer.len();

    if version >= 200 {
        let offset = writer.begin_chunk(CHUNK_ID_META);
        write_dict_entries(
            &mut writer,
            &[(
                "_anim_range",
                Some(format!(
                    "{} {}",
                    scene.anim_range_start, scene.anim_range_end
                )),
            )],
        )?;
        writer.finish_chunk(offset)?;
    }

    for (index, model) in scene.models.iter().enumerate() {
        if model.size_x == 0 || model.size_y == 0 || model.size_z == 0 {
            return Err(VoxError::InvalidData("model has zero dimension".into()));
        }
        if model.size_x > 256 || model.size_y > 256 || model.size_z > 256 {
            return Err(VoxError::InvalidData(
                "model dimensions exceed 256x256x256".into(),
            ));
        }
        let voxel_count = model.voxel_count()?;
        if model.voxels.len() != voxel_count {
            return Err(VoxError::InvalidData(
                "model voxel count does not match dimensions".into(),
            ));
        }

        writer.write_u32(CHUNK_ID_SIZE);
        writer.write_u32(12);
        writer.write_u32(0);
        writer.write_u32(model.size_x);
        writer.write_u32(model.size_y);
        writer.write_u32(model.size_z);

        let solid_voxels = model.solid_voxel_count();
        let xyzi_chunk_size = 4usize
            .checked_add(solid_voxels.checked_mul(4).ok_or(VoxError::FileTooLarge)?)
            .ok_or(VoxError::FileTooLarge)?;
        writer.write_u32(CHUNK_ID_XYZI);
        writer.write_u32(u32::try_from(xyzi_chunk_size).map_err(|_| VoxError::FileTooLarge)?);
        writer.write_u32(0);
        writer.write_u32(usize_to_u32(solid_voxels)?);

        let mut voxel_index = 0usize;
        for z in 0..model.size_z {
            for y in 0..model.size_y {
                for x in 0..model.size_x {
                    let color_index = model.voxels[voxel_index];
                    if color_index != 0 {
                        writer.write_u8(x as u8);
                        writer.write_u8(y as u8);
                        writer.write_u8(z as u8);
                        writer.write_u8(color_index);
                    }
                    voxel_index += 1;
                }
            }
        }

        if !progress((index + 1) as f32 / (scene.models.len() + 1) as f32) {
            return Err(VoxError::WriteCancelled);
        }
    }

    if scene.groups.is_empty() {
        return Err(VoxError::InvalidData(
            "scene must contain at least one group".into(),
        ));
    }

    let first_group_transform_node_id = 0u32;
    let first_group_node_id = first_group_transform_node_id + usize_to_u32(scene.groups.len())?;
    let first_shape_node_id = first_group_node_id + usize_to_u32(scene.groups.len())?;
    let first_instance_transform_node_id =
        first_shape_node_id + usize_to_u32(scene.instances.len())?;

    for (group_index, group) in scene.groups.iter().enumerate() {
        write_ntrn_chunk(
            &mut writer,
            first_group_transform_node_id + usize_to_u32(group_index)?,
            first_group_node_id + usize_to_u32(group_index)?,
            group.name.as_deref(),
            group.hidden,
            &group.transform,
            group.layer_index,
            &group.transform_anim,
        )?;
    }

    for (group_index, group) in scene.groups.iter().enumerate() {
        let offset = writer.begin_chunk(CHUNK_ID_NGRP);
        writer.write_u32(first_group_node_id + usize_to_u32(group_index)?);

        let hidden = group.hidden.then(|| "1".to_owned());
        write_dict_entries(&mut writer, &[("_hidden", hidden)])?;

        let num_child_groups = scene
            .groups
            .iter()
            .filter(|child| child.parent_group_index == Some(group_index))
            .count();
        let num_child_instances = scene
            .instances
            .iter()
            .filter(|instance| instance.group_index == Some(group_index))
            .count();
        writer.write_u32(usize_to_u32(num_child_groups + num_child_instances)?);
        for (child_group_index, child_group) in scene.groups.iter().enumerate() {
            if child_group.parent_group_index == Some(group_index) {
                writer.write_u32(first_group_transform_node_id + usize_to_u32(child_group_index)?);
            }
        }
        for (child_instance_index, instance) in scene.instances.iter().enumerate() {
            if instance.group_index == Some(group_index) {
                writer.write_u32(
                    first_instance_transform_node_id + usize_to_u32(child_instance_index)?,
                );
            }
        }
        writer.finish_chunk(offset)?;
    }

    for (instance_index, instance) in scene.instances.iter().enumerate() {
        let offset = writer.begin_chunk(CHUNK_ID_NSHP);
        writer.write_u32(first_shape_node_id + usize_to_u32(instance_index)?);
        write_dict_entries(
            &mut writer,
            &[("_loop", instance.model_anim.looped.then(|| "1".to_owned()))],
        )?;
        if instance.model_anim.keyframes.is_empty() {
            writer.write_u32(1);
            writer.write_u32(usize_to_u32(instance.model_index)?);
            writer.write_u32(0);
        } else {
            writer.write_u32(usize_to_u32(instance.model_anim.keyframes.len())?);
            for keyframe in &instance.model_anim.keyframes {
                writer.write_u32(usize_to_u32(keyframe.model_index)?);
                write_dict_entries(
                    &mut writer,
                    &[("_f", Some(keyframe.frame_index.to_string()))],
                )?;
            }
        }
        writer.finish_chunk(offset)?;
    }

    for (instance_index, instance) in scene.instances.iter().enumerate() {
        write_ntrn_chunk(
            &mut writer,
            first_instance_transform_node_id + usize_to_u32(instance_index)?,
            first_shape_node_id + usize_to_u32(instance_index)?,
            instance.name.as_deref(),
            instance.hidden,
            &instance.transform,
            instance.layer_index,
            &instance.transform_anim,
        )?;
    }

    for camera in &scene.cameras {
        let offset = writer.begin_chunk(CHUNK_ID_RCAM);
        writer.write_u32(camera.camera_id);
        let mode = match camera.mode {
            CameraMode::Free => "free",
            CameraMode::Pano => "pano",
            CameraMode::Isometric => "iso",
            CameraMode::Orthographic => "orth",
            CameraMode::Unknown | CameraMode::Perspective => "pers",
        };
        write_dict_entries(
            &mut writer,
            &[
                ("_mode", Some(mode.to_owned())),
                (
                    "_focus",
                    Some(format!(
                        "{:.5} {:.5} {:.5}",
                        camera.focus[0], camera.focus[1], camera.focus[2]
                    )),
                ),
                (
                    "_angle",
                    Some(format!(
                        "{:.5} {:.5} {:.5}",
                        camera.angle[0], camera.angle[1], camera.angle[2]
                    )),
                ),
                ("_radius", Some(format!("{:.5}", camera.radius))),
                ("_frustum", Some(format!("{:.5}", camera.frustum))),
                ("_fov", Some(camera.fov.to_string())),
            ],
        )?;
        writer.finish_chunk(offset)?;
    }

    if let Some(sun) = &scene.sun {
        let offset = writer.begin_chunk(CHUNK_ID_ROBJ);
        write_dict_entries(
            &mut writer,
            &[
                ("_type", Some("_inf".to_owned())),
                ("_i", Some(format!("{:.5}", sun.intensity))),
                ("_area", Some(format!("{:.5}", sun.area))),
                (
                    "_angle",
                    Some(format!("{} {}", sun.angle[0] as i32, sun.angle[1] as i32)),
                ),
                (
                    "_k",
                    Some(format!("{} {} {}", sun.rgba.r, sun.rgba.g, sun.rgba.b)),
                ),
                ("_disk", Some(if sun.disk { "1" } else { "0" }.to_owned())),
            ],
        )?;
        writer.finish_chunk(offset)?;
    }

    writer.write_u32(CHUNK_ID_RGBA);
    writer.write_u32(1024);
    writer.write_u32(0);
    for index in 0..256 {
        let color = scene.palette.colors[(index + 1) & 255];
        writer.write_u8(color.r);
        writer.write_u8(color.g);
        writer.write_u8(color.b);
        writer.write_u8(color.a);
    }

    if !scene.color_names.is_empty() {
        let offset = writer.begin_chunk(CHUNK_ID_NOTE);
        writer.write_u32(usize_to_u32(scene.color_names.len())?);
        for name in &scene.color_names {
            writer.write_string(name)?;
        }
        writer.finish_chunk(offset)?;
    }

    for (material_index, material) in scene.materials.iter().enumerate() {
        if !material.has_content() {
            continue;
        }

        let offset = writer.begin_chunk(CHUNK_ID_MATL);
        writer.write_u32(material_index as u32);

        let material_type = match material.material_type {
            MaterialType::Diffuse => "_diffuse",
            MaterialType::Metal => "_metal",
            MaterialType::Glass => "_glass",
            MaterialType::Emit => "_emit",
            MaterialType::Blend => "_blend",
            MaterialType::Media => "_media",
        };

        let media_type = match material.media_type {
            MediaType::Absorb => "_absorb",
            MediaType::Scatter => "_scatter",
            MediaType::Emit => "_emit",
            MediaType::Sss => "_sss",
        };

        let mut entries = vec![("_type", Some(material_type.to_owned()))];
        if matches!(
            material.material_type,
            MaterialType::Glass | MaterialType::Blend | MaterialType::Media
        ) {
            entries.push(("_media_type", Some(media_type.to_owned())));
        }
        for (key, value) in [
            ("_metal", material.metal),
            ("_rough", material.rough),
            ("_spec", material.spec),
            ("_ior", material.ior),
            ("_att", material.att),
            ("_flux", material.flux),
            ("_emit", material.emit),
            ("_ldr", material.ldr),
            ("_trans", material.trans),
            ("_alpha", material.alpha),
            ("_d", material.d),
            ("_sp", material.sp),
            ("_g", material.g),
            ("_media", material.media),
        ] {
            entries.push((key, value.map(|value| format!("{value:.6}"))));
        }
        write_dict_entries(&mut writer, &entries)?;
        writer.finish_chunk(offset)?;
    }

    for (layer_index, layer) in scene.layers.iter().enumerate() {
        let offset = writer.begin_chunk(CHUNK_ID_LAYR);
        writer.write_u32(usize_to_u32(layer_index)?);
        write_dict_entries(
            &mut writer,
            &[
                ("_name", layer.name.clone()),
                ("_hidden", layer.hidden.then(|| "1".to_owned())),
                (
                    "_color",
                    Some(format!(
                        "{} {} {}",
                        layer.color.r, layer.color.g, layer.color.b
                    )),
                ),
            ],
        )?;
        writer.write_u32(INVALID_U32_INDEX);
        writer.finish_chunk(offset)?;
    }

    let total_size = writer.len();
    let child_size = total_size
        .checked_sub(main_child_offset)
        .ok_or(VoxError::FileTooLarge)?;
    let child_size = u32::try_from(child_size).map_err(|_| VoxError::FileTooLarge)?;
    writer.patch_u32(main_child_offset - 4, child_size);

    if !progress(1.0) {
        return Err(VoxError::WriteCancelled);
    }

    Ok(writer.into_inner())
}

#[cfg(feature = "std")]
pub(crate) fn write_scene_to_writer<W, F>(
    scene: &Scene,
    writer: &mut W,
    progress: F,
) -> Result<(), VoxError>
where
    W: std::io::Write,
    F: FnMut(f32) -> bool,
{
    let bytes = write_scene_with_progress(scene, progress)?;
    writer
        .write_all(&bytes)
        .map_err(|error| VoxError::IoErrorKind(error.kind()))
}

pub(crate) fn merge_scenes(scenes: &[&Scene], required_colors: &[Rgba]) -> Result<Scene, VoxError> {
    if required_colors.len() > 255 {
        return Err(VoxError::TooManyRequiredColors(required_colors.len()));
    }

    let mut master_palette = [Rgba::default(); 256];
    let mut master_materials: [Material; 256] = array::from_fn(|_| Material::default());
    let mut master_palette_count = 1usize;
    for &color in required_colors {
        master_palette[master_palette_count] = color;
        master_palette_count += 1;
    }

    let mut models = Vec::new();
    let mut groups = vec![default_root_group(Some(0))];
    let mut instances = Vec::new();
    let layers = vec![Layer {
        name: Some("merged".to_owned()),
        color: Rgba {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        },
        hidden: false,
    }];

    let mut offset_x = 0i32;
    for scene in scenes {
        let scene = normalize_scene_graph(scene, false)?;
        let mapping = update_master_palette_and_materials_from_scene(
            &mut master_palette,
            &mut master_palette_count,
            &scene,
            &mut master_materials,
        );

        let base_model_index = models.len();
        let base_group_index = groups.len();

        for model in &scene.models {
            let mut remapped = model.clone();
            for voxel in &mut remapped.voxels {
                *voxel = mapping[*voxel as usize] as u8;
            }
            models.push(remapped);
        }

        let (scene_min_x, scene_max_x) = scene_bounding_box_x(&scene);
        let scene_offset_x = (offset_x - scene_min_x) as f32;

        if scene.groups.is_empty() || scene.groups[0].parent_group_index.is_some() {
            return Err(VoxError::InvalidData(
                "merged scenes must have a root group at index 0".into(),
            ));
        }

        for group in scene.groups.iter().skip(1) {
            let mut merged_group = group.clone();
            merged_group.layer_index = Some(0);
            merged_group.parent_group_index = match merged_group.parent_group_index {
                Some(0) => Some(0),
                Some(parent) => Some(base_group_index + parent - 1),
                None => Some(0),
            };
            if merged_group.parent_group_index == Some(0) {
                merged_group.transform.m30 += scene_offset_x;
            }
            groups.push(merged_group);
        }

        for instance in &scene.instances {
            let mut merged_instance = instance.clone();
            merged_instance.layer_index = Some(0);
            merged_instance.group_index = match merged_instance.group_index {
                Some(0) => Some(0),
                Some(group_index) => Some(base_group_index + group_index - 1),
                None => Some(0),
            };
            merged_instance.model_index += base_model_index;
            for keyframe in &mut merged_instance.model_anim.keyframes {
                keyframe.model_index += base_model_index;
            }
            if merged_instance.group_index == Some(0) {
                merged_instance.transform.m30 += scene_offset_x;
            }
            instances.push(merged_instance);
        }

        offset_x += scene_max_x - scene_min_x;
        offset_x += 4;
    }

    for color in &mut master_palette[master_palette_count..] {
        *color = Rgba {
            r: 255,
            g: 0,
            b: 255,
            a: 255,
        };
    }

    let merged = Scene {
        file_version: 0,
        models,
        instances,
        layers,
        groups,
        color_names: Vec::new(),
        palette: Palette {
            colors: master_palette,
        },
        materials: master_materials,
        cameras: Vec::new(),
        sun: None,
        anim_range_start: 0,
        anim_range_end: 30,
    };
    validate_scene_references(&merged)?;
    Ok(merged)
}
