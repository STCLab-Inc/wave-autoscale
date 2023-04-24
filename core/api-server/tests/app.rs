#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use api_server::{app_state::get_app_state, controller};

    #[tokio::test]
    async fn test_api_server_metric() {
        let app_state = get_app_state();

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .configure(controller::init_metric_controller),
        )
        .await;
        let req = test::TestRequest::get().uri("/metric").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "Response status is not success");
    }
}
