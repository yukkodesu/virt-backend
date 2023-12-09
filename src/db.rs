pub mod entity;
use entity::{prelude::*, *};
use sea_orm::{Database, DbErr, EntityTrait, DatabaseConnection};

pub async fn init(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    let user = User::find().all(&db).await?;
    println!("{:?}", user);
    Ok(db)
}
