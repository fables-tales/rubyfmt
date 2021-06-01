use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(not(debug_assertions))]
use std::process::Stdio;
use std::time::Duration;

const ONE_DAY: Duration = Duration::from_secs(60 * 60 * 24);

#[derive(Debug, Clone, serde::Deserialize)]
struct GithubTag {
    name: String,
}

/// Begins the process of checking if a new version is available
///
/// This will check the timestamp of a file containing the latest version of rubyfmt. If the file
/// hasn't been updated in at least a day, the file will be touched and a process will be spawned
/// to write the most recent version.
///
/// Panics on any error in debug builds. Since this process is strictly optional, any errors are
/// ignored in release builds.
pub(crate) fn begin_checking_for_updates() {
    let result = try_begin_checking_for_updates();
    if cfg!(debug_assertions) {
        result.unwrap();
    }
}

fn try_begin_checking_for_updates() -> io::Result<()> {
    let path = path_to_latest_version_file()?;
    let time_since_modified = time_since_last_update(&path)?;
    if time_since_modified >= ONE_DAY {
        // Eagerly update the timestamp. If something goes wrong, we don't
        // want to be spamming GitHub's API.
        filetime::set_file_mtime(&path, filetime::FileTime::now())?;
        let mut command = Command::new(env::current_exe()?);
        command.arg("--internal-fetch-latest-version");
        #[cfg(not(debug_assertions))]
        {
            command.stdout(Stdio::null());
            command.stderr(Stdio::null());
        }
        command.spawn()?;
    }
    Ok(())
}

/// Reports if updates are available
///
/// This will check a file which contains the latest available version of rubyfmt. If it is greater
/// than the current version, we will print to stderr that an update is available. Since this
/// process is strictly optional, any errors will be silently ignored.
///
/// For most code bases, rubyfmt will finish running well before the update check is completed.
/// This means that this function will most likely only print a warning on the second run after an
/// update is available.
pub(crate) fn report_if_update_available() {
    let result = try_report_if_update_available();
    if cfg!(debug_assertions) {
        result.unwrap();
    }
}

fn try_report_if_update_available() -> io::Result<()> {
    let latest_version = latest_available_rubyfmt_version()?;
    if latest_version > installed_rubyfmt_version() {
        eprintln!("A new version of rubyfmt is available at https://github.com/penelopezone/rubyfmt/releases/tag/v{}", latest_version);
    }
    Ok(())
}

fn installed_rubyfmt_version() -> semver::Version {
    semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("$CARGO_PKG_VERSION should always be a valid semver version")
}

fn latest_available_rubyfmt_version() -> io::Result<semver::Version> {
    let latest_version_str = fs::read_to_string(path_to_latest_version_file()?)?;
    semver::Version::parse(&latest_version_str)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Determine the most recent version of Rubyfmt, and write it to a file
///
/// Because GitHub paginates its responses to this endpoint, this assumes that the latest version
/// is in the first 30 tags given to us (which are sorted in lexographically descending order).
/// Tags which are a version release are assumed to always be `v` followed by a semver version (for
/// example, v1.0.0)
pub(crate) fn fetch_latest_version() -> Result<(), Box<dyn Error>> {
    let max_version = ureq::get("https://api.github.com/repos/penelopezone/rubyfmt/tags")
        .set("Accept", "application/vnd.github.v3+json")
        .set("User-Agent", "rubyfmt update checker")
        .call()?
        .into_json::<Vec<GithubTag>>()?
        .into_iter()
        .filter(|tag| tag.name.starts_with('v'))
        .filter_map(|tag| semver::Version::parse(&tag.name[1..]).ok())
        .max()
        .ok_or("no valid versions found")?;
    fs::write(path_to_latest_version_file()?, max_version.to_string())?;
    Ok(())
}

fn path_to_latest_version_file() -> io::Result<PathBuf> {
    let path = dirs::data_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No data directory configured"))?
        .join(".rubyfmt-latest-version");
    Ok(path)
}

fn time_since_last_update(path: &Path) -> io::Result<Duration> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            fs::write(&path, env!("CARGO_PKG_VERSION"))?;
            return Ok(Duration::default());
        }
        Err(e) => return Err(e),
    };
    let time_since_modified = meta.modified()?.elapsed();
    // If mtime is in the future, we'll get an error from `elapsed`.
    // Just return a zero duration in this case
    Ok(time_since_modified.unwrap_or_default())
}
