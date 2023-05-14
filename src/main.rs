use axum::{routing, Router, Server};
use hyper::Error;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::Mutex;
use utoipa::OpenApi;

mod plugin;
mod todo;

#[derive(Default)]
pub struct PluginState {
    store: Mutex<Vec<todo::Todo>>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let port = 3030;
    let base_url = format!("http://localhost:{port}");

    let manifest = plugin::Manifest::builder()
        .schema_version("v1")
        .name_for_human("To-do List")
        .name_for_model("todo list")
        .description_for_human("Create, update, and complete todo lists!")
        .description_for_model("plugin for creating, updating, and removing todo lists")
        .auth(plugin::ManifestAuth::None)
        .api(plugin::ManifestApi::Openapi {
            url: format!("{base_url}/openapi.json"),
            is_user_authenticated: false,
        })
        .logo_url(format!("{base_url}/logo.json"))
        .contact_email("example@gmail.com")
        .legal_info_url("http://example.com/legal")
        .build();

    #[derive(OpenApi)]
    #[openapi(
        paths(
            todo::list_todos,
            todo::search_todos,
            todo::create_todo,
            todo::mark_done,
            todo::delete_todo,
        ),
        components(
            schemas(todo::Todo, todo::TodoError)
        ),
        tags(
            (name = "todo", description = "Todo items management API")
        )
    )]
    struct ApiDoc;

    let state = Arc::new(Default::default());

    let app = Router::new()
        .route(
            "/todo",
            routing::get(todo::list_todos).post(todo::create_todo),
        )
        .route("/todo/search", routing::get(todo::search_todos))
        .route(
            "/todo/:id",
            routing::put(todo::mark_done).delete(todo::delete_todo),
        )
        .with_state(state)
        .merge(plugin::serve_plugin_info(manifest, ApiDoc::openapi(), "assets/plugin.png"));

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    Server::bind(&address).serve(app.into_make_service()).await
}
