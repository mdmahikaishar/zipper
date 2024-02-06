use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{self, ZipArchive, ZipWriter};

pub fn zip(src_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut zip_writer = ZipWriter::new(output_file);

    let src_file_path = Path::new(&src_path);

    if src_file_path.is_dir() {
        zip_recursive(src_path, src_path.clone(), &mut zip_writer)?;
    } else {
        zip_write_file(
            &mut zip_writer,
            &src_file_path.to_path_buf(),
            src_file_path.to_str().unwrap(),
            generate_file_options(),
        )?;
    }

    zip_writer.finish()?;

    Ok(())
}

fn generate_file_options() -> FileOptions {
    FileOptions::default().compression_method(zip::CompressionMethod::Stored)
}

fn zip_recursive<W: Write + Seek>(
    src_path: &str,
    current_path: &str,
    zip_writer: &mut ZipWriter<W>,
) -> Result<(), Box<dyn Error>> {
    let file_option = generate_file_options();
    // .unix_permissions(fs::metadata(current_path)?.permissions());

    for entry in fs::read_dir(current_path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let relative_path = entry_path.strip_prefix(src_path)?.to_str().unwrap();

        if entry_path.is_dir() {
            zip_writer.add_directory(relative_path, file_option)?;
            zip_recursive(src_path, entry_path.to_str().unwrap(), zip_writer)?;
            continue;
        }

        zip_write_file(zip_writer, &entry_path, relative_path, file_option)?;
    }

    Ok(())
}

fn zip_write_file<W: Write + Seek>(
    zip_writer: &mut ZipWriter<W>,
    file_path: &PathBuf,
    relative_path: &str,
    options: FileOptions,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();
    let mut file = File::open(file_path)?;
    file.read_to_end(&mut buffer)?;

    zip_writer.start_file(relative_path, options)?;
    zip_writer.write_all(&buffer)?;

    Ok(())
}

pub fn unzip(src_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let output_path = PathBuf::from(output_path);
    let zip_file = File::open(src_path)?;
    let mut zip_archive = ZipArchive::new(zip_file)?;

    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;

        let file_enclosed_name = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        // Normalize path
        let mut file_path = output_path.clone();
        file_path.push(
            file_enclosed_name
                .to_str()
                .unwrap()
                .replace('/', std::path::MAIN_SEPARATOR_STR),
        );

        if file.is_dir() {
            fs::create_dir_all(&file_path)?;
        } else {
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(&file_path)?;
                }
            }

            let mut output_file = File::create(&file_path)?;
            io::copy(&mut file, &mut output_file)?;
        }

        // set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&file_path, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}
