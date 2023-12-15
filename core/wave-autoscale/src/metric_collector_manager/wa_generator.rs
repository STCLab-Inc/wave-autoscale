use anyhow::Result;
use data_layer::MetricDefinition;
use tokio::task::JoinHandle;
use tracing::debug;

pub fn run(definition: MetricDefinition, output_url: String) -> Result<JoinHandle<()>> {
    let handle = tokio::spawn(async move {
        let metadata = definition.metadata;
        let (
            Some(serde_json::Value::Array(pattern)),
            Some(gap_seconds),
        ) = (
            metadata.get("pattern"),
            metadata.get("gap_seconds").and_then(|v| v.as_u64()),
        ) else {
            debug!("[wa-generator] Invalid metadata");
            return;
        };

        let url = format!(
            "{}?metric_id={}&collector=wagenerator",
            output_url, definition.id
        );

        loop {
            let url = url.clone();

            for value in pattern.iter() {
                tokio::time::sleep(tokio::time::Duration::from_secs(gap_seconds)).await;

                let value = value.as_f64();
                if value.is_none() {
                    debug!("[wa-generator] Invalid pattern");
                    return;
                }
                let json = serde_json::json!(
                    {
                        "metrics": [
                            {
                                "value": value.unwrap(),
                                "timestamp": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                            }
                        ]
                    }
                );
                let res = reqwest::Client::new()
                    .post(url.clone())
                    .json(&json)
                    .send()
                    .await;
                if let Err(e) = res {
                    debug!("[wa-generator] {}", e);
                }
            }
        }
    });
    Ok(handle)
}

#[cfg(test)]
mod test {
    use super::run;
    use data_layer::MetricDefinition;
    use std::collections::HashMap;

    #[tokio::test]
    #[ignore]
    async fn test_run() {
        let definition = MetricDefinition {
            kind: data_layer::types::object_kind::ObjectKind::Metric,
            db_id: String::from("db_id"),
            id: String::from("metric-id"),
            collector: String::from("wa-generator"),
            metadata: HashMap::from([
                (
                    "pattern".to_string(),
                    serde_json::Value::Array(vec![
                        serde_json::Value::Number(serde_json::Number::from(1)),
                        serde_json::Value::Number(serde_json::Number::from(2)),
                        serde_json::Value::Number(serde_json::Number::from(3)),
                    ]),
                ),
                ("gap_seconds".to_string(), serde_json::json!(1)),
            ]),
        };
        let output_url = "http://localhost:8080".to_string();
        let handle = run(definition, output_url).unwrap();
        handle.await.unwrap();
    }
}
