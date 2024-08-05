use image::{GenericImageView, ImageBuffer, Rgba};
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use walkdir::WalkDir;
use chrono::Local;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = read_input("Path to icons: ")?;
    let output = read_input("Output path: ")?;
    let output_module = read_input("Module output path: ")?;

    let icon_files = collect_icon_files(&input)?;
    if icon_files.is_empty() {
        eprintln!("No icon files found in the specified directory.");
        return Ok(());
    }

    let (spritesheet, icon_info) = create_spritesheet(&icon_files)?;
    generate_luau_module(&output_module, &icon_info)?;
    spritesheet.save(&output)?;

    println!("Spritesheet and Luau module generated successfully!");
    Ok(())
}

fn read_input(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn collect_icon_files(input_folder: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut icon_files: Vec<PathBuf> = WalkDir::new(input_folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    icon_files.sort();
    Ok(icon_files)
}

fn create_spritesheet(icon_files: &[PathBuf]) -> Result<(ImageBuffer<Rgba<u8>, Vec<u8>>, Vec<IconInfo>), Box<dyn std::error::Error>> {
    let icon_count = icon_files.len();
    let icons_per_row = (icon_count as f32).sqrt().ceil() as u32;
    let rows = ((icon_count as f32) / (icons_per_row as f32)).ceil() as u32;

    let first_icon = image::open(&icon_files[0])?;
    let (icon_width, icon_height) = first_icon.dimensions();

    let mut spritesheet = ImageBuffer::new(icon_width * icons_per_row, icon_height * rows);
    let mut icon_info = Vec::new();

    for (index, icon_path) in icon_files.iter().enumerate() {
        let icon = image::open(icon_path)?;
        let x = (index as u32 % icons_per_row) * icon_width;
        let y = (index as u32 / icons_per_row) * icon_height;

        for (i, j, pixel) in icon.pixels() {
            spritesheet.put_pixel(x + i, y + j, pixel);
        }

        let icon_name = icon_path.file_stem().unwrap().to_str().unwrap().to_string();
        icon_info.push(IconInfo {
            name: icon_name,
            x,
            y,
            width: icon_width,
            height: icon_height,
        });
    }

    Ok((spritesheet, icon_info))
}

fn generate_luau_module(output_module: &str, icon_info: &[IconInfo]) -> Result<(), Box<dyn std::error::Error>> {
    let mut module_file = File::create(output_module)?;
    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    writeln!(module_file, "--[[ module generated from ssheet.rs :: generated at {} ]]\n\nlocal Icons = {{}}", current_time)?;

    for info in icon_info {
        writeln!(
            module_file,
            "Icons['{}'] = {{ x = {}, y = {}, width = {}, height = {} }}",
            info.name, info.x, info.y, info.width, info.height
        )?;
    }

    writeln!(module_file, "\nreturn Icons")?;
    Ok(())
}

struct IconInfo {
    name: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
