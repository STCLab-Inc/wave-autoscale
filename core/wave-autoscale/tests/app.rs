#[cfg(test)]
mod app_test {
    use std::time::Duration;

    use anyhow::Result;
    use tokio::time::sleep;
    use wave_autoscale::{app, args::Args};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // multithreaded test
    #[tokio::test]
    async fn test_run() -> Result<()> {
        init();
        let mut app = app::App::new(Args {
            plan: Some("tests/yaml/plan_prometheus_ec2.yaml".to_string()),
            config: None,
        })
        .await;
        app.run().await;

        sleep(Duration::from_secs(10)).await;
        Ok(())
    }
}
