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

impl Query {
    pub fn create_filter(&self) -> tide::Result<Option<Document>> {
        if let Some(filter) = &self.r#where {
            let value: serde_json::Value = serde_json::from_str(filter)
                .map_err(|_| Error::from_str(StatusCode::BadRequest, "Invalid json in where"))?;
            if let Ok(Bson::Document(document)) = Bson::try_from(value) {
                Ok(Some(document))
            } else {
                Err(Error::from_str(StatusCode::BadRequest, "Expect json object in where"))
            }
        } else {
            Ok(None)
        }
    }

    pub fn create_sort(&self) -> Option<Document> {
        self.order.as_ref().map(|order| {
            let mut sort = Document::new();
            for field in order.split(',') {
                if let Some(field) = field.strip_prefix('-') {
                    sort.insert(field, -1);
                } else {
                    sort.insert(field, 1);
                }
            }
            sort
        })
    }

    pub fn create_projection(&self) -> Option<Document> {
        self.keys.as_ref().map(|keys| {
            let mut fields = Document::new();
            for field in keys.split(',') {
                if let Some(field) = field.strip_prefix('-') {
                    fields.insert(field, 0);
                } else {
                    fields.insert(field, 1);
                }
            }
            fields
        })
    }

    pub fn create_find_options(&self) -> FindOptions {
        FindOptions::builder()
            .skip(self.skip.unwrap_or(0))
            .limit(self.limit.unwrap_or(200))
            .sort(self.create_sort())
            .projection(self.create_projection())
            .build()
    }
}

pub async fn find_object(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let id = req.param("id")?;
    let filter = doc! {"_id": ObjectId::with_string(id)?};
    let collection_name = req.param("collection")?;
    let collection = req.state().db.collection(collection_name);
    let result = collection.find_one(filter, None).await?;
    if let Some(mut doc) = result {
        util::make_json_friendly(&mut doc)?;
        Ok(Body::from_json(&doc)?)
    } else {
        Err(Error::from_str(StatusCode::NotFound, "Object not found"))
    }
}

pub async fn find_objects(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let query: Query = req.query()?;
    let filter = query.create_filter()?;
    let find_options = query.create_find_options();

    let collection_name = req.param("collection")?;
    let collection = req.state().db.collection(collection_name);
    let count = if query.count == Some(1) {
        Some(collection.count_documents(filter.clone(), None).await?)
    } else {
        None
    };
    let mut cursor = collection.find(filter, find_options).await?;
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

pub async fn insert_object(mut req: Request<State>) -> tide::Result<impl Into<Response>> {
    let mut object = util::parse_request_body(&mut req).await?;
    let now = Utc::now();
    object.insert("createdAt", now);

    let collection_name = req.param("collection")?;
    let collection = req.state().db.collection(collection_name);
    let result = collection.insert_one(object, None).await?;
    let response = Response::builder(StatusCode::Created).body(json!({
        "objectId": result.inserted_id.as_object_id().unwrap().to_hex(),
        "createdAt": util::format_datetime(&now)
    }));
    Ok(response)
}

pub async fn update_object(mut req: Request<State>) -> tide::Result<impl Into<Response>> {
    let id = req.param("id")?;
    let filter = doc! {"_id": ObjectId::with_string(id)?};
    let document = util::parse_request_body(&mut req).await?;
    let mut update = Document::new();
    update.insert("$set", document);
    update.insert("$currentDate", doc! { "updatedAt": true });

    let collection_name = req.param("collection")?;
    let collection = req.state().db.collection(collection_name);
    let result = collection.update_one(filter, update, None).await?;
    Ok(json!(result))
}

pub async fn modify_object(mut req: Request<State>) -> tide::Result<impl Into<Response>> {
    let id = req.param("id")?;
    let filter = doc! {"_id": ObjectId::with_string(id)?};
    let mut document = util::parse_request_body(&mut req).await?;
    document.insert("$currentDate", doc! { "updatedAt": true });

    let collection_name = req.param("collection")?;
    let collection = req.state().db.collection(collection_name);
    let result = collection.update_one(filter, document, None).await?;
    Ok(json!(result))
}

pub async fn delete_object(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let id = req.param("id")?;
    let mut filter = doc! {"_id": ObjectId::with_string(id)?};
    let query: Query = req.query()?;
    if let Some(additional) = query.create_filter()? {
        filter.extend(additional);
    }

    let collection_name = req.param("collection")?;
    let collection = req.state().db.collection(collection_name);
    let result = collection.delete_one(filter, None).await?;
    Ok(json!(result))
}
