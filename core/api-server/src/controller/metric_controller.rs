use actix_web::{get, post, web, HttpResponse, Responder};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get);
}

#[get("/metric")]
async fn get() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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
