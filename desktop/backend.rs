//! In-process API transport. No listener or browser-accessible HTTP endpoint.
use crate::{
    AppState,
    app::http::{
        controllers::auth_controller::hash_password_for_request,
        error::{AppError, Result},
        requests::{UserRequest, extractors::Json as Input},
    },
    database,
};
use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode, Uri},
    routing::get,
};
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tower::ServiceExt;

pub struct DesktopBackend {
    router: Router,
    cookie: Mutex<String>,
    pub db: sqlx::AnyPool,
}
#[derive(Deserialize)]
pub struct ApiRequest {
    pub path: String,
    pub method: String,
    pub body: Option<Value>,
    pub csrf: Option<String>,
}
#[derive(Serialize, Debug)]
pub struct ApiResponse {
    pub status: u16,
    pub data: Value,
}

impl DesktopBackend {
    pub fn new(state: AppState) -> Self {
        let setup = Router::new()
            .route("/api/desktop/setup", get(setup_status).post(setup))
            .layer(axum::extract::DefaultBodyLimit::max(16 * 1024))
            .with_state(state.clone());
        Self {
            db: state.db.clone(),
            router: crate::application::router(state).merge(setup),
            cookie: Mutex::new(String::new()),
        }
    }
    pub async fn request(&self, input: ApiRequest) -> std::result::Result<ApiResponse, String> {
        let uri: Uri = input.path.parse().map_err(|_| "Invalid API path")?;
        if uri.scheme().is_some() || uri.authority().is_some() || !uri.path().starts_with("/api/") {
            return Err("Only local API paths are allowed".into());
        }
        let method: Method = input.method.parse().map_err(|_| "Invalid method")?;
        if !matches!(
            method,
            Method::GET | Method::POST | Method::PUT | Method::PATCH | Method::DELETE
        ) {
            return Err("Unsupported method".into());
        }
        let body = input.body.map(|b| b.to_string()).unwrap_or_default();
        if body.len() > 16 * 1024 {
            return Err("Request is too large".into());
        }
        // Serialize this window's requests so cookie rotation cannot race another response.
        let mut cookie = self.cookie.lock().await;
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json");
        if !cookie.is_empty() {
            request = request.header("cookie", cookie.as_str());
        }
        if let Some(csrf) = input.csrf {
            request = request.header("x-csrf-token", csrf);
        }
        let request = request
            .body(Body::from(body))
            .map_err(|_| "Invalid request headers")?;
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .map_err(|_| "Request failed")?;
        if let Some(value) = response
            .headers()
            .get("set-cookie")
            .and_then(|h| h.to_str().ok())
        {
            *cookie = value.split(';').next().unwrap_or_default().to_owned();
        }
        let status = response.status().as_u16();
        let bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|_| "Invalid response")?
            .to_bytes();
        let data = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).map_err(|_| "Invalid JSON response")?
        };
        Ok(ApiResponse { status, data })
    }
}

async fn setup_status(State(state): State<AppState>) -> Result<Json<Value>> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;
    Ok(Json(json!({"needs_setup":count == 0})))
}

#[derive(Deserialize)]
struct SetupInput {
    name: String,
    email: String,
    password: String,
}
async fn setup(
    State(state): State<AppState>,
    Input(input): Input<SetupInput>,
) -> Result<(StatusCode, Json<Value>)> {
    let input = UserRequest {
        name: input.name,
        email: input.email,
        password: input.password,
        role: "admin".into(),
        status: "active".into(),
    }
    .validated(true)?;
    let password = hash_password_for_request(&state, input.password).await?;
    let mut tx = state.db.begin().await?;
    database::auth_lock(&mut tx).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await?;
    if count != 0 {
        return Err(AppError::new(
            StatusCode::CONFLICT,
            "Workspace is already configured. Sign in instead.",
        ));
    }
    sqlx::query(&database::sql(&state.db, "INSERT INTO users (name, email, password, role, status) VALUES (?, ?, ?, 'admin', 'active')"))
        .bind(input.name).bind(input.email).bind(password).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"message":"Workspace created."})),
    ))
}
