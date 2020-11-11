use super::{util, State};
use chrono::Utc;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    options::FindOptions,
};
use std::convert::TryFrom;
use tide::{prelude::*, Body, Error, Request, Response, StatusCode};

#[derive(Debug, Deserialize)]
pub struct Query {
    r#where: Option<String>,
    order: Option<String>,
    keys: Option<String>,
    skip: Option<i64>,
    limit: Option<i64>,
    count: Option<i8>,
}

pub async fn find_record(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let id = req.param("id")?;
    let filter = doc! {"_id": ObjectId::with_string(id)?};
    let collection_name = req.param("collection")?;
    let collection = req.state().db.collection(collection_name);
    match collection.find_one(filter, None).await {
        Ok(result) => {
            if let Some(mut doc) = result {
                util::make_json_friendly(&mut doc)?;
                Ok(Body::from_json(&doc)?)
            } else {
                Err(Error::from_str(
                    StatusCode::NotFound,
                    format!("{} not found", id),
                ))
            }
        }
        Err(err) => Err(Error::new(StatusCode::ServiceUnavailable, err)),
    }
}

pub async fn find_records(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let query: Query = req.query()?;
    let filter = if let Some(string) = query.r#where {
        let value: serde_json::Value = serde_json::from_str(&string)?;
        match Bson::try_from(value)? {
            Bson::Document(document) => Some(document),
            _ => None,
        }
    } else {
        None
    };
    let order = query.order.map(|string| {
        let mut sort = Document::new();
        for field in string.split(',') {
            if let Some(field) = field.strip_prefix('-') {
                sort.insert(field, -1);
            } else {
                sort.insert(field, 1);
            }
        }
        sort
    });
    let projection = query.keys.map(|string| {
        let mut fields = Document::new();
        for field in string.split(',') {
            if let Some(field) = field.strip_prefix('-') {
                fields.insert(field, -1);
            } else {
                fields.insert(field, 1);
            }
        }
        fields
    });
    let find_options = FindOptions::builder()
        .skip(query.skip.unwrap_or(0))
        .limit(query.limit.unwrap_or(200))
        .sort(order)
        .projection(projection)
        .build();

    let collection_name = req.param("collection")?;
    let collection = req.state().db.collection(collection_name);
    let count = if query.count == Some(1) {
        Some(
            collection
                .count_documents(filter.clone(), None)
                .await
                .unwrap(),
        )
    } else {
        None
    };
    let mut cursor = collection.find(filter, find_options).await.unwrap();
    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(mut doc) => {
                util::make_json_friendly(&mut doc)?;
                results.push(doc);
            }
            Err(_) => {}
        }
    }
    if let Some(count) = count {
        Ok(json!({ "results": results, "count": count }))
    } else {
        Ok(json!({ "results": results }))
    }
}

pub async fn insert_record(mut req: Request<State>) -> tide::Result<impl Into<Response>> {
    let value: serde_json::Value = req
        .body_json()
        .await
        .map_err(|_| Error::from_str(StatusCode::BadRequest, "Invalid json"))?;
    match Bson::try_from(value)? {
        Bson::Document(mut document) => {
            let now = Utc::now();
            document.insert("createdAt", now);
            let collection_name = req.param("collection")?;
            let collection = req.state().db.collection(collection_name);
            match collection.insert_one(document, None).await {
                Ok(result) => {
                    let response = Response::builder(StatusCode::Created).body(json!({
                        "objectId": result.inserted_id.as_object_id().unwrap().to_hex(),
                        "createdAt": util::format_datetime(&now)
                    }));
                    Ok(response)
                }
                Err(err) => Err(Error::new(StatusCode::InternalServerError, err)),
            }
        }
        _ => Err(Error::from_str(StatusCode::BadRequest, "Expect document")),
    }
}

pub async fn update_record(mut req: Request<State>) -> tide::Result<impl Into<Response>> {
    let id = req.param("id")?;
    let filter = doc! {"_id": ObjectId::with_string(id)?};
    let value: serde_json::Value = req
        .body_json()
        .await
        .map_err(|_| Error::from_str(StatusCode::BadRequest, "Invalid json"))?;
    match Bson::try_from(value)? {
        Bson::Document(document) => {
            let collection_name = req.param("collection")?;
            let collection = req.state().db.collection(collection_name);
            let mut update = Document::new();
            update.insert("$set", document);
            update.insert("$currentDate", doc! { "updatedAt": true });
            match collection.update_one(filter, update, None).await {
                Ok(result) => Ok(json!(result)),
                Err(err) => Err(Error::new(StatusCode::InternalServerError, err)),
            }
        }
        _ => Err(Error::from_str(StatusCode::BadRequest, "Expect document")),
    }
}

pub async fn delete_record(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let id = req.param("id")?;
    let filter = doc! {"_id": ObjectId::with_string(id)?};
    let collection_name = req.param("collection")?;
    let collection = req.state().db.collection(collection_name);
    match collection.delete_one(filter, None).await {
        Ok(result) => Ok(json!(result)),
        Err(err) => Err(Error::new(StatusCode::InternalServerError, err)),
    }
}
