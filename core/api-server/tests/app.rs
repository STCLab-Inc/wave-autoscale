#[cfg(test)]
mod api_server {
    use actix_web::{test, App};
    use api_server::{
        app_state::{get_app_state, GetAppStateParam},
        controller,
    };
    use data_layer::MetricDefinition;
    use serde_json::json;

    #[tokio::test]
    async fn test_api_server_metric() {
        const TEST_DB: &str = "sqlite://./tests/data-layer/test.db";
        // Delete the test db if it exists
        let path = std::path::Path::new(TEST_DB.trim_start_matches("sqlite://"));
        let _ = std::fs::remove_file(path);

        let app_state = get_app_state(GetAppStateParam {
            sql_url: TEST_DB.to_owned(),
        })
        .await;

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .configure(controller::init_metric_controller),
        )
        .await;

        let id1 = uuid::Uuid::new_v4().to_string();
        let id2 = uuid::Uuid::new_v4().to_string();
        let payload = json!({
            "metrics": [{
            "kind": "Metric",
            "id": id1,
            "metric_kind": "prometheus",
            "metadata": {
                "name": "test_metric_1",
                "description": "test_metric_1",
                "unit": "test_metric_1",
                "labels": {
                    "test_metric_1": "test_metric_1",
                },
            },
        }, {
            "kind": "Metric",
            "id": id2,
            "metric_kind": "cloudwatch",
            "metadata": {
                "name": "test_metric_2",
                "description": "test_metric_2",
                "unit": "test_metric_2",
                "labels": {
                    "test_metric_2": "test_metric_2",
                },
            },
        }]});

        let post_metrics = test::TestRequest::post()
            .uri("/metrics")
            .set_json(payload)
            .to_request();
        let resp = test::call_service(&app, post_metrics).await;
        println!("resp: {:?}", resp);
        assert!(resp.status().is_success(), "Response status is not success");

        let get_metrics = test::TestRequest::get().uri("/metrics").to_request();
        let resp: Vec<MetricDefinition> = test::call_and_read_body_json(&app, get_metrics).await;
        println!("resp: {:?}", resp);
        assert!(resp.len() == 2, "Response status is not success");

        // Update a metric
        let mut metric = resp[0].clone();
        metric.metric_kind = "new_prometheus".to_owned();
        let update_metric = test::TestRequest::put()
            .uri(&format!("/metrics/{}", metric.db_id))
            .set_json(&metric)
            .to_request();
        let resp = test::call_service(&app, update_metric).await;
        println!("resp: {:?}", resp);
        assert!(resp.status().is_success(), "Response status is not success");

        // Delete a metric
        let delete_metric = test::TestRequest::delete()
            .uri(&format!("/metrics/{}", metric.db_id))
            .to_request();
        let resp = test::call_service(&app, delete_metric).await;
        println!("resp: {:?}", resp);
        assert!(resp.status().is_success(), "Response status is not success");
    }
}
