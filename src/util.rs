use chrono::{DateTime, SecondsFormat, Utc};
use mongodb::bson::{document::ValueAccessResult, Bson, Document};

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
    let id = document.get_object_id("_id")?.to_hex();
    document.remove("_id");
    document.insert("objectId", id);
    Ok(())
}
