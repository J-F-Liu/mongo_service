use mongodb::Client;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    let mut app = tide::new();
    app.at("/collections").nest({
        let mut app = tide::with_state(mongo_service::State {
            db: client.database("database"),
        });
        app.at("/").get(mongo_service::routes::list_collections);
        app
    });
    app.at("/data").nest(mongo_service::serve(client.database("database")));
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
