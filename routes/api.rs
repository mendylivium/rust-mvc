use crate::{
    AppState,
    app::http::{
        controllers::{
            auth_controller as auth, dashboard_controller as dashboard, user_controller as users,
        },
        middleware::session,
    },
};
use axum::{
    Router, middleware,
    routing::{get, post},
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/auth/session", get(auth::current))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/profile", axum::routing::patch(auth::profile))
        .route("/dashboard", get(dashboard::index))
        .route("/users", get(users::index).post(users::store))
        .route(
            "/users/{id}",
            axum::routing::put(users::update).delete(users::destroy),
        )
        .fallback(|| async {
            (
                axum::http::StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({"message":"API route not found."})),
            )
        })
        .layer(middleware::from_fn_with_state(state, session::csrf))
}
