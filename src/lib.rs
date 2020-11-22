use mongodb::Database;

pub mod routes;
pub mod util;
use routes::*;

#[derive(Clone)]
pub struct State {
    pub db: Database,
}

/// Create a `tide::Server` for data storage service.
pub fn serve(database: Database) -> tide::Server<State> {
    let mut app = tide::with_state(State { db: database });
    app.at("/:collection").get(find_objects).post(insert_object);
    app.at("/:collection/:id")
        .get(find_object)
        .put(update_object)
        .patch(modify_object)
        .delete(delete_object);
    app
}
