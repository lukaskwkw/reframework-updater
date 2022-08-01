pub mod unzip {
    use error_stack::IntoReport;
    use error_stack::Report;
    use error_stack::Result;
    use error_stack::ResultExt;
    use log::debug;
    use log::info;
    use log::trace;
    use std::ffi::OsStr;
    use std::fs;
    use std::io;
    use std::path::Path;
    use std::path::PathBuf;

    pub fn unzip<F>(
        file_to_unzip: impl AsRef<Path>,
        destination: impl AsRef<Path>,
        should_skip_this_file: Option<F>,
    ) -> Result<bool, UnzipError>
    where
        F: Fn(&OsStr) -> bool,
    {
        info!(
            "Unzipping files from {} to {}",
            file_to_unzip.as_ref().display(),
            destination.as_ref().display()
        );
        let file = fs::File::open(&file_to_unzip)
            .report()
            .change_context(UnzipError::other)?;

        let mut archive = zip::ZipArchive::new(file)
            .report()
            .change_context(UnzipError::zip_new_err)?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .report()
                .change_context(UnzipError::by_index_err)?;
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let final_path: PathBuf = destination.as_ref().join(outpath);

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    debug!("File {} comment: {}", i, comment);
                }
            }

            if (*file.name()).ends_with('/') {
                if let Some(ref fun) = should_skip_this_file {
                    if fun(OsStr::new(file.name())) {
                        trace!("Skip folder {:?} Folder not extracted", file.name());
                        continue;
                    }
                };

                trace!("File {} extracted to \"{}\"", i, final_path.display());
                fs::create_dir_all(&final_path)
                    .report()
                    .change_context(UnzipError::create_dir_all)?;
            } else {
                let file_name = match final_path.file_name() {
                    Some(it) => it,
                    None => return Err(Report::new(UnzipError::OutpathFileName)),
                };

                if let Some(ref fun) = should_skip_this_file {
                    if fun(file_name) {
                        trace!("Skip File {:?} File not extracted", file_name);
                        continue;
                    }
                };

                trace!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    final_path.display(),
                    file.size()
                );

                if let Some(p) = final_path.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p)
                            .report()
                            .change_context(UnzipError::create_dir_all)?;
                    }
                }

                let mut outfile = fs::File::create(&final_path)
                    .report()
                    .change_context(UnzipError::file_create)?;
                io::copy(&mut file, &mut outfile)
                    .report()
                    .change_context(UnzipError::io_copy)?;
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&final_path, fs::Permissions::from_mode(mode))
                        .report()
                        .change_context(UnzipError::set_permissions)?;
                }
            }
        }

        Ok(true)
    }

    #[cfg(test)]
    pub fn mock_unzip<F>(
        _file_to_unzip: impl AsRef<Path>,
        _destination: impl AsRef<Path>,
        _should_skip_this_file: Option<F>,
    ) -> Result<bool, UnzipError>
    where
        F: Fn(&OsStr) -> bool,
    {
        Ok(true)
    }

    use std::error::Error;

    use std::fmt::Display;

    #[derive(Debug)]
    pub enum UnzipError {
        OutpathFileName,
        other,
        zip_new_err,
        by_index_err,
        create_dir_all,
        file_create,
        io_copy,
        set_permissions,
    }

    impl Display for UnzipError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Local error")
        }
    }

    impl Error for UnzipError {}
}
