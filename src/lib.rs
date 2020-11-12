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
    app.at("/:collection").get(find_objects).post(insert_object);
    app.at("/:collection/:id")
        .get(find_object)
        .put(update_object)
        .patch(modify_object)
        .delete(delete_object);
    Ok(app)
}
