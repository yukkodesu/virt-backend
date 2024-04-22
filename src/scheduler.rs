use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

pub struct SchedConnect {
    pub tx: Sender<SchedCommand>,
    pub rx: Mutex<Receiver<SchedResult>>,
}

type SchedResult = Result<String, JobSchedulerError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchedCommand {
    Add(SchedTaskConfig),
    Delete(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchedTaskConfig {
    pub dom_name: String,
    pub cron: String,
}

impl SchedConnect {
    pub async fn new() -> Self {
        let (sched_tx, mut sched_rx): (Sender<SchedCommand>, Receiver<SchedCommand>) =
            mpsc::channel(2);
        let (result_tx, mut result_rx): (Sender<SchedResult>, Receiver<SchedResult>) =
            mpsc::channel(2);
        tokio::spawn(async move {
            let scheduler = JobScheduler::new().await.unwrap();
            scheduler.start().await.unwrap();
            while let Some(recv) = sched_rx.recv().await {
                match recv {
                    SchedCommand::Add(config) => {
                        let res = scheduler
                            .add(
                                Job::new(config.cron.as_str(), |_uuid, _l| {
                                    println!("sched run!");
                                })
                                .unwrap(),
                            )
                            .await;
                        match res {
                            Ok(uuid) => result_tx.send(Ok(uuid.to_string())).await.unwrap(),
                            Err(e) => result_tx.send(Err(e)).await.unwrap(),
                        }
                    }
                    SchedCommand::Delete(str) => {
                        let res = scheduler.remove(&Uuid::parse_str(&str).unwrap()).await;
                        match res {
                            Ok(_) => result_tx
                                .send(Ok("Delete Sucessfully".to_string()))
                                .await
                                .unwrap(),
                            Err(e) => result_tx.send(Err(e)).await.unwrap(),
                        }
                    }
                }
            }
        });
        SchedConnect {
            tx: sched_tx,
            rx: Mutex::new(result_rx),
        }
    }
}
