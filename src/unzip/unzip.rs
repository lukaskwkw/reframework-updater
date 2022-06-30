use crate::unzip::UnzipError::UnzipError;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub fn unzip(
    files_to_skip: Vec<&str>,
    destination: Option<impl AsRef<Path>>,
    dry_run: bool,
) -> Result<bool, Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename>", args[0]);
        return Ok(false);
    }
    let fname = std::path::Path::new(&*args[1]);
    let file = fs::File::open(&fname)?;

    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let final_path: PathBuf = match destination.as_ref() {
            Some(path) => path.as_ref().join(outpath),
            None => outpath,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, final_path.display());
            fs::create_dir_all(&final_path)?;
        } else {
            let file_name = match final_path.file_name() {
                Some(it) => it,
                None => return Err(Box::new(UnzipError::OutpathFileName)),
            };

            if files_to_skip.iter().any(|fname| &file_name == fname) {
                continue;
            }

            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                final_path.display(),
                file.size()
            );

            if dry_run {
                println!("Nothing copied - dryRun!");
                continue;
            }
            if let Some(p) = final_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }

            let mut outfile = fs::File::create(&final_path)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&final_path, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(true)
}
