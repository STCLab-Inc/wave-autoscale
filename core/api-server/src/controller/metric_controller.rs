use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use data_layer::MetricDefinition;
use serde::Deserialize;
use validator::Validate;

use crate::app_state::{self, AppState};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_metrics)
        .service(post_metrics)
        .service(put_metric_by_id)
        .service(delete_metric_by_id);
}

#[get("/metrics")]
async fn get_metrics(app_state: web::Data<AppState>) -> impl Responder {
    // HttpResponse::Ok().body("Hello world!")
    // const metrics = &app
    let metrics = app_state.data_layer.get_all_metrics().await;
    if metrics.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", metrics));
    }
    HttpResponse::Ok().json(metrics.unwrap())
}

// [POST] /metrics
#[derive(Deserialize, Validate)]
struct PostMetricsRequest {
    metrics: Vec<MetricDefinition>,
}

#[post("/metrics")]
async fn post_metrics(
    request: web::Json<PostMetricsRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let result = app_state
        .data_layer
        .add_metrics(request.metrics.clone())
        .await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    HttpResponse::Ok().body("ok")
}

#[put("/metrics/{db_id}")]
async fn put_metric_by_id(
    db_id: web::Path<String>,
    request: web::Json<MetricDefinition>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let mut metric = request.into_inner();
    metric.db_id = db_id.into_inner();

    let result = app_state.data_layer.update_metric(metric).await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    HttpResponse::Ok().body("ok")
}

#[delete("/metrics/{db_id}")]
async fn delete_metric_by_id(
    db_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let result = app_state.data_layer.delete_metric(db_id.into_inner()).await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    HttpResponse::Ok().body("ok")
}

// use validator::ValidationError;
// fn validate_username(username: &str) -> Result<(), ValidationError> {
//     // todo: use regex for robust
//     if first_char_is_number(username) {
//         return Err(ValidationError::new(
//             "terrible_username: first char is number",
//         ));
//     }

//     if username.contains("@") {
//         // the value of the username will automatically be added later
//         return Err(ValidationError::new("terrible_username: contains @"));
//     }

//     Ok(())
// }

// pub fn first_char_is_number(s: &str) -> bool {
//     s.get(0..1).and_then(|c| c.parse::<u8>().ok()).is_some()
// }

// // [GET] /users
// #[get("/users")]
// async fn get_users(app_state: web::Data<AppState>) -> impl Responder {
//     let result = UserRepository::get_users(&app_state.db_context).await;
//     match result {
//         Ok(users) => HttpResponse::Ok().json(users),
//         Err(error) => HttpResponse::InternalServerError().body(format!("{:?}", error)),
//     }
// }

// // [POST] /users
// #[derive(Deserialize, Validate)]
// struct RegisterForm {
//     #[validate(length(min = 3, max = 33), custom = "validate_username")]
//     username: String,
//     name: String,
//     #[validate(email)]
//     email: String,
//     password: String,
// }

// #[post("/users")]
// async fn add_users(
//     form: web::Json<RegisterForm>,
//     app_state: web::Data<AppState>,
// ) -> impl Responder {
//     let form = form.into_inner();
//     if let Err(error) = form.validate() {
//         return HttpResponse::BadRequest().body(error.to_string());
//     }
//     let user = User {
//         id: Uuid::new_v4(),
//         name: form.name,
//         username: form.username,
//         email: form.email,
//     };
//     let result = UserRepository::create_user(&app_state.db_context, &user).await;
//     match result {
//         Ok(_) => HttpResponse::Ok().body(format!("{:?}", user)),
//         Err(error) => HttpResponse::InternalServerError().body(format!("{:?}", error)),
//     }
// }
