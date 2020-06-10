use crate::config::{Config, EXT_CMD_CODE};
use crate::crc::crc32;
use crate::firmware::Firmware;
use crate::header::Header;
use crate::protocol::generate_script;
use crate::script::Script;
use crate::xcmd::ExtCmdProtocol;
use crate::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use git2::{Repository, Status, IndexEntry, IndexAddOption, Commit, ObjectType};

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
    let protocol = ExtCmdProtocol::new(EXT_CMD_CODE);
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

pub fn write_crc(fw: &mut Firmware) {
    let crc = crc32(&fw.data[4..fw.image_length()]);
    fw.write_u32(0, crc);
}

pub fn merge_all(config: &mut Config, config_dir: &Path) -> Result<Vec<Firmware>, Error> {
    let mut ret = Vec::new();
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
        || status.is_wt_renamed() || status.is_ignored() || status.is_conflicted()
}

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

fn format_release_message(config: &Config) -> String {
    todo!()
}

pub fn release(config: &mut Config, config_dir: &Path) -> Result<(), Error> {
    let repo_path = config.get_repo_path(config_dir)?;
    let output_dir = repo_path.join("release");

    // start by checking git repository
    let repo = Repository::open(&repo_path).map_err(Error::GitError)?;
    if !repo.is_worktree() {
        return Err(Error::GitRepoIsNotAWorktree);
    }
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
    let mut new_config = config.clone();

    // merge firmwares
    let fws = merge_all(&mut new_config, config_dir)?;
    let merged_files = write_fws(&config, &fws, &output_dir)?;
    output_files.extend(merged_files);

    // create a branch
    todo!();

    // create a commit
    let mut index = repo.index().map_err(Error::GitError)?;
    index.add_all(output_files, IndexAddOption::DEFAULT, None);
    let oid = index.write_tree().map_err(Error::GitError)?;

    let signature = repo.signature().map_err(Error::GitError)?;
    let tree = repo.find_tree(oid).map_err(Error::GitError)?;
    let parent = find_last_commit(&repo).map_err(Error::GitError)?;

    let message = format_release_message(&config);
    repo.commit(Some("HEAD"), &signature, &signature,
                &message, &tree, &[&parent]).map_err(Error::GitError)?;

    // create tag
    todo!();

    // push tag and branch
    todo!();

    Ok(())
}

pub fn load_app(config: &mut Config, idx: usize, config_dir: &Path) -> Result<Firmware, Error> {
    let path = Config::normalize_path(&config.images[idx].app_path, config_dir)?;
    let mut fw = Firmware::load_from_file(
        &path,
        &config.images[idx].hex_file_format,
        &config.images[idx].device_config,
        &config.images[idx].app_address,
    )?;
    write_crc(&mut fw);
    let mut header = Header::new(&mut fw, config.images[idx].header_offset);
    if config.product_id != 0 && config.product_id != header.product_id() {
        return Err(Error::InvalidConfig(format!(
            "Product ID in firmware and config does not match: {} vs. {}",
            config.product_id,
            header.product_id()
        )));
    } else if config.product_id == 0 {
        config.product_id = header.product_id();
    }
    if config.major_version != 0xFF && config.major_version != header.major_version() {
        return Err(Error::InvalidConfig(format!(
            "Major version in firmware and config does not match: {} vs. {}",
            config.major_version,
            header.major_version()
        )));
    } else if config.major_version == 0xFF {
        config.major_version = header.major_version();
    } else if header.major_version() == 0xFF {
        header.set_major_version(config.major_version);
    }

    let minor = config.images[idx].version.minor;
    if minor != 0xFF && minor != header.minor_version() {
        return Err(Error::InvalidConfig(format!(
            "Minor version in firmware and config does not match: {} vs. {}",
            minor,
            header.minor_version()
        )));
    } else if minor == 0xFF {
        config.images[idx].version.minor = header.minor_version();
    } else if header.minor_version() == 0xFF {
        header.set_minor_version(minor);
    }

    let build = config.images[idx].version.build;
    if build != 0xFF && build != header.build_version() {
        return Err(Error::InvalidConfig(format!(
            "Build version in firmware and config does not match: {} vs. {}",
            build,
            header.build_version()
        )));
    } else if build == 0xFF {
        config.images[idx].version.build = header.build_version();
    } else if header.build_version() == 0xFF {
        header.set_build_version(build);
    }

    let fw_id = config.images[idx].fw_id;
    if fw_id != 0 && fw_id != header.fw_id() {
        return Err(Error::InvalidConfig(format!(
            "Firmware ID in firmware and config does not match: {} vs. {}",
            build,
            header.fw_id()
        )));
    } else if fw_id == 0 {
        config.images[idx].fw_id = header.fw_id();
    } else if header.fw_id() == 0 {
        header.set_fw_id(fw_id);
    }

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
