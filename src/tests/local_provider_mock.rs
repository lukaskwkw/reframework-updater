use std::path::PathBuf;

use crate::utils::local_version::{MockLocalFiles};

pub(crate) fn mock_local_provider_w_defaults(local_provider_mock: &mut Box<MockLocalFiles>) {
    local_provider_mock
        .expect_create_ms_lnk()
        .returning(|_, _, _| Ok(()));
    local_provider_mock
        .expect_create_cache_dir()
        .returning(|| Ok(PathBuf::from("ms/links/folder")));
}
