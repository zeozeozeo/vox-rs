use std::fs;
use std::path::PathBuf;

use crate::types::compute_looped_frame_index;

use super::*;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join(name)
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    fs::read(fixture_path(name)).expect("fixture must be readable")
}

fn read_fixture(name: &str, options: ReadOptions) -> Scene {
    Scene::read_with_options(&fixture_bytes(name), options).expect("fixture must parse")
}

fn full_options() -> ReadOptions {
    ReadOptions {
        preserve_groups: true,
        preserve_keyframes: true,
        keep_empty_models_instances: true,
        keep_duplicate_models: true,
    }
}

fn assert_f32_eq(lhs: f32, rhs: f32) {
    let delta = (lhs - rhs).abs();
    assert!(
        delta <= 1e-4,
        "expected {lhs} ~= {rhs} but delta was {delta}"
    );
}

fn assert_transform_eq(lhs: &Transform, rhs: &Transform) {
    assert_f32_eq(lhs.m00, rhs.m00);
    assert_f32_eq(lhs.m01, rhs.m01);
    assert_f32_eq(lhs.m02, rhs.m02);
    assert_f32_eq(lhs.m03, rhs.m03);
    assert_f32_eq(lhs.m10, rhs.m10);
    assert_f32_eq(lhs.m11, rhs.m11);
    assert_f32_eq(lhs.m12, rhs.m12);
    assert_f32_eq(lhs.m13, rhs.m13);
    assert_f32_eq(lhs.m20, rhs.m20);
    assert_f32_eq(lhs.m21, rhs.m21);
    assert_f32_eq(lhs.m22, rhs.m22);
    assert_f32_eq(lhs.m23, rhs.m23);
    assert_f32_eq(lhs.m30, rhs.m30);
    assert_f32_eq(lhs.m31, rhs.m31);
    assert_f32_eq(lhs.m32, rhs.m32);
    assert_f32_eq(lhs.m33, rhs.m33);
}

fn assert_material_eq(lhs: &Material, rhs: &Material) {
    assert_eq!(lhs.material_type, rhs.material_type);
    assert_eq!(lhs.media_type, rhs.media_type);
    for (lhs, rhs) in [
        (lhs.metal, rhs.metal),
        (lhs.rough, rhs.rough),
        (lhs.spec, rhs.spec),
        (lhs.ior, rhs.ior),
        (lhs.att, rhs.att),
        (lhs.flux, rhs.flux),
        (lhs.emit, rhs.emit),
        (lhs.ldr, rhs.ldr),
        (lhs.trans, rhs.trans),
        (lhs.alpha, rhs.alpha),
        (lhs.d, rhs.d),
        (lhs.sp, rhs.sp),
        (lhs.g, rhs.g),
        (lhs.media, rhs.media),
    ] {
        match (lhs, rhs) {
            (Some(lhs), Some(rhs)) => assert_f32_eq(lhs, rhs),
            (None, None) => {}
            _ => panic!("material option mismatch"),
        }
    }
}

fn assert_scene_semantics(lhs: &Scene, rhs: &Scene) {
    assert_eq!(lhs.file_version, rhs.file_version);
    assert_eq!(lhs.anim_range_start, rhs.anim_range_start);
    assert_eq!(lhs.anim_range_end, rhs.anim_range_end);
    assert_eq!(lhs.models, rhs.models);
    assert_eq!(lhs.layers, rhs.layers);
    assert_eq!(lhs.color_names, rhs.color_names);
    assert_eq!(lhs.palette, rhs.palette);
    assert_eq!(lhs.cameras.len(), rhs.cameras.len());
    assert_eq!(lhs.sun.is_some(), rhs.sun.is_some());
    assert_eq!(lhs.instances.len(), rhs.instances.len());
    assert_eq!(lhs.groups.len(), rhs.groups.len());

    for (lhs, rhs) in lhs.materials.iter().zip(rhs.materials.iter()) {
        assert_material_eq(lhs, rhs);
    }

    for (lhs, rhs) in lhs.cameras.iter().zip(rhs.cameras.iter()) {
        assert_eq!(lhs.camera_id, rhs.camera_id);
        assert_eq!(lhs.mode, rhs.mode);
        for (lhs, rhs) in lhs.focus.iter().zip(rhs.focus.iter()) {
            assert_f32_eq(*lhs, *rhs);
        }
        for (lhs, rhs) in lhs.angle.iter().zip(rhs.angle.iter()) {
            assert_f32_eq(*lhs, *rhs);
        }
        assert_f32_eq(lhs.radius, rhs.radius);
        assert_f32_eq(lhs.frustum, rhs.frustum);
        assert_eq!(lhs.fov, rhs.fov);
    }

    if let (Some(lhs), Some(rhs)) = (&lhs.sun, &rhs.sun) {
        assert_f32_eq(lhs.intensity, rhs.intensity);
        assert_f32_eq(lhs.area, rhs.area);
        assert_f32_eq(lhs.angle[0], rhs.angle[0]);
        assert_f32_eq(lhs.angle[1], rhs.angle[1]);
        assert_eq!(lhs.rgba, rhs.rgba);
        assert_eq!(lhs.disk, rhs.disk);
    }

    for (lhs, rhs) in lhs.groups.iter().zip(rhs.groups.iter()) {
        assert_eq!(lhs.name, rhs.name);
        assert_eq!(lhs.parent_group_index, rhs.parent_group_index);
        assert_eq!(lhs.layer_index, rhs.layer_index);
        assert_eq!(lhs.hidden, rhs.hidden);
        assert_eq!(lhs.transform_anim.looped, rhs.transform_anim.looped);
        assert_transform_eq(&lhs.transform, &rhs.transform);
        assert_eq!(
            lhs.transform_anim.keyframes.len(),
            rhs.transform_anim.keyframes.len()
        );
        for (lhs, rhs) in lhs
            .transform_anim
            .keyframes
            .iter()
            .zip(rhs.transform_anim.keyframes.iter())
        {
            assert_eq!(lhs.frame_index, rhs.frame_index);
            assert_transform_eq(&lhs.transform, &rhs.transform);
        }
    }

    for (lhs, rhs) in lhs.instances.iter().zip(rhs.instances.iter()) {
        assert_eq!(lhs.name, rhs.name);
        assert_eq!(lhs.model_index, rhs.model_index);
        assert_eq!(lhs.layer_index, rhs.layer_index);
        assert_eq!(lhs.group_index, rhs.group_index);
        assert_eq!(lhs.hidden, rhs.hidden);
        assert_eq!(lhs.transform_anim.looped, rhs.transform_anim.looped);
        assert_eq!(lhs.model_anim.looped, rhs.model_anim.looped);
        assert_transform_eq(&lhs.transform, &rhs.transform);
        assert_eq!(
            lhs.transform_anim.keyframes.len(),
            rhs.transform_anim.keyframes.len()
        );
        for (lhs, rhs) in lhs
            .transform_anim
            .keyframes
            .iter()
            .zip(rhs.transform_anim.keyframes.iter())
        {
            assert_eq!(lhs.frame_index, rhs.frame_index);
            assert_transform_eq(&lhs.transform, &rhs.transform);
        }
        assert_eq!(lhs.model_anim.keyframes, rhs.model_anim.keyframes);
    }
}

#[test]
fn looped_frame_index() {
    assert_eq!(compute_looped_frame_index(0, 0, 0), 0);
    assert_eq!(compute_looped_frame_index(0, 0, 1), 0);
    assert_eq!(compute_looped_frame_index(0, 0, 15), 0);

    assert_eq!(compute_looped_frame_index(1, 1, 0), 1);
    assert_eq!(compute_looped_frame_index(1, 1, 1), 1);
    assert_eq!(compute_looped_frame_index(1, 1, 15), 1);

    assert_eq!(compute_looped_frame_index(0, 9, 0), 0);
    assert_eq!(compute_looped_frame_index(0, 9, 4), 4);
    assert_eq!(compute_looped_frame_index(0, 9, 9), 9);
    assert_eq!(compute_looped_frame_index(0, 9, 10), 0);
    assert_eq!(compute_looped_frame_index(0, 9, 11), 1);
    assert_eq!(compute_looped_frame_index(0, 9, 14), 4);
    assert_eq!(compute_looped_frame_index(0, 9, 19), 9);
    assert_eq!(compute_looped_frame_index(0, 9, 21), 1);

    assert_eq!(compute_looped_frame_index(4, 13, 0), 10);
    assert_eq!(compute_looped_frame_index(4, 13, 3), 13);
    assert_eq!(compute_looped_frame_index(4, 13, 4), 4);
    assert_eq!(compute_looped_frame_index(4, 13, 5), 5);
    assert_eq!(compute_looped_frame_index(4, 13, 12), 12);
    assert_eq!(compute_looped_frame_index(4, 13, 13), 13);
    assert_eq!(compute_looped_frame_index(4, 13, 14), 4);
    assert_eq!(compute_looped_frame_index(4, 13, 21), 11);
}

#[test]
fn animation_sampling() {
    let transform_a = Transform {
        m30: 4.0,
        m31: -4.0,
        m32: 8.0,
        ..Transform::identity()
    };
    let transform_b = Transform {
        m00: 0.0,
        m01: 1.0,
        m10: -1.0,
        m11: 0.0,
        m30: 13.0,
        m31: 7.0,
        m32: -9.0,
        ..Transform::identity()
    };

    let instance = Instance {
        name: None,
        transform: transform_a,
        model_index: 2,
        layer_index: None,
        group_index: None,
        hidden: false,
        transform_anim: AnimTransform {
            keyframes: vec![
                KeyframeTransform {
                    frame_index: 4,
                    transform: transform_a,
                },
                KeyframeTransform {
                    frame_index: 13,
                    transform: transform_b,
                },
            ],
            looped: true,
        },
        model_anim: AnimModel {
            keyframes: vec![
                KeyframeModel {
                    frame_index: 4,
                    model_index: 2,
                },
                KeyframeModel {
                    frame_index: 13,
                    model_index: 9,
                },
            ],
            looped: true,
        },
    };

    let sampled = instance.sample_transform_local(8);
    assert_eq!(sampled.m00, transform_a.m00);
    assert_eq!(sampled.m01, transform_a.m01);
    assert_eq!(sampled.m10, transform_a.m10);
    assert_eq!(sampled.m11, transform_a.m11);
    assert_eq!(sampled.m30, 8.0);
    assert_eq!(sampled.m31, 0.0);
    assert_eq!(sampled.m32, 0.0);

    assert_eq!(instance.sample_model_index(8), 2);
    assert_eq!(instance.sample_model_index(14), 2);
    assert_eq!(instance.sample_model_index(13), 9);
}

#[test]
fn reads_meta_chunk_fixture() {
    let scene = read_fixture("test_meta_chunk.vox", full_options());

    assert_eq!(scene.anim_range_start, 7);
    assert_eq!(scene.anim_range_end, 36);
    assert_eq!(scene.file_version, 200);
    assert_eq!(scene.cameras.len(), 10);
    assert_eq!(scene.models.len(), 1);
    assert_eq!(scene.instances.len(), 1);
    assert_eq!(scene.layers.len(), 16);
    assert_eq!(scene.groups.len(), 1);
    assert_eq!(scene.color_names.len(), 32);
    assert_eq!(scene.color_names[0], "NOTE");

    let model = &scene.models[0];
    assert_eq!(model.size_x, 40);
    assert_eq!(model.size_y, 40);
    assert_eq!(model.size_z, 40);
    assert_eq!(model.solid_voxel_count(), 64000);

    let layer = &scene.layers[0];
    assert_eq!(layer.name, None);
    assert!(!layer.hidden);
    assert_eq!(
        layer.color,
        Rgba {
            r: 255,
            g: 204,
            b: 153,
            a: 255
        }
    );

    let group = &scene.groups[0];
    assert_eq!(group.name, None);
    assert_eq!(group.parent_group_index, None);
    assert_eq!(group.layer_index, None);
    assert!(!group.hidden);
}

#[test]
fn reads_group_fixture() {
    let scene = read_fixture("test_groups.vox", full_options());
    assert_eq!(scene.groups.len(), 5);
    assert_eq!(scene.file_version, 150);
    assert_eq!(scene.groups[3].name.as_deref(), Some("characters"));
    assert_eq!(scene.groups[4].name.as_deref(), Some("text"));
}

#[test]
fn round_trips_meta_fixture() {
    let original = read_fixture("test_meta_chunk.vox", full_options());
    let bytes = original.write().expect("scene must serialize");
    let reparsed = Scene::read_with_options(&bytes, full_options()).expect("round trip must parse");
    assert_scene_semantics(&original, &reparsed);
}

#[test]
fn merge_supports_write_and_readback() {
    let left = read_fixture("chr_old.vox", full_options());
    let right = read_fixture("chr_sword.vox", full_options());
    let merged = Scene::merge(&[&left, &right], &[]).expect("merge must succeed");

    assert_eq!(merged.models.len(), left.models.len() + right.models.len());
    assert_eq!(
        merged.instances.len(),
        left.instances.len() + right.instances.len()
    );
    assert_eq!(merged.layers.len(), 1);
    assert_eq!(merged.layers[0].name.as_deref(), Some("merged"));
    assert!(!merged.groups.is_empty());
    assert_eq!(merged.groups[0].parent_group_index, None);

    let bytes = merged.write().expect("merged scene must serialize");
    let reparsed =
        Scene::read_with_options(&bytes, full_options()).expect("merged round trip must parse");
    assert_eq!(reparsed.layers[0].name.as_deref(), Some("merged"));
    assert_eq!(reparsed.instances.len(), merged.instances.len());
    assert_eq!(reparsed.models.len(), merged.models.len());
}
