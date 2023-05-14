use axum::{routing, Router, Server};
use hyper::Error;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use utoipa::OpenApi;

mod entities;
mod plugin;
mod todo;

#[derive(Default)]
pub struct PluginState {
    database: DatabaseConnection,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let port = 3030;
    let base_url = format!("http://localhost:{port}");

    let manifest = plugin::Manifest::builder()
        .schema_version("v1")
        .name_for_human("To-Do Plugin")
        .name_for_model("todo")
        .description_for_human(
            "Plugin for managing a TODO list, you can add, remove and view your TODOs.",
        )
        .description_for_model(
            "Plugin for managing a TODO list, you can add, remove and view your TODOs.",
        )
        .auth(plugin::ManifestAuth::None)
        .api(plugin::ManifestApi::Openapi {
            url: format!("{base_url}/openapi.json"),
            is_user_authenticated: false,
        })
        .logo_url(format!("{base_url}/logo.json"))
        .contact_email("support@example.com")
        .legal_info_url("http://example.com/legal")
        .build();

    #[derive(OpenApi)]
    #[openapi(
        paths(
            todo::list_todos,
            todo::create_todo,
            todo::delete_todo,
        ),
        components(
            schemas(todo::Todo, todo::TodoCreate, todo::TodoDelete)
        ),
        tags(
            (name = "todo", description = "Todo items management API")
        )
    )]
    struct ApiDoc;

    dotenv::dotenv().ok();
    let db_url = std::env::var("DB_URL").expect("DB_URL not set");
    let db_name = std::env::var("DB_NAME").expect("DB_NAME not set");
    let database = Database::connect(format!("{db_url}/{db_name}"))
        .await
        .expect("error connecting to database");

    migration::Migrator::up(&database, None)
        .await
        .expect("error syncing database");

    let state = Arc::new(PluginState { database });

    let app = Router::new()
        .route(
            "/todos/:username",
            routing::get(todo::list_todos)
                .post(todo::create_todo)
                .delete(todo::delete_todo),
        )
        .with_state(state)
        .merge(plugin::serve_plugin_info(
            manifest,
            ApiDoc::openapi(),
            "assets/plugin.png",
        ));

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    Server::bind(&address).serve(app.into_make_service()).await
}
