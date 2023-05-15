use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

use crate::{entities, PluginState};

/// To-Do item
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Todo {
    id: i32,
    #[schema(example = "Buy groceries")]
    item: String,
}

#[derive(Deserialize, IntoParams)]
pub struct Username {
    /// The name of the user.
    username: String,
}

impl From<entities::todo::Model> for Todo {
    fn from(value: entities::todo::Model) -> Self {
        Self {
            id: value.id,
            item: value.item,
        }
    }
}

/// List all Todo items for the given user.
#[utoipa::path(
    get,
    path = "/todo/{username}",
    params(Username),
    responses(
        (status = 200, description = "List user's To-Dos", body = [Todo]),
        (status = 500, description = "Database operation error"),
    )
)]
pub async fn list_todos(
    State(state): State<Arc<PluginState>>,
    Path(Username { username }): Path<Username>,
) -> Result<Json<Vec<Todo>>, StatusCode> {
    Ok(Json(
        entities::todo::Entity::find()
            .filter(entities::todo::Column::Username.eq(&username))
            .all(&state.database)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .into_iter()
            .map(|model| model.into())
            .collect(),
    ))
}

/// To-Do create request
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct TodoCreate {
    #[schema(example = "Buy groceries")]
    item: String,
}

/// Adds a new To-Do item to the database
#[utoipa::path(
    post,
    path = "/todo/{username}",
    params(Username),
    request_body = TodoCreate,
    responses(
        (status = 201, description = "Todo item created successfully"),
        (status = 500, description = "Database operation error"),
    )
)]
pub async fn create_todo(
    State(state): State<Arc<PluginState>>,
    Path(Username { username }): Path<Username>,
    Json(todo): Json<TodoCreate>,
) -> impl IntoResponse {
    let mut new_todo = entities::todo::ActiveModel::new();
    new_todo.username = sea_orm::Set(username);
    new_todo.item = sea_orm::Set(todo.item);

    match new_todo.insert(&state.database).await {
        Ok(_) => StatusCode::CREATED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// To-Do delete request
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct TodoDelete {
    id: i32,
}

/// Delete To-Do by the given ID.
#[utoipa::path(
    put,
    path = "/todo/{username}",
    responses(
        (status = 200, description = "To-Do deleted successfully"),
        (status = 403, description = "This user is not permitted to delete the given To-Do"),
        (status = 404, description = "The given To-Do was not found."),
        (status = 500, description = "Database operation error"),
    ),
    params(Username),
    request_body = TodoDelete,
)]
pub async fn delete_todo(
    State(state): State<Arc<PluginState>>,
    Path(Username { username }): Path<Username>,
    Json(TodoDelete { id }): Json<TodoDelete>,
) -> Result<StatusCode, StatusCode> {
    let todo = entities::todo::Entity::find_by_id(id)
        .one(&state.database)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if todo.username != username {
        return Err(StatusCode::FORBIDDEN);
    }

    todo.delete(&state.database)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
