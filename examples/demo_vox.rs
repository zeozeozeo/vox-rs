use std::env;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use vox_rs::{ReadOptions, Scene};

fn load_scene(path: &Path, options: ReadOptions) -> Result<Scene, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    Ok(Scene::read_with_options(&bytes, options)?)
}

fn save_scene(path: &Path, scene: &Scene) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    scene.write_to(&mut writer)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os();
    let _program = args.next();
    let input = args
        .next()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new("testdata").join("test_groups.vox"));
    let output = args
        .next()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new("saved.vox").to_path_buf());

    let scene = load_scene(
        &input,
        ReadOptions {
            preserve_groups: true,
            preserve_keyframes: true,
            ..ReadOptions::default()
        },
    )?;

    println!("#layers: {}", scene.layers.len());
    for (index, layer) in scene.layers.iter().enumerate() {
        println!(
            "layer[{index},name={}] is {}",
            layer.name.as_deref().unwrap_or(""),
            if layer.hidden { "hidden" } else { "shown" }
        );
    }

    println!("#groups: {}", scene.groups.len());
    for (index, group) in scene.groups.iter().enumerate() {
        let layer_name = group
            .layer_index
            .and_then(|layer_index| scene.layers.get(layer_index))
            .and_then(|layer| layer.name.as_deref())
            .unwrap_or("");
        println!(
            "group[{index}] has parent group {}, is part of layer[{},name={layer_name}] and is {}",
            group
                .parent_group_index
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            group
                .layer_index
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            if group.hidden { "hidden" } else { "shown" }
        );
    }

    println!("# instances: {}", scene.instances.len());
    for (index, instance) in scene.instances.iter().enumerate() {
        let layer_name = instance
            .layer_index
            .and_then(|layer_index| scene.layers.get(layer_index))
            .and_then(|layer| layer.name.as_deref())
            .unwrap_or("");

        println!(
            "instance[{index},name={}] at position ({:.0},{:.0},{:.0}) uses model {}, is in layer[{}, name='{layer_name}'], group {}, and is {}",
            instance.name.as_deref().unwrap_or(""),
            instance.transform.m30,
            instance.transform.m31,
            instance.transform.m32,
            instance.model_index,
            instance
                .layer_index
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            instance
                .group_index
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            if instance.hidden { "hidden" } else { "shown" }
        );
    }

    println!("# models: {}", scene.models.len());
    for (index, model) in scene.models.iter().enumerate() {
        let total_voxel_count = model.voxels.len();
        let solid_voxel_count = model.solid_voxel_count();
        println!(
            "model[{index}] has dimension {}x{}x{}, with {solid_voxel_count} solid voxels of the total {total_voxel_count} voxels",
            model.size_x, model.size_y, model.size_z
        );
    }

    save_scene(&output, &scene)?;
    println!("saved {}", output.display());

    Ok(())
}
