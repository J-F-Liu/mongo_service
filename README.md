# mongo_service

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
let data_store = mongo_service::serve(
    "mongodb://localhost:27017", "database").await?;
let mut app = tide::new();
app.at("/api").nest(data_store);
app.listen("127.0.0.1:8080").await?;
```