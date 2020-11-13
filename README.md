# mongo_service
[![Crates.io](https://img.shields.io/crates/v/mongo_service.svg)](https://crates.io/crates/mongo_service)
[![Docs](https://docs.rs/mongo_service/badge.svg)](https://docs.rs/mongo_service)

General CRUD RESTful APIs for MongoDB.

### Routes

- /:collection
    - GET - Get object list
    - POST - Create new object
- /:collection/:id
    - GET - Get object
    - PUT - Update object with new field values
    - PATCH - Update object with MongoDB update operators
    - DELETE - Delete object

### Usage

```rust
use mongodb::Client;

let client = Client::with_uri_str("mongodb://localhost:27017").await?;
let mut app = tide::new();
app.at("/api").nest(mongo_service::serve(client.database("database")));
app.listen("127.0.0.1:8080").await?;
```