mod libs;

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        eprintln!("Not enough arguments.");
        std::process::exit(0);
    }

    match &args[1][..] {
        "-h" | "--help" => {
            println!(
                "Usage:
            \r<src>              : zip a file. 
            \r<-u, --unzip> <src>: unzip a file.
            \r<-rd, --read-dir>  : read dir zipping.
            \r<-h, --help>       : more information.
            "
            );
        }
        "-rd" | "--read-dir" if args.len() == 2 => {
            let current_dir = std::env::current_dir().unwrap();
            let current_dir = current_dir.to_str().unwrap();

            for entry in fs::read_dir(current_dir).unwrap() {
                let entry = entry.unwrap();
                let entry_path = entry.path();
                let file_path = entry_path.to_str().unwrap();
                let file_name = entry_path.file_stem().unwrap();
                let file_name = file_name.to_str().unwrap();

                if entry_path.is_file() {
                    continue;
                }

                libs::zip(file_path, &format!("{}.zip", file_name)).unwrap();
            }
        }
        "-u" | "--unzip" if args.len() == 3 => {
            let src_path = std::path::Path::new(&args[2]);
            let file_name = src_path.file_stem().unwrap();
            let file_name = file_name.to_str().unwrap();

            libs::unzip(&args[2], file_name).unwrap();
        }
        _ if args.len() == 2 => {
            let src_path = std::path::Path::new(&args[1]);

            if !src_path.exists() {
                eprintln!("File dosen't exists.");
                std::process::exit(0);
            }

            let file_name = src_path.file_stem().unwrap();
            let file_name = file_name.to_str().unwrap();
            let output_path = format!("{}.zip", file_name);

            libs::zip(&args[1], &output_path).unwrap();
        }
        _ => {
            eprintln!("Too many arguments.");
        }
    }
}
