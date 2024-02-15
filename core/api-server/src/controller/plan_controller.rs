use crate::app_state::AppState;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use data_layer::ScalingPlanDefinition;
use serde::Deserialize;
use tracing::{debug, error};
use validator::Validate;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_plans)
        .service(get_plan_by_id)
        .service(post_plans)
        .service(put_plan_by_id)
        .service(delete_plan_by_id)
        .service(run_plan);
}

#[get("/api/plans")]
async fn get_plans(app_state: web::Data<AppState>) -> impl Responder {
    let plans = app_state.data_layer.get_all_plans_json().await;
    if plans.is_err() {
        error!("Failed to get plans: {:?}", plans);
        return HttpResponse::InternalServerError().body(format!("{:?}", plans));
    }
    HttpResponse::Ok().json(plans.unwrap())
}

#[get("/api/plans/{db_id}")]
async fn get_plan_by_id(
    db_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Getting plan by id: {}", db_id);
    let plan = app_state
        .data_layer
        .get_plan_by_id(db_id.into_inner())
        .await;
    if plan.is_err() {
        error!("Failed to get plan: {:?}", plan);
        return HttpResponse::InternalServerError().body(format!("{:?}", plan));
    }
    let plan = plan.unwrap();
    debug!("Got plan: {:?}", plan);
    HttpResponse::Ok().json(plan)
}

#[derive(Deserialize, Validate)]
struct PostPlansRequest {
    plans: Vec<ScalingPlanDefinition>,
}

#[post("/api/plans")]
async fn post_plans(
    request: web::Json<PostPlansRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Adding plans: {:?}", request.plans);
    let result = app_state.data_layer.add_plans(request.plans.clone()).await;
    if result.is_err() {
        error!("Failed to add plans: {:?}", result);
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    debug!("Added plans: {:?}", request.plans);
    HttpResponse::Ok().body("ok")
}

#[put("/api/plans/{db_id}")]
async fn put_plan_by_id(
    db_id: web::Path<String>,
    request: web::Json<ScalingPlanDefinition>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let mut plan = request.into_inner();
    debug!("Updating plan: {:?}", plan);
    plan.db_id = db_id.into_inner();

    let result = app_state.data_layer.update_plan(plan).await;
    if result.is_err() {
        error!("Failed to update plan: {:?}", result);
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    debug!("Updated plan");
    HttpResponse::Ok().body("ok")
}

#[delete("/api/plans/{db_id}")]
async fn delete_plan_by_id(
    db_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Deleting plan by id: {}", db_id);
    let result = app_state.data_layer.delete_plan(db_id.into_inner()).await;
    if result.is_err() {
        error!("Failed to delete plan: {:?}", result);
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    debug!("Deleted plan");
    HttpResponse::Ok().body("ok")
}

#[derive(Deserialize, Debug)]
struct RunPlanRequest {
    plan_id: String,
    plan_item_id: String,
}
#[post("/api/run-plan")]
async fn run_plan(
    request: web::Json<RunPlanRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Running plan: {:?}", request);
    let result = app_state
        .data_layer
        .send_plan_action(request.plan_id.clone(), request.plan_item_id.clone());

    if result.is_err() {
        error!("Failed to run plan: {:?}", result);
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    debug!("Ran plan");
    HttpResponse::Ok().body("ok")
}

#[cfg(test)]
mod tests {
    use crate::utils::test_utils::get_app_state_for_test;

    use super::init;
    use actix_web::{test, App};
    use data_layer::{
        data_layer::DataLayer,
        types::{object_kind::ObjectKind, plan_item_definition::PlanItemDefinition},
    };
    use serde_json::json;
    use std::collections::HashMap;

    // Utility functions

    async fn add_plans_for_test(data_layer: &DataLayer) {
        let _ = data_layer
            .add_plans(vec![
                data_layer::ScalingPlanDefinition {
                    id: "test1".to_string(),
                    db_id: "test1".to_string(),
                    kind: ObjectKind::ScalingPlan,
                    metadata: HashMap::new(),
                    variables: HashMap::from([
                        ("test1".to_string(), json!("test1")),
                        ("test2".to_string(), json!("test2")),
                    ]),
                    plans: vec![PlanItemDefinition {
                        id: "test1".to_string(),
                        description: None,
                        expression: None,
                        cron_expression: None,
                        cool_down: None,
                        ui: None,
                        priority: 1,
                        scaling_components: vec![json!({
                            "name": "test1",
                            "value": 1
                        })],
                    }],
                    enabled: true,
                },
                data_layer::ScalingPlanDefinition {
                    id: "test2".to_string(),
                    db_id: "test2".to_string(),
                    kind: ObjectKind::ScalingPlan,
                    metadata: HashMap::new(),
                    variables: HashMap::from([
                        ("test1".to_string(), json!("test1")),
                        ("test2".to_string(), json!("test2")),
                    ]),
                    plans: vec![PlanItemDefinition {
                        id: "test2".to_string(),
                        description: None,
                        expression: None,
                        cron_expression: None,
                        cool_down: None,
                        ui: None,
                        priority: 1,
                        scaling_components: vec![json!({
                            "name": "test2",
                            "value": 2
                        })],
                    }],
                    enabled: true,
                },
            ])
            .await;
    }

    // [GET] /api/plans (get_all_plans_json)

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_get_all_plans_json() {
        let app_state = get_app_state_for_test().await;
        add_plans_for_test(&app_state.data_layer).await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
        let req = test::TestRequest::get().uri("/api/plans").to_request();
        let resp: Vec<serde_json::Value> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 2);
        for plan in resp.iter() {
            if let Some(created_at) = plan.get("created_at").and_then(|v| v.as_str()) {
                assert!(!created_at.is_empty());
            } else {
                panic!("created_at field is missing or not a string");
            }

            if let Some(updated_at) = plan.get("updated_at").and_then(|v| v.as_str()) {
                assert!(!updated_at.is_empty());
            } else {
                panic!("updated_at field is missing or not a string");
            }

            if let Some(variables) = plan.get("variables").and_then(|v| v.as_object()) {
                assert_eq!(variables.len(), 2);
            } else {
                panic!("variables field is missing or not an object");
            }
        }
    }

    // [GET] /api/run-plan
    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_run_plan() {
        let app_state = get_app_state_for_test().await;
        let mut receiver = app_state.data_layer.subscribe_action();
        // add_plans_for_test(&app_state.data_layer).await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
        let req = test::TestRequest::post()
            .uri("/api/run-plan")
            .set_json(json!({
                "plan_id": "plan_id_1",
                "plan_item_id": "plan_item_id_1"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let receiver_result = receiver.recv().await;
        assert!(receiver_result.is_ok());

        let value = receiver_result.unwrap();
        assert_eq!(value.get("plan_id").unwrap(), "plan_id_1");
        assert_eq!(value.get("plan_item_id").unwrap(), "plan_item_id_1");
    }
}
