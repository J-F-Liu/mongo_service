use chrono::{DateTime, SecondsFormat, Utc};
use mongodb::bson::{document::ValueAccessResult, Bson, Document};
use std::convert::TryFrom;
use tide::{Error, Request, StatusCode};

#[inline]
pub fn format_datetime(time: &DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub fn make_json_friendly(document: &mut Document) -> ValueAccessResult<()> {
    let time_fields = document
        .keys()
        .filter_map(|key: &String| {
            document
                .get_datetime(key)
                .map(|time| (key.clone(), format_datetime(time)))
                .ok()
        })
        .collect::<Vec<_>>();
    for (key, time) in time_fields {
        if let Some(value) = document.get_mut(&key) {
            *value = Bson::String(time)
        }
    }
    if let Ok(id) = document.get_object_id("_id") {
        let object_id = id.to_hex();
        document.remove("_id");
        document.insert("objectId", object_id);
    }
    Ok(())
}

pub async fn parse_request_body<State>(req: &mut Request<State>) -> tide::Result<Document> {
    let value: serde_json::Value = req
        .body_json()
        .await
        .map_err(|_| Error::from_str(StatusCode::BadRequest, "Invalid json"))?;
    if let Ok(Bson::Document(document)) = Bson::try_from(value) {
        Ok(document)
    } else {
        Err(Error::from_str(
            StatusCode::BadRequest,
            "Expect json object in request body",
        ))
    }
}
