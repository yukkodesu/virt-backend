use rocket::{http::Status, response::content, serde::json::Json, State};

use crate::{
    db::entity::{prelude::*, *},
    middleware::authenticate::JWT,
    scheduler::{SchedCommand, SchedConnect, SchedTaskConfig},
    virt::{shell, SnapShotConfig, VirtCommand, VirtCommandType, VirtConnect},
};

use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};

#[post("/list", format = "application/json", data = "<dom_names>")]
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

#[post("/list-tree", format = "application/json", data = "<dom_name>")]
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

#[post("/create", format = "application/json", data = "<configure>")]
pub fn create_snapshot(
    _jwt: JWT,
    configure: Json<SnapShotConfig>,
) -> (Status, content::RawJson<String>) {
    match shell::create_snapshot(configure.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/delete", format = "application/json", data = "<configure>")]
pub fn delete_snapshot(
    _jwt: JWT,
    configure: Json<SnapShotConfig>,
) -> (Status, content::RawJson<String>) {
    match shell::delete_snapshot(configure.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/edit", format = "application/json", data = "<configure>")]
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

#[post("/clone-as-vm", format = "application/json", data = "<configure>")]
pub fn clone_snapshot_as_vm(
    _jwt: JWT,
    configure: Json<SnapShotConfig>,
) -> (Status, content::RawJson<String>) {
    match shell::clone_snapshot_as_vm(configure.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/set-current", format = "application/json", data = "<configure>")]
pub fn set_current_snapshot(
    _jwt: JWT,
    configure: Json<SnapShotConfig>,
) -> (Status, content::RawJson<String>) {
    match shell::set_current_snapshot(configure.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/sched-task/add", data = "<config>")]
pub async fn add_sched_task(
    _jwt: JWT,
    db: &State<DatabaseConnection>,
    sched: &State<SchedConnect>,
    config: Json<SchedTaskConfig>,
) -> (Status, String) {
    let config = config.0;
    let sched = sched as &SchedConnect;
    let db = db as &DatabaseConnection;
    sched
        .tx
        .send(SchedCommand::Add(config.clone()))
        .await
        .unwrap();
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

#[post("/sched-task/delete", data = "<uuid>")]
pub async fn delete_sched_task(
    _jwt: JWT,
    db: &State<DatabaseConnection>,
    sched: &State<SchedConnect>,
    uuid: Json<String>,
) -> (Status, String) {
    let uuid = uuid.0;
    let sched = sched as &SchedConnect;
    let db = db as &DatabaseConnection;
    sched
        .tx
        .send(SchedCommand::Delete(uuid.clone()))
        .await
        .unwrap();
    let _ = sched.rx.lock().await.recv().await.unwrap();
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
