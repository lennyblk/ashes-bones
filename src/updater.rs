use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::network::VERSION;

const REPO_OWNER: &str = "lennyblk";
const REPO_NAME: &str = "ashes-bones";

pub enum UpdateStatus {
    Downloading(String),
    Installed(String),
}

pub fn install_dir() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    dir.join("assets").is_dir().then_some(dir)
}

pub fn start() -> Option<Receiver<UpdateStatus>> {
    let dir = install_dir()?;
    let (sender, status) = mpsc::channel();
    thread::spawn(move || {
        let _ = update(&dir, &sender);
    });
    Some(status)
}

fn update(dir: &Path, sender: &Sender<UpdateStatus>) -> Result<(), Box<dyn std::error::Error>> {
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .build()?
        .fetch()?;
    let Some(latest) = releases.latest() else {
        return Ok(());
    };
    if !self_update::version::bump_is_greater(VERSION, latest.version())? {
        return Ok(()); // déjà à jour
    }
    let asset = latest
        .asset_for(self_update::get_target(), None)
        .ok_or("no build for this OS in the release")?;
    let _ = sender.send(UpdateStatus::Downloading(latest.version().to_string()));

    let staging = dir.join(".update");
    let extracted = staging.join("extracted");
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&extracted)?;
    fs::create_dir_all(staging.join("stash"))?;
    let zip_path = staging.join(asset.name());
    self_update::Download::from_url(asset.download_url())
        .request_header(
            self_update::http::header::ACCEPT,
            "application/octet-stream",
        )
        .download_to(fs::File::create(&zip_path)?)?;
    self_update::Extract::from_source(&zip_path)
        .archive(self_update::ArchiveKind::Zip)
        .extract_into(&extracted)?;

    self_update::MoveAll::from_temp(staging.join("stash"))
        .add(extracted.join("assets"), dir.join("assets"))
        .commit()?;
    let new_exe = extracted.join(format!("ashes-bones{}", std::env::consts::EXE_SUFFIX));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&new_exe, fs::Permissions::from_mode(0o755))?;
    }
    self_replace::self_replace(&new_exe)?;
    let _ = fs::remove_dir_all(&staging);
    let _ = sender.send(UpdateStatus::Installed(latest.version().to_string()));
    Ok(())
}
