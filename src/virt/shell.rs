use serde::{Deserialize, Serialize};
use std::process::Command;

use super::{CreateVirtConfig, SnapShotConfig, SystemType};

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

pub fn set_current_snapshot(configure: SnapShotConfig) -> Result<String, std::io::Error> {
    let mut cmd = Command::new("virsh");
    cmd.arg("snapshot-revert")
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


pub fn create_virt(configure: CreateVirtConfig) -> Result<String, std::io::Error> {
    let mut create_qcow2_cmd = Command::new("qemu-img");
    create_qcow2_cmd
        .arg("create")
        .arg("-f")
        .arg("qcow2")
        .arg("disk.qcow2")
        .arg(configure.disk_size)
        .current_dir("/data_disk/create_test");
    let status = create_qcow2_cmd.status()?;
    match status.code() {
        Some(code) => {
            if code == 0 {
                ()
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    String::from_utf8(create_qcow2_cmd.output()?.stderr)
                        .unwrap()
                        .trim(),
                ));
            }
        }
        None => (),
    };
    let mut cmd = Command::new("virt-install");
    cmd.arg("--name")
        .arg(configure.virt_name)
        .arg("--memory")
        .arg(configure.memory)
        .arg("--vcpu")
        .arg(configure.vcpu)
        .arg("--disk")
        .arg("/data_disk/create_test/disk.qcow2")
        .arg("--cdrom")
        .arg(format!("/data_disk/create_test/cdrom.iso"))
        .arg("--graphics")
        .arg(format!(
            "vnc,port={},password={},listen=0.0.0.0",
            "5903", "abc123"
        ))
        .arg("--network")
        .arg("default");
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
