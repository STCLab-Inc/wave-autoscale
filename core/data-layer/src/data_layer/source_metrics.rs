use super::{DataLayer, SOURCE_METRICS_DATA};
use crate::types::source_metrics::SourceMetrics;
use anyhow::{anyhow, Result};
use get_size::GetSize;
use serde_json::json;
use sqlx::Row;
use std::{
    collections::{BTreeMap, HashMap},
    time::{Duration, SystemTime},
};
use tracing::{debug, error};
use ulid::Ulid;

impl DataLayer {
    // Source Metrics
    pub async fn add_source_metrics_in_data_layer(
        &self,
        collector: &str,
        metric_id: &str,
        json_value: &str,
    ) -> Result<()> {
        /* [ Comment ]
         *  source metrics: Metric data is separated by metric_id and ulid is sorted in ascending order (using for ScalingPlan search)
         *  source metrics metadata: Metric data is sorted in ASC order by ULID (using for remove target data to maintain buffer size)
         * [ Data structure ]
         *  source metrics - HashMap<key: metric_id, value: BTreeMap<key: ULID, value: SourceMetrics>>
         *  source metrics metadata - LinkedList<(metric_id, ULID, data size(source metrics + source metrics metadata)> - list order by ULID ASC */
        let Ok(mut source_metrics_data) = self.source_metrics_data.write() else {
            error!("[add_source_metrics_in_data_layer] Failed to get source metrics data");
            return Err(anyhow!("Failed to get source metrics data"));
        };

        let now_ulid = Ulid::new().to_string();
        let source_metric_insert_data = SourceMetrics {
            json_value: json_value.to_string(),
        };
        // source_metric_insert_data_size = source metrics + source metrics metadata
        let source_metric_insert_data_size = source_metric_insert_data.get_heap_size()
            + (metric_id.to_string().get_heap_size() * 2)
            + (now_ulid.get_heap_size() * 2);

        let mut total_source_metrics_size =
            source_metrics_data.source_metrics_size + source_metric_insert_data_size;

        // get remove target data
        let mut remove_target_data: Vec<(String, String, usize)> = Vec::new();
        let mut subtract_total_size = total_source_metrics_size;
        loop {
            // check size :: buffersize - total size < 0 (None) => remove target data
            if source_metrics_data
                .metric_buffer_size_byte
                .checked_sub(subtract_total_size as u64)
                .is_some()
            {
                break;
            }
            let Some(front_source_metrics_metadata) = source_metrics_data.source_metrics_metadata.pop_front() else {
                break;
            };
            subtract_total_size -= front_source_metrics_metadata.2; // oldest metadata size
            remove_target_data.append(&mut vec![front_source_metrics_metadata]);
        }

        // remove target data
        remove_target_data
            .iter()
            .for_each(|(metric_id, ulid, size)| {
                let Some(source_metrics_map) = source_metrics_data.source_metrics.get_mut(metric_id) else {
                    return;
                };
                // remove source metrics - btreemap ulid
                source_metrics_map.remove(ulid);
                // (if btreemap is empty) remove source metrics - hashmap metric_id
                if source_metrics_map.is_empty() {
                    source_metrics_data.source_metrics.remove(metric_id);
                }
                total_source_metrics_size -= size;
            });

        // add source metrics
        match source_metrics_data.source_metrics.get_mut(metric_id) {
            Some(source_metrics_map) => {
                source_metrics_map.insert(now_ulid.clone(), source_metric_insert_data.clone());
            }
            None => {
                let mut source_metrics_map = BTreeMap::new();
                source_metrics_map.insert(now_ulid.clone(), source_metric_insert_data.clone());
                source_metrics_data
                    .source_metrics
                    .insert(metric_id.to_string(), source_metrics_map);
            }
        }
        // add source metrics metadata
        source_metrics_data.source_metrics_metadata.push_back((
            metric_id.to_string(),
            now_ulid,
            source_metric_insert_data_size,
        ));

        debug!(
            "[source metrics] add item size: {} / total size: {}",
            source_metric_insert_data_size, total_source_metrics_size
        );

        // update source_metric_size
        source_metrics_data.source_metrics_size = total_source_metrics_size;

        // debug!("Save Metric\n{:?}", source_metric_insert_data);

        // Save to database
        if source_metrics_data.enable_metrics_log {
            let result_save_db = self
                .add_source_metric(collector, metric_id, json_value)
                .await;
            if result_save_db.is_err() {
                error!("Failed to save metric into the DB: {:?}", result_save_db);
            }
        }
        Ok(())
    }

    // Add a SourceMetric to the database
    pub async fn add_source_metric(
        &self,
        collector: &str,
        metric_id: &str,
        json_value: &str,
    ) -> Result<()> {
        // TODO: Validate json_value
        // json_value has to follow the below format('tags' is optional)
        // {
        //     "name": "cpu_usage",
        //     "tags": {
        //        "host": "localhost",
        //        "region": "us-west"
        //     },
        //    "value": 0.64,
        // }
        //
        let query_string =
            "INSERT INTO source_metrics (id, collector, metric_id, json_value) VALUES ($1,$2,$3,$4)";
        // ULID as id instead of UUID because of the time based sorting
        let id = Ulid::new().to_string();
        let result = sqlx::query(query_string)
            // VALUE
            .bind(id)
            .bind(collector)
            .bind(metric_id)
            .bind(json_value)
            .execute(&self.pool)
            .await;
        debug!("result: {:?}", result);
        if result.is_err() {
            let error_message = result.err().unwrap().to_string();
            error!("Error: {}", error_message);
            return Err(anyhow!(error_message));
        }
        debug!("Added a source metric: {}", metric_id);
        Ok(())
    }
    // Get a latest metrics by collector and metric_id from the database
    pub async fn get_source_metrics_values(
        &self,
        metric_ids: Vec<String>,
        time_greater_than: u64,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Subtract the duration from the current time
        //
        let offset_time = SystemTime::now() - Duration::from_millis(time_greater_than);
        let ulid = Ulid::from_datetime(offset_time);
        let query_string =
            "SELECT metric_id, id, json_value FROM source_metrics WHERE id >= $1 and metric_id in ($2) ORDER BY id DESC LIMIT 1";
        let metric_ids = metric_ids.join(",");
        let result = sqlx::query(query_string)
            .bind(ulid.to_string())
            .bind(metric_ids)
            .fetch_all(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        let mut metric_values: HashMap<String, serde_json::Value> = HashMap::new();
        for row in result {
            let metric_id: String = row.try_get("metric_id")?;
            let json_value: String = row.try_get("json_value")?;
            let json_value = json!(json_value);
            metric_values.insert(metric_id, json_value);
        }
        Ok(metric_values)
    }

    // Get a latest metrics by collector from the database
    pub async fn get_source_metrics_values_all_metric_ids(
        &self,
        read_before_ms: u64,
    ) -> Result<Vec<serde_json::Value>> {
        let offset_time = SystemTime::now() - Duration::from_millis(read_before_ms);
        let ulid = Ulid::from_datetime(offset_time);
        let query_string = "SELECT metric_id, id, json_value FROM source_metrics WHERE id >= $1";
        let result = sqlx::query(query_string)
            .bind(ulid.to_string())
            .fetch_all(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        let mut metric_values: Vec<serde_json::Value> = Vec::new();
        for row in result {
            let metric_id: String = row.try_get("metric_id")?;
            let id: String = row.try_get("id")?;
            let json_value: String = row.try_get("json_value")?;
            let json_value = json!({"metric_id": metric_id, "id": id, "json_value": json_value});
            metric_values.append(&mut vec![json_value]);
        }
        Ok(metric_values)
    }

    // Get inflow metric id
    pub async fn get_inflow_metric_ids(&self) -> Result<Vec<String>> {
        let mut metric_ids: Vec<String> = Vec::new();

        // Acquire read lock on source metrics data
        let source_metrics_data = match SOURCE_METRICS_DATA.read() {
            Ok(data) => data,
            Err(err) => {
                eprintln!(
                    "Failed to acquire read lock on SOURCE_METRICS_DATA: {}",
                    err
                );
                return Ok(metric_ids);
            }
        };

        // Extract metric_ids from source_metrics
        for metric_id in source_metrics_data.source_metrics.keys() {
            metric_ids.push(metric_id.clone());
        }

        Ok(metric_ids)
    }
    // Get the most recent inflow of specific metric_id with some count
    pub async fn get_inflow_with_metric_id_and_count(
        &self,
        metric_id: String,
        count: usize,
    ) -> Result<Vec<SourceMetrics>> {
        let mut inflow: Vec<SourceMetrics> = Vec::new();

        // Acquire read lock on source metrics data
        let source_metrics_data = match SOURCE_METRICS_DATA.read() {
            Ok(data) => data,
            Err(err) => {
                eprintln!(
                    "Failed to acquire read lock on SOURCE_METRICS_DATA: {}",
                    err
                );
                return Ok(inflow);
            }
        };

        // Extract source metrics data with metric id
        if let Some(source_metrics_item) = source_metrics_data.source_metrics.get(&metric_id) {
            let recent = source_metrics_item.iter().rev().take(count);
            for (_key, value) in recent {
                inflow.push(SourceMetrics {
                    json_value: value.json_value.clone(),
                });
            }
        }
        debug!("inflow: {:?}", inflow);
        Ok(inflow)
    }
}

#[cfg(test)]
mod tests {
    use crate::data_layer::tests::{get_data_layer_with_postgres, get_data_layer_with_sqlite};

    use super::DataLayer;
    use super::*;
    use tracing_test::traced_test;
    use ulid::Ulid;

    const DEFAULT_ENABLE_METRICS_LOG: bool = false;

    #[tokio::test]
    async fn test_add_source_metrics_in_data_layer() {
        const DB_URL: &str = "sqlite://tests/temp/test.db";
        const METRIC_BUFFER_SIZE_KB: u64 = 1;
        let data_layer =
            DataLayer::new(DB_URL, METRIC_BUFFER_SIZE_KB, DEFAULT_ENABLE_METRICS_LOG).await;
        let collector = "vector";
        let metric_id = "metric_1";
        let json_value = r#"[{
            "name": "name",
            "tags": {
                "tag1": "value1"
            },
            "value": 2.0
        }]"#;
        for _idx in 1..10 {
            let _ = data_layer
                .add_source_metrics_in_data_layer(collector, metric_id, json_value)
                .await;
        }
        let mut total_source_metric_size = 0;
        SOURCE_METRICS_DATA
            .read()
            .unwrap()
            .source_metrics
            .iter()
            .for_each(|source_metric| {
                source_metric.1.iter().for_each(|(ulid, source_metrics)| {
                    total_source_metric_size += source_metrics.get_heap_size()
                        + (metric_id.get_heap_size() * 2)
                        + (ulid.get_heap_size() * 2);
                });
            });

        let measure_json_value_size = SourceMetrics {
            json_value: json_value.to_string(),
        }
        .get_heap_size();
        let measure_metric_id_size = metric_id.get_heap_size() * 2;
        let measure_ulid_size = Ulid::new().to_string().get_heap_size() * 2;
        let measure_size = measure_json_value_size + measure_metric_id_size + measure_ulid_size;
        assert!(total_source_metric_size < (METRIC_BUFFER_SIZE_KB * 1000) as usize); // source_metrics of size is less than METRIC_BUFFER_SIZE_KB
        assert!(total_source_metric_size % measure_size == 0); // source_metrics of size is multiple of measure_size
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

    async fn get_add_source_metrics_in_data_layer_size(_data_layer: &DataLayer) -> usize {
        let mut total_source_metrics_size = 0;
        for (metric_id, btree_map) in SOURCE_METRICS_DATA.read().unwrap().source_metrics.iter() {
            for (ulid, source_metrics) in btree_map.iter() {
                let source_metrics_size = source_metrics.get_heap_size()
                    + (metric_id.get_heap_size() * 2)
                    + (ulid.get_heap_size() * 2);
                total_source_metrics_size += source_metrics_size;
            }
        }
        total_source_metrics_size
    }

    async fn get_add_source_metrics_metadata_in_data_layer_size(_data_layer: &DataLayer) -> usize {
        let mut total_source_metrics_metadata_size = 0;
        SOURCE_METRICS_DATA
            .read()
            .unwrap()
            .source_metrics_metadata
            .iter()
            .for_each(|(_metric_id, _ulid, size)| {
                total_source_metrics_metadata_size += size;
            });
        total_source_metrics_metadata_size
    }

    #[tokio::test]
    async fn test_add_source_metrics_in_data_layer_check_save_data() {
        const DB_URL: &str = "sqlite://tests/temp/test.db";
        const METRIC_BUFFER_SIZE_KB: u64 = 1;
        let data_layer =
            DataLayer::new(DB_URL, METRIC_BUFFER_SIZE_KB, DEFAULT_ENABLE_METRICS_LOG).await;
        let ulid_size = Ulid::new().to_string().get_heap_size();

        // sample data 1
        let sample_1_metric_id = "sample_1".to_string();
        let sample_1_json_value =
            json!([{"name": "test", "tags": {"tag1": "value","tag2": "value","tag3": "value"}, "value": 1.0}]).to_string();
        let sample_1_total_size = add_source_metrics_in_data_layer_save_test_data(
            &data_layer,
            sample_1_metric_id,
            sample_1_json_value,
            ulid_size,
        )
        .await;

        // sample data 2
        let sample_2_metric_id = "sample_2".to_string();
        let sample_2_json_value =
            json!([{"name": "test", "tags": {"tag1": "value","tag2": "value","tag3": "value","tag4": "value","tag5": "value","tag6": "value","tag7": "value","tag8": "value","tag9": "value","tag10": "value","tag11": "value","tag12": "value","tag13": "value","tag14": "value","tag15": "value","tag16": "value","tag17": "value","tag18": "value","tag19": "value","tag20": "value","tag21": "value"}, "value": 1.0}]).to_string();
        let sample_2_total_size = add_source_metrics_in_data_layer_save_test_data(
            &data_layer,
            sample_2_metric_id,
            sample_2_json_value,
            ulid_size,
        )
        .await;

        // sample data 3
        let sample_3_metric_id = "sample_1".to_string();
        let sample_3_json_value =
            json!([{"name": "test", "tags": {"tag1": "value","tag2": "value","tag3": "value","tag4": "value","tag5": "value","tag6": "value","tag7": "value","tag8": "value","tag9": "value","tag10": "value"}, "value": 1.0}]).to_string();
        let sample_3_total_size = add_source_metrics_in_data_layer_save_test_data(
            &data_layer,
            sample_3_metric_id,
            sample_3_json_value,
            ulid_size,
        )
        .await;

        // check save data size = source_metrics
        let total_source_metrics_size =
            get_add_source_metrics_in_data_layer_size(&data_layer).await;

        // check save data size => source_metrics_metadata
        let total_source_metrics_metadata_size =
            get_add_source_metrics_metadata_in_data_layer_size(&data_layer).await;

        // sample data size(1 + 2 + 3) = total source metrics size
        assert!(
            (sample_1_total_size + sample_2_total_size + sample_3_total_size)
                == total_source_metrics_size
        );
        // sample data size(1 + 2 + 3) = total source metrics metadata size
        assert!(
            (sample_1_total_size + sample_2_total_size + sample_3_total_size)
                == total_source_metrics_metadata_size
        );

        // sample data 4
        let sample_4_metric_id = "sample_4".to_string();
        let sample_4_json_value =
            json!([{"name": "test", "tags": {"tag1": "value","tag2": "value","tag3": "value","tag4": "value","tag5": "value","tag6": "value","tag7": "value","tag8": "value","tag9": "value","tag10": "value","tag11": "value","tag12": "value","tag13": "value","tag14": "value","tag15": "value"}, "value": 1.0}]).to_string();
        let sample_4_total_size = add_source_metrics_in_data_layer_save_test_data(
            &data_layer,
            sample_4_metric_id,
            sample_4_json_value,
            ulid_size,
        )
        .await;
        // check save data size = source_metrics
        let total_source_metrics_size_2 =
            get_add_source_metrics_in_data_layer_size(&data_layer).await;

        // check save data size => source_metrics_metadata
        let total_source_metrics_metadata_size_2 =
            get_add_source_metrics_metadata_in_data_layer_size(&data_layer).await;

        // sample data size(3 + 4) = total source metrics size
        assert!((sample_3_total_size + sample_4_total_size) == total_source_metrics_size_2);
        // sample data size(3 + 4) = total source metrics metadata size
        assert!(
            (sample_3_total_size + sample_4_total_size) == total_source_metrics_metadata_size_2
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_source_metric_data_save_in_multi_thread() {
        let loop_cnt = 10;
        let mut hendle_vec = vec![];
        for idx in 0..loop_cnt {
            let hendle = tokio::spawn(async move {
                let json_value1 =
                    json!([{"name": format!("test_{}", idx), "value": 1.0}]).to_string();
                let mut source_metrics_data = SOURCE_METRICS_DATA.write().unwrap();
                let mut source_metrics_map = BTreeMap::new();
                source_metrics_map.insert(
                    Ulid::new().to_string(),
                    SourceMetrics {
                        json_value: json_value1,
                    },
                );
                source_metrics_data
                    .source_metrics
                    .insert(format!("metric_{}", idx), source_metrics_map);
                println!("idx : {}", idx);
            });
            hendle_vec.push(hendle);
        }
        for hendle in hendle_vec {
            if let Err(e) = hendle.await {
                println!("Error in handle: {:?}", e);
            }
        }
        let mut read_idx = 0;
        SOURCE_METRICS_DATA
            .read()
            .unwrap()
            .source_metrics
            .iter()
            .for_each(|(metric_id, map)| {
                println!(
                    "metric_id: {}, ulid: {}, name: {}",
                    metric_id,
                    map.first_key_value().unwrap().0,
                    map.first_key_value().unwrap().1.json_value
                );
                read_idx += 1;
            });
        assert_eq!(read_idx, loop_cnt);
    }

    #[tokio::test]
    #[traced_test]
    async fn test_get_source_metrics_values_all_metric_ids() {
        let data_layer = get_data_layer_with_sqlite().await;
        test_get_source_metrics_values_all_metric_ids_with_data_layer(data_layer).await;

        let data_layer = get_data_layer_with_postgres().await;
        test_get_source_metrics_values_all_metric_ids_with_data_layer(data_layer).await;
    }
    async fn test_get_source_metrics_values_all_metric_ids_with_data_layer(data_layer: DataLayer) {
        let json_value = r#"[{
            "name": "test",
            "tags": {
                "tag1": "value1"
            },
            "value": 2.0
        }]
        "#;

        // add a source metric
        let add_source_metric = data_layer
            .add_source_metric("vector", "source_metrics_test_1", json_value)
            .await;
        assert!(add_source_metric.is_ok());

        // read source metric
        let source_metrics = data_layer
            .get_source_metrics_values_all_metric_ids(10 * 1000)
            .await
            .unwrap();
        let source_metrics_filter_arr: Vec<&serde_json::Value> = source_metrics
            .iter()
            .filter(|value| value.get("metric_id").unwrap() == "source_metrics_test_1")
            .collect();
        assert!(!source_metrics_filter_arr.is_empty());
    }
}
