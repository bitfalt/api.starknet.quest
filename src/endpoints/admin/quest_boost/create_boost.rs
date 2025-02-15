use crate::models::{BoostTable, JWTClaims, QuestDocument};
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use axum::http::HeaderMap;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateBoostQuery {
    amount: i32,
    token: String,
    num_of_winners: i64,
    token_decimals: i64,
    name: String,
    quest_id: i32,
    hidden: bool,
    expiry: i64,
    img_url: String,
}

#[route(post, "/admin/quest_boost/create_boost")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateBoostQuery>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<BoostTable>("boosts");
    let quests_collection = state.db.collection::<QuestDocument>("quests");

    let res = verify_quest_auth(user, &quests_collection, &(body.quest_id as i64)).await;
    if !res {
        return get_error("Error creating boost".to_string());
    };

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = BoostTable {
        name: body.name.clone(),
        amount: body.amount.clone(),
        token_decimals: body.token_decimals.clone(),
        token: body.token.clone(),
        expiry: body.expiry.clone(),
        num_of_winners: body.num_of_winners.clone(),
        quests: vec![body.quest_id.clone()],
        id: next_id,
        hidden: body.hidden.clone(),
        img_url: body.img_url.clone(),
        winner: None,
    };

    // insert document to boost collection
    return match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Boost created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating boosts".to_string()),
    };
}
