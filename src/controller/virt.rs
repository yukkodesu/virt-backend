use rocket::{
    data::{Data, ToByteUnit},
    http::Status,
    response::content,
    serde::json::Json,
    State,
};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};

use crate::{
    db::entity::{prelude::*, *},
    middleware::authenticate::JWT,
    scheduler::{SchedCommand, SchedConnect},
    virt::{shell, AltDomStateCommand, SnapShotConfig, VirtCommand, VirtCommandType, VirtConnect},
};

#[get("/list-all")]
pub fn list_all(_jwt: JWT, conn: &State<VirtConnect>) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    if let Err(e) = conn.tx.send(VirtCommand::create(VirtCommandType::ListAll)) {
        return (
            Status::InternalServerError,
            content::RawJson(
                String::from("Error sending VirtCommand to LibVirt Thread:") + &e.to_string(),
            ),
        );
    }
    match conn.rx.lock().unwrap().recv() {
        Ok(output) => match output {
            Ok(res) => (Status::Ok, content::RawJson(res)),
            Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
        },
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/list-snapshot", format = "application/json", data = "<dom_names>")]
pub fn list_snapshot(
    _jwt: JWT,
    conn: &State<VirtConnect>,
    dom_names: Json<Vec<String>>,
) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    let dom_names: Vec<String> = dom_names.0.into_iter().collect();
    if let Err(e) = conn.tx.send(VirtCommand::create_with_params(
        VirtCommandType::ListSnapshot,
        dom_names,
    )) {
        return (
            Status::InternalServerError,
            content::RawJson(
                String::from("Error sending VirtCommand to LibVirt Thread:") + &e.to_string(),
            ),
        );
    }
    match conn.rx.lock().unwrap().recv() {
        Ok(output) => match output {
            Ok(res) => (Status::Ok, content::RawJson(res)),
            Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
        },
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post(
    "/list-snapshot-tree",
    format = "application/json",
    data = "<dom_name>"
)]
pub fn list_snapshot_tree(
    _jwt: JWT,
    conn: &State<VirtConnect>,
    dom_name: Json<String>,
) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    if let Err(e) = conn.tx.send(VirtCommand::create_with_params(
        VirtCommandType::ListSnapshotTree,
        vec![dom_name.0],
    )) {
        return (
            Status::InternalServerError,
            content::RawJson(
                String::from("Error sending VirtCommand to LibVirt Thread:") + &e.to_string(),
            ),
        );
    }
    match conn.rx.lock().unwrap().recv() {
        Ok(output) => match output {
            Ok(res) => (Status::Ok, content::RawJson(res)),
            Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
        },
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/create-snapshot", format = "application/json", data = "<configure>")]
pub fn create_snapshot(
    _jwt: JWT,
    configure: Json<SnapShotConfig>,
) -> (Status, content::RawJson<String>) {
    match shell::create_snapshot(configure.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/delete-snapshot", format = "application/json", data = "<configure>")]
pub fn delete_snapshot(
    _jwt: JWT,
    configure: Json<SnapShotConfig>,
) -> (Status, content::RawJson<String>) {
    match shell::delete_snapshot(configure.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/edit-snapshot", format = "application/json", data = "<configure>")]
pub fn edit_snapshot(
    _jwt: JWT,
    conn: &State<VirtConnect>,
    configure: String,
) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    if let Err(e) = conn.tx.send(VirtCommand::create_with_params(
        VirtCommandType::EditSnapshot,
        vec![configure],
    )) {
        return (
            Status::InternalServerError,
            content::RawJson(
                String::from("Error sending VirtCommand to LibVirt Thread:") + &e.to_string(),
            ),
        );
    }
    match conn.rx.lock().unwrap().recv() {
        Ok(output) => match output {
            Ok(res) => (Status::Ok, content::RawJson(res)),
            Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
        },
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post(
    "/clone-snapshot-as-vm",
    format = "application/json",
    data = "<configure>"
)]
pub fn clone_snapshot_as_vm(
    _jwt: JWT,
    configure: Json<SnapShotConfig>,
) -> (Status, content::RawJson<String>) {
    match shell::clone_snapshot_as_vm(configure.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post(
    "/set-current-snapshot",
    format = "application/json",
    data = "<configure>"
)]
pub fn set_current_snapshot(
    _jwt: JWT,
    configure: Json<SnapShotConfig>,
) -> (Status, content::RawJson<String>) {
    match shell::set_current_snapshot(configure.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/upload-iso", data = "<isofile>")]
pub async fn upload_iso(isofile: Data<'_>) -> (Status, String) {
    match isofile.open(8.gigabytes()).into_file("./test.iso").await {
        Ok(_) => (Status::Ok, "".to_string()),
        Err(e) => (Status::InsufficientStorage, e.to_string()),
    }
}

#[post("/alt-vm-state", data = "<config>")]
pub async fn alt_vm_state(
    _jwt: JWT,
    config: Json<AltDomStateCommand>,
) -> (Status, content::RawJson<String>) {
    match shell::alt_vm_state(config.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/sched-task", data = "<command>")]
pub async fn sched_task(
    _jwt: JWT,
    db: &State<DatabaseConnection>,
    sched: &State<SchedConnect>,
    command: Json<SchedCommand>,
) -> (Status, String) {
    let command = command.0;
    let sched = sched as &SchedConnect;
    let db = db as &DatabaseConnection;
    sched.tx.send(command.clone()).await.unwrap();
    match command {
        SchedCommand::Add(config) => {
            let result = sched.rx.lock().await.recv().await.unwrap();
            match result {
                Ok(uuid) => {
                    if let Err(e) = ScheduleJobs::insert(schedule_jobs::ActiveModel {
                        cron: ActiveValue::set(config.cron),
                        domain: ActiveValue::set(config.dom_name),
                        uuid: ActiveValue::set(uuid),
                        ..Default::default()
                    })
                    .exec(db)
                    .await
                    {
                        return (Status::InternalServerError, e.to_string());
                    }
                    (Status::Ok, "schedule job set successfully!".to_string())
                }
                Err(e) => (Status::InternalServerError, e.to_string()),
            }
        }
        SchedCommand::Delete(uuid) => {
            match ScheduleJobs::find()
                .filter(schedule_jobs::Column::Uuid.eq(uuid.clone()))
                .one(db)
                .await
            {
                Ok(Some(v)) => {
                    v.delete(db).await.unwrap();
                    (
                        Status::Ok,
                        "delete schedule job set successfully!".to_string(),
                    )
                }
                _ => (
                    Status::InternalServerError,
                    format!("can not find job id {}", &uuid).to_string(),
                ),
            }
        }
    }
}
