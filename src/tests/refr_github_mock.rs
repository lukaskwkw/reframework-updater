use std::collections::HashMap;

use lazy_static::lazy_static;
use self_update::update::ReleaseAsset;

use self_update::update::Release;

use crate::reframework_github::refr_github::AssetsReport;
use crate::reframework_github::refr_github::MockREFRGithub;
use crate::DynResult;

use mockall::mock;

use crate::reframework_github::refr_github::REFRGithub;

use crate::reframework_github::refr_github::ManageGithub;
mock!(
    pub ManageGithubM<T: 'static> {}
    impl ManageGithub<REFRGithub> for ManageGithubM<REFRGithub> {
        pub fn get_reframework_latest_release(&mut self) -> DynResult<()>;
        pub fn generate_assets_report(&mut self) -> DynResult<()>;
        pub fn download_release_asset(&self, release_asset: &ReleaseAsset) -> DynResult<&'static REFRGithub>;
        pub fn fetch_release(&self) -> DynResult<Release>;
        pub fn getRelease(&self) -> Option<&'static Release>;
        pub fn getAssetsReport(&self) -> &'static AssetsReport;
    }
);

static mut RELEASE: Release = Release {
    assets: Vec::new(),
    body: None,
    version: String::new(),
    date: String::new(),
    name: String::new(),
};

lazy_static! {
    static ref ASSETS_REPORT: HashMap<String, Vec<ReleaseAsset>> = {
        let mut m = HashMap::new();
        m.insert(
            "RE7".to_string(),
            [
                ReleaseAsset {
                    name: "RE7.zip".to_string(),
                    download_url: "dupa_url".to_string(),
                },
                ReleaseAsset {
                    name: "RE7_TDBXXX.zip".to_string(),
                    download_url: "dupa_url".to_string(),
                },
            ]
            .to_vec(),
        );

        m.insert(
            "RE2".to_string(),
            [
                ReleaseAsset {
                    name: "RE2.zip".to_string(),
                    download_url: "dupa_url".to_string(),
                },
                ReleaseAsset {
                    name: "RE2_TDBXXX.zip".to_string(),
                    download_url: "dupa_url".to_string(),
                },
            ]
            .to_vec(),
        );

        m.insert(
            "RE3".to_string(),
            [
                ReleaseAsset {
                    name: "RE3.zip".to_string(),
                    download_url: "dupa_url".to_string(),
                },
                ReleaseAsset {
                    name: "RE3_TDBXXX.zip".to_string(),
                    download_url: "dupa_url".to_string(),
                },
            ]
            .to_vec(),
        );

        m.insert(
            "RE8".to_string(),
            [ReleaseAsset {
                name: "RE8.zip".to_string(),
                download_url: "dupa_url".to_string(),
            }]
            .to_vec(),
        );

        m
    };
    static ref REFRGithub_STATIC: REFRGithub = REFRGithub::default();
}

pub fn prepare_refr_github_mock_and_get_constr() -> (
    crate::reframework_github::refr_github::__mock_MockREFRGithub::__new::Context,
    fn(&str, &str) -> Box<dyn ManageGithub<REFRGithub>>,
) {
    let ctx = MockREFRGithub::new_context();
    ctx.expect().returning(|_, _| {
        let mut mock = MockManageGithubM::new();

        unsafe {
            RELEASE.name = "v1.333-07ab146".to_string();
            mock.expect_get_reframework_latest_release()
                .returning(|| Ok(()));
            mock.expect_getRelease().return_const(Some(&RELEASE));
            mock.expect_getAssetsReport().return_const(&*ASSETS_REPORT);
            mock.expect_download_release_asset()
                .returning(|_| Ok(&REFRGithub_STATIC));
        }
        Box::new(mock)
    });
    (ctx, MockREFRGithub::new)
}
