use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SnapShotConfig {
    dom_name: String,
    snapshot_name: String,
    description: Option<String>,
    is_live: Option<String>,
}

pub fn create_snapshot(configure: SnapShotConfig) -> Result<String, std::io::Error> {
    let mut cmd = Command::new("virsh");
    cmd.arg("snapshot-create-as")
        .arg(configure.dom_name)
        .arg("--name")
        .arg(configure.snapshot_name);
    if let Some(des) = configure.description {
        cmd.arg("--description").arg(des);
    }
    if let Some(is_live) = configure.is_live {
        if is_live == "yes" {
            cmd.arg("--live");
        }
    }
    let status = cmd.status()?;
    match status.code() {
        Some(code) => {
            if code == 0 {
                Ok("".to_string())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    String::from_utf8(cmd.output()?.stderr).unwrap().trim(),
                ))
            }
        }
        None => Ok("".to_string()),
    }
}

pub fn delete_snapshot(configure: SnapShotConfig) -> Result<String, std::io::Error> {
    let mut cmd = Command::new("virsh");
    cmd.arg("snapshot-delete")
        .arg(configure.dom_name)
        .arg("--snapshotname")
        .arg(configure.snapshot_name);
    let status = cmd.status()?;
    match status.code() {
        Some(code) => {
            if code == 0 {
                Ok("".to_string())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    String::from_utf8(cmd.output()?.stderr).unwrap().trim(),
                ))
            }
        }
        None => Ok("".to_string()),
    }
}