use std::{env, path::Path, process::Command};

fn main()
{
    let out_dir = env::var("OUT_DIR").unwrap();
    let tailwindcss_path = "tailwindcss";
    let input_css = "public/input.css";
    let output_css = "public/tailwind_generated.css";

    let _ = std::fs::create_dir_all(Path::new(&out_dir).join("public"));

    // Skip if the CSS already exists (e.g. in Docker builds without tailwindcss)
    if Path::new(output_css).exists() && std::fs::metadata(output_css).map(|m| m.len() > 0).unwrap_or(false) {
        println!("Tailwind CSS already exists, skipping generation");
        return;
    }

    // Try to run tailwind, but don't fail if it's not installed
    let status = Command::new(tailwindcss_path)
        .arg("-i")
        .arg(input_css)
        .arg("-o")
        .arg(output_css)
        .status();

    match status {
        Ok(s) if s.success() => println!("Tailwind CSS compiled successfully"),
        Ok(_) => eprintln!("Warning: Tailwind CSS build failed, using existing CSS"),
        Err(_) => eprintln!("Warning: tailwindcss not found, using existing CSS"),
    }
}