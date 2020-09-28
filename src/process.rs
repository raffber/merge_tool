use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use git2::{BranchType, Commit, ObjectType, Repository, Status};

use crate::config::{Config, DDP_CMD_CODE};
use crate::config::default;
use crate::crc::crc32;
use crate::Error;
use crate::firmware::Firmware;
use crate::header::Header;
use crate::protocol::generate_script;
use crate::script::Script;
use crate::ddp::DdpProtocol;

pub fn merge_firmware(
    config: &mut Config,
    idx: usize,
    config_dir: &Path,
) -> Result<Firmware, Error> {
    let app = load_app(config, idx, config_dir)?;
    let btl = load_btl(config, idx, config_dir)?;
    Firmware::merge(btl, app)
}

fn generate_script_filename(config: &Config) -> String {
    let mut parts = Vec::new();
    parts.push(format!(
        "{}_{}",
        config.product_name.clone(),
        config.major_version
    ));
    for fw_config in &config.images {
        parts.push(format!(
            "_{}.{}",
            fw_config.version.minor, fw_config.version.build
        ));
    }
    parts.push(".gctbtl".to_string());
    parts.join("")
}

pub fn create_script(
    config: &mut Config,
    config_dir: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Error> {
    config.transform_to_byte_addrs();
    let protocol = DdpProtocol::new(DDP_CMD_CODE);
    let mut fws = Vec::new();
    for idx in 0..config.images.len() {
        fws.push(load_app(config, idx, config_dir)?);
    }
    let cmds = generate_script(&protocol, &fws, config);
    let filename = generate_script_filename(config);

    let script = Script::new(cmds);
    let path = output_dir.join(&filename);
    let mut file = File::create(&path).map_err(Error::Io)?;
    file.write_all(script.serialize().as_bytes())
        .map_err(Error::Io)?;
    Ok(path)
}

pub fn merge_all(config: &mut Config, config_dir: &Path) -> Result<Vec<Firmware>, Error> {
    let mut ret = Vec::new();
    config.transform_to_byte_addrs();
    for idx in 0..config.images.len() {
        let fw = merge_firmware(config, idx, config_dir)?;
        ret.push(fw);
    }
    Ok(ret)
}

pub fn write_fws(config: &Config, fws: &[Firmware], target_folder: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut ret = Vec::new();
    for (fw, fw_config) in fws.iter().zip(config.images.iter()) {
        let file_name = format!(
            "{}.{}",
            fw_config.designator(),
            fw_config.hex_file_format.file_extension()
        );
        let fpath = target_folder.join(file_name);
        fw.write_to_file(&fpath, &fw_config.hex_file_format)?;
        ret.push(fpath);
    }
    Ok(ret)
}


pub fn is_git_repo_dirty(status: Status) -> bool {
    status.is_index_modified() || status.is_index_deleted() || status.is_index_renamed()
        || status.is_index_typechange() || status.is_wt_deleted() || status.is_wt_typechange()
        || status.is_wt_renamed() || status.is_conflicted() || status.is_wt_new()
        || status.is_wt_modified()
}

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

fn format_release_message(config: &Config) -> String {
    let mut parts = vec![format!("Firmware release for `{}`", config.product_name)];
    for img in &config.images {
        parts.push(format!("F{}={}.{}.{}", img.fw_id, config.major_version, img.version.minor, img.version.build));
    }
    parts.join(" ")
}

fn format_branch_name(config: &Config) -> String {
    let mut parts = vec![format!("release/{}_{}_", config.product_name, config.major_version)];
    for img in &config.images {
        parts.push(format!("{}.{}", img.version.minor, img.version.build));
    }
    parts.join("")
}

pub fn release(config: &mut Config, config_dir: &Path) -> Result<(), Error> {
    config.transform_to_byte_addrs();
    let repo_path = config.get_repo_path(config_dir)?;
    let output_dir = repo_path.join("release");
    fs::create_dir_all(&output_dir).map_err(Error::Io)?;

    // start by checking git repository
    let repo = Repository::open(&repo_path).map_err(Error::GitError)?;
    let statuses = repo.statuses(None).map_err(Error::GitError)?;
    for status in statuses.iter() {
        let status = status.status();
        if is_git_repo_dirty(status) {
            return Err(Error::GitRepoHasUncommitedChanges);
        }
    }
    if repo.head_detached().map_err(Error::GitError)? {
        return Err(Error::GitRepoInDetachedHead);
    }

    let mut output_files = Vec::new();
    // create script
    let mut new_config = config.clone();
    let script_path = create_script(&mut new_config, config_dir, &output_dir)?;
    output_files.push(script_path);

    // merge firmwares
    let fws = merge_all(config, config_dir)?;
    let merged_files = write_fws(config, &fws, &output_dir)?;
    output_files.extend(merged_files);

    // retrieve some basic information about the current state
    let parent = find_last_commit(&repo).map_err(Error::GitError)?;
    let signature = repo.signature().map_err(Error::GitError)?;

    // create a branch
    let branch_name = format_branch_name(config);
    if let Ok(_) = repo.find_branch(&branch_name, BranchType::Local) {
        return Err(Error::GitBranchAlreadyExists(branch_name));
    }
    let branch = repo.branch(&branch_name, &parent, false).map_err(Error::GitError)?;
    repo.set_head(branch.get().name().unwrap()).map_err(Error::GitError)?;

    // create a commit
    let mut index = repo.index().map_err(Error::GitError)?;
    for file in &output_files {
        // unwrap is fine because we created the files relative to the
        // current directory
        let file = pathdiff::diff_paths(file, &repo_path).unwrap();
        index.add_path(&file).map_err(Error::GitError)?;
    }
    let oid = index.write_tree().map_err(Error::GitError)?;
    let tree = repo.find_tree(oid).map_err(Error::GitError)?;

    let message = format_release_message(&config);
    repo.commit(Some("HEAD"), &signature, &signature,
                &message, &tree, &[&parent]).map_err(Error::GitError)?;

    // create tag
    let obj = repo.head()
        .and_then(|x| x.resolve())
        .and_then(|x| x.peel(ObjectType::Commit))
        .map_err(Error::GitError)?;
    repo.tag(&branch_name, &obj, &signature, &message, false)
        .map_err(Error::GitError)?;

    // TODO: push is difficult since we don't know the authentication method
    // push this to user?

    // push tag and branch
    // let mut remote = repo.find_remote("origin").map_err(Error::GitRepoHasNoOrigin)?;
    // let branch_ref = format!("refs/heads/{}:refs/heads/{}", &branch_name, &branch_name);
    // let tag_ref = format!("refs/tags/{}:refs/tags/{}", &branch_name, &branch_name);
    // remote.connect(Direction::Push).map_err(Error::GitCannotPush)?;
    // remote.push(&[&branch_ref, &tag_ref], None).map_err(Error::GitCannotPush)?;

    Ok(())
}

fn configure_header(mut fw: Firmware, config: &mut Config, idx: usize) -> Result<Firmware, Error> {
    let image_length = fw.image_length();
    let mut header = Header::new(&mut fw, config.images[idx].header_offset)?;
    if config.product_id != default::product_id() && config.product_id != header.product_id() {
        return Err(Error::InvalidConfig(format!(
            "Product ID in firmware and config does not match: {} vs. {}",
            config.product_id,
            header.product_id()
        )));
    } else if config.product_id == default::product_id() {
        config.product_id = header.product_id();
    }
    if config.major_version != default::major_version() && config.major_version != header.major_version() {
        return Err(Error::InvalidConfig(format!(
            "Major version in firmware and config does not match: {} vs. {}",
            config.major_version,
            header.major_version()
        )));
    } else if config.major_version == default::major_version() {
        config.major_version = header.major_version();
    } else if header.major_version() == default::major_version() {
        header.set_major_version(config.major_version);
    }

    let minor = config.images[idx].version.minor;
    if minor != default::minor_version() && minor != header.minor_version() {
        return Err(Error::InvalidConfig(format!(
            "Minor version in firmware and config does not match: {} vs. {}",
            minor,
            header.minor_version()
        )));
    } else if minor == default::minor_version() {
        config.images[idx].version.minor = header.minor_version();
    } else if header.minor_version() == default::minor_version() {
        header.set_minor_version(minor);
    }

    let build = config.images[idx].version.build;
    if build != default::build_version() && build != header.build_version() {
        return Err(Error::InvalidConfig(format!(
            "Build version in firmware and config does not match: {} vs. {}",
            build,
            header.build_version()
        )));
    } else if build == default::build_version() {
        config.images[idx].version.build = header.build_version();
    } else if header.build_version() == default::build_version() {
        header.set_build_version(build);
    }

    let fw_id = config.images[idx].fw_id;
    if fw_id != default::fw_id() && fw_id != header.fw_id() {
        return Err(Error::InvalidConfig(format!(
            "Firmware ID in firmware and config does not match: {} vs. {}",
            build,
            header.fw_id()
        )));
    } else if fw_id == default::fw_id() {
        config.images[idx].fw_id = header.fw_id();
    } else if header.fw_id() == default::fw_id() {
        header.set_fw_id(fw_id);
    }
    header.set_length(image_length as u32);
    Ok(fw)
}

pub fn load_app(config: &mut Config, idx: usize, config_dir: &Path) -> Result<Firmware, Error> {
    let path = Config::normalize_path(&config.images[idx].app_path, config_dir)?;
    let fw = Firmware::load_from_file(
        &path,
        &config.images[idx].hex_file_format,
        &config.images[idx].device_config,
        &config.images[idx].app_address,
    )?;

    let mut fw = configure_header(fw, config, idx)?;
    let crc = crc32(&fw.data[4..fw.image_length()]);
    fw.write_u32(0, crc);

    Ok(fw)
}

pub fn load_btl(config: &Config, idx: usize, config_dir: &Path) -> Result<Firmware, Error> {
    let path = Config::normalize_path(&config.images[idx].btl_path, config_dir)?;
    let fw_config = &config.images[idx];
    Firmware::load_from_file(
        &path,
        &fw_config.hex_file_format,
        &fw_config.device_config,
        &fw_config.btl_address,
    )
}
