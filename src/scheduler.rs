use sea_orm::prelude::Uuid;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

pub struct Scheduler {
    sched: JobScheduler,
}

impl Scheduler {
    pub async fn new() -> Result<Scheduler, JobSchedulerError> {
        Ok(Scheduler {
            sched: JobScheduler::new().await?,
        })
    }

    pub async fn add(&mut self, job: Job) -> Result<Uuid, JobSchedulerError> {
        let job_id = self.sched.add(job).await?;
        Ok(job_id)
    }
}
