use crate::app_state::AppState;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use tracing::{debug, error};
use validator::Validate;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(post_defintions);
}

#[derive(Deserialize, Validate)]
struct PostDefinitionsRequest {
    yaml: String,
}

#[post("/api/definitions")]
async fn post_defintions(
    request: web::Json<PostDefinitionsRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Adding the definition: {:?}", request.yaml);
    let result = app_state
        .data_layer
        .add_definitions(request.yaml.as_str())
        .await;
    if result.is_err() {
        error!("Failed to add plans: {:?}", result);
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    debug!("Added definitions");
    HttpResponse::Ok().body("ok")
}

#[cfg(test)]
mod tests {
    use super::init;
    use crate::utils::test_utils::get_app_state_for_test;
    use actix_web::{test, App};
    use serde_json::json;

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_post_definitions() {
        let app_state = get_app_state_for_test().await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;

        let request = json!({
            "yaml":
r#"
---
kind: Metric
# id should be unique. And alphanumeric characters and underscores are allowed.
# for example, "metric_id" is valid but "metric-id" is not.
id: metric_id
metadata:
  user_key: user_value
---
kind: ScalingComponent
id: scaling_component_id
component_kind: ec2-autoscaling
metadata:
  access_key: access_key
  secret_key: secret_key
---
kind: ScalingPlan
id: scaling_plan_id
metadata:
  title: scaling_plan_title
plans:
  - id: plan_id
    expression: "metric_id >= 30"
    priority: 1
    scaling_components:
    - id: scaling_component_id
      desired: "Math.floor(metric_id / 10)"
      min: 1
      max: 5
      cooldown: 300
"#
        });
        let req = test::TestRequest::post()
            .uri("/api/definitions")
            .set_json(&request)
            .to_request();
        let response = test::call_service(&app, req).await;
        assert!(response.status().is_success());
    }

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_post_definitions_with_invalid_yaml() {
        let app_state = get_app_state_for_test().await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;

        let request = json!({
            "yaml":
r#"
---
kind----this--line--is--invalid
id: metric_id
metadata:
  user_key: user_value
---
kind: ScalingComponent
id: scaling_component_id
component_kind: ec2-autoscaling
metadata:
  access_key: access_key
  secret_key: secret_key
---
kind: ScalingPlan
id: scaling_plan_id
metadata:
  title: scaling_plan_title
plans:
  - id: plan_id
    expression: "metric_id >= 30"
    priority: 1
    scaling_components:
    - id: scaling_component_id
      desired: "Math.floor(metric_id / 10)"
      min: 1
      max: 5
      cooldown: 300
"#
        });
        let req = test::TestRequest::post()
            .uri("/api/definitions")
            .set_json(&request)
            .to_request();
        let response = test::call_service(&app, req).await;
        assert!(!response.status().is_success());
    }
}
