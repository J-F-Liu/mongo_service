use mongodb::{error::Result, Client, Database};

mod routes;
mod util;
use routes::*;

#[derive(Clone)]
pub struct State {
    db: Database,
}

pub async fn serve(mongo_uri: &str, database: &str) -> Result<tide::Server<State>> {
    let client = Client::with_uri_str(mongo_uri).await?;
    let db = client.database(database);

    let mut app = tide::with_state(State { db });
    app.at("/:collection").get(find_records).post(insert_record);
    app.at("/:collection/:id")
        .get(find_record)
        .put(update_record)
        .delete(delete_record);
    Ok(app)
}
