use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use vox_rs::Scene;

fn read_scene(path: &Path) -> Result<Scene, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    Ok(Scene::read(&mut reader)?)
}

fn write_scene(path: &Path, scene: &Scene) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    scene.write_to(&mut writer)?;
    Ok(())
}

fn print_help(program: &str) {
    println!("voxmerge");
    println!("usage:");
    println!("  {program} <outputfilename.vox> <input0.vox> <input1.vox> ...");
    println!();
    println!("example:");
    println!(
        "  {program} merged.vox testdata/chr_old.vox testdata/chr_rain.vox testdata/chr_sword.vox"
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    let program = args
        .first()
        .and_then(|value| Path::new(value).file_name())
        .and_then(|value| value.to_str())
        .unwrap_or("voxmerge");

    if args.len() <= 2 {
        eprintln!("ERROR: not enough arguments provided");
        print_help(program);
        std::process::exit(2);
    }

    let output = PathBuf::from(&args[1]);
    let inputs: Vec<PathBuf> = args[2..].iter().map(PathBuf::from).collect();

    let loaded_scenes = inputs
        .iter()
        .map(|path| {
            read_scene(path)
                .map_err(|error| format!("failed to load scene from {}: {error}", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let scene_refs: Vec<&Scene> = loaded_scenes.iter().collect();
    let merged = Scene::merge(&scene_refs, &[])?;
    write_scene(&output, &merged)?;

    println!(
        "merged {} scene(s) into {}",
        scene_refs.len(),
        output.display()
    );

    Ok(())
}
