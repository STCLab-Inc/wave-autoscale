use data_layer::data_layer::DataLayer;
use data_layer::types::source_metrics::SourceMetrics;
use get_size::GetSize;
use serde_json::json;
use std::fs::File;
use std::io::Read;
use ulid::Ulid;

#[tokio::test]
async fn performance_test_add_source_metrics_in_data_layer() {
    const SAMPLE_DATA_10MB_FILE_PATH: &str = "./tests/sample/source_metrics_performance_10MB.txt";
    const DB_URL: &str = "";
    const METRIC_BUFFER_SIZE_KB: u64 = 100_000; // 100 MB
    let data_layer = DataLayer::new(DB_URL, METRIC_BUFFER_SIZE_KB).await;
    let ulid_size = Ulid::new().to_string().get_heap_size();

    // read sample data file
    let mut file = File::open(SAMPLE_DATA_10MB_FILE_PATH).unwrap();
    let mut sample_data_10mb = String::new();
    file.read_to_string(&mut sample_data_10mb).unwrap();
    let sample_data_1mb = sample_data_10mb.clone();
    let _sample_data_1mb = sample_data_1mb.split_at(1_000_000).0.to_string();

    // START
    let start_time = chrono::Utc::now();
    println!(
        "- start time: {}",
        start_time.format("%Y-%m-%d %H:%M:%S.%f")
    );

    // 1. 100 MB sample data save - 65 Byte * 1,538,461 = 99,999,965 Byte
    let mut sample_total_size_100mb = 0;
    for _i in 0..1_538_461 {
        // sample data - 65 Byte
        let sample_metric_id = "1".to_string();
        let sample_json_value = json!([{"2": "3"}]).to_string();
        let sample_total_size = add_source_metrics_in_data_layer_save_test_data(
            &data_layer,
            sample_metric_id,
            sample_json_value,
            ulid_size,
        )
        .await;
        sample_total_size_100mb += sample_total_size;
    }

    let save_time_100mb = chrono::Utc::now();
    println!(
        "- 100 MB save time: {} (duration: {})",
        save_time_100mb.format("%Y-%m-%d %H:%M:%S.%f"),
        (save_time_100mb - start_time)
    );

    // 2. 10MB sample data save - 10,000,063 Byte
    let sample_metric_id = "2".to_string();
    let sample_json_value = json!([{ "2": sample_data_10mb }]).to_string();
    let _sample_total_size_10mb = add_source_metrics_in_data_layer_save_test_data(
        &data_layer,
        sample_metric_id,
        sample_json_value,
        ulid_size,
    )
    .await;

    let save_time_10mb = chrono::Utc::now();
    println!(
        "- 10 MB save time: {} (duration: {})",
        save_time_10mb.format("%Y-%m-%d %H:%M:%S.%f"),
        (save_time_10mb - save_time_100mb)
    );

    let end_time = chrono::Utc::now();
    println!("- end time: {}", end_time.format("%Y-%m-%d %H:%M:%S.%f"));

    println!("- total duration: {}", end_time - start_time);
}

async fn add_source_metrics_in_data_layer_save_test_data(
    data_layer: &DataLayer,
    metric_id: String,
    json_value: String,
    ulid_size: usize,
) -> usize {
    // sample data 1
    let metric_id_size = metric_id.get_heap_size();
    let json_value_data = SourceMetrics {
        json_value: json_value.to_string(),
    };
    let sample_data_total_size =
        json_value_data.get_heap_size() + (metric_id_size * 2) + (ulid_size * 2);

    // save sample data 1
    let _ = data_layer
        .add_source_metrics_in_data_layer("vector", &metric_id, &json_value)
        .await;
    sample_data_total_size
}
