use super::DataLayer;
use crate::types::metrics_data_item::MetricsDataItem;
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
    pub async fn add_metrics_data(
        &self,
        collector: &str,
        metric_id: &str,
        json_value: &str,
    ) -> Result<()> {
        /* [ Comment ]
         *  metrics data: Metric data is separated by metric_id and ulid is sorted in ascending order (using for ScalingPlan search)
         *  metrics data metadata: Metric data is sorted in ASC order by ULID (using for remove target data to maintain buffer size)
         * [ Data structure ]
         *  metrics data - HashMap<key: metric_id, value: BTreeMap<key: ULID, value: SourceMetrics>>
         *  metrics data metadata - LinkedList<(metric_id, ULID, data size(metrics data + metrics data metadata)> - list order by ULID ASC */

        // After dropping the lock, check whether saving to the database is enabled
        // Add '_' to the variable name to suppress the warning
        let mut _enable_metrics_log = false;

        // Acquire write lock on metrics data data but do not hold the lock while 'await'
        // https://rust-lang.github.io/rust-clippy/master/index.html#/await_holding_lock
        {
            let Ok(mut metrics_data) = self.metrics_data.write() else {
                error!("[add_metrics_data] Failed to get metrics data data");
                return Err(anyhow!("Failed to get metrics data data"));
            };

            let now_ulid = Ulid::new().to_string();
            let metrics_to_be_added = MetricsDataItem {
                json_value: json_value.to_string(),
            };

            let size_of_metrics_to_be_added = metrics_to_be_added.get_heap_size()
                + (metric_id.to_string().get_heap_size() * 2)
                + (now_ulid.get_heap_size() * 2);

            let mut metrics_data_total_size =
                metrics_data.metrics_data_total_size + size_of_metrics_to_be_added;

            // get remove target data
            let mut remove_target_data: Vec<(String, String, usize)> = Vec::new();
            let mut subtract_total_size = metrics_data_total_size;
            loop {
                // check size :: buffersize - total size < 0 (None) => remove target data
                if metrics_data
                    .metric_buffer_size_byte
                    .checked_sub(subtract_total_size as u64)
                    .is_some()
                {
                    break;
                }
                let Some(front_metrics_metadata) = metrics_data.metrics_data_metadata.pop_front() else {
                break;
            };
                subtract_total_size -= front_metrics_metadata.2; // oldest metadata size
                remove_target_data.append(&mut vec![front_metrics_metadata]);
            }

            // remove target data
            remove_target_data
            .iter()
            .for_each(|(metric_id, ulid, size)| {
                let Some(metrics_data_item_map) = metrics_data.metrics_data_map.get_mut(metric_id) else {
                    return;
                };
                // remove metrics data - btreemap ulid
                metrics_data_item_map.remove(ulid);
                // (if btreemap is empty) remove metrics data - hashmap metric_id
                if metrics_data_item_map.is_empty() {
                    metrics_data.metrics_data_map.remove(metric_id);
                }
                metrics_data_total_size -= size;
            });

            // add metrics data
            match metrics_data.metrics_data_map.get_mut(metric_id) {
                Some(metrics_data_item_map) => {
                    metrics_data_item_map.insert(now_ulid.clone(), metrics_to_be_added);
                }
                None => {
                    let mut metrics_data_item_map = BTreeMap::new();
                    metrics_data_item_map.insert(now_ulid.clone(), metrics_to_be_added);
                    metrics_data
                        .metrics_data_map
                        .insert(metric_id.to_string(), metrics_data_item_map);
                }
            }
            // add metrics data metadata
            metrics_data.metrics_data_metadata.push_back((
                metric_id.to_string(),
                now_ulid,
                size_of_metrics_to_be_added,
            ));

            debug!(
                "[metrics data] add item size: {} / total size: {}",
                size_of_metrics_to_be_added, metrics_data_total_size
            );

            // update metrics_data_total_size
            metrics_data.metrics_data_total_size = metrics_data_total_size;

            _enable_metrics_log = metrics_data.enable_metrics_log;
        }

        // Save to database
        if _enable_metrics_log {
            let result_save_db = self
                .add_metrics_data_into_db(collector, metric_id, json_value)
                .await;
            if result_save_db.is_err() {
                error!("Failed to save metric into the DB: {:?}", result_save_db);
            }
        }
        Ok(())
    }

    /**
    Get the stats of the metrics data
    It returns metric id(key), the latest timestamps within (duration) seconds, the last value of the metrics data
    */
    pub async fn get_metrics_data_stats(
        &self,
        duration: u64,
    ) -> Result<HashMap<String, (Vec<String>, String)>> {
        let metrics_data = self.metrics_data.read().unwrap();
        let mut metrics_data_stats: HashMap<String, (Vec<String>, String)> = HashMap::new();
        metrics_data
            .metrics_data_map
            .iter()
            .for_each(|(metric_id, metrics_data_item_map)| {
                let mut latest_timestamps: Vec<String> = Vec::new();
                let mut last_value = String::new();
                if !metrics_data_item_map.is_empty() {
                    // Get the latest timestamps within 5 minutes
                    let minutes_ago = SystemTime::now() - Duration::from_secs(duration);
                    for (ulid, _) in metrics_data_item_map.iter().rev() {
                        let timestamp_ms = Ulid::from_string(ulid);
                        if timestamp_ms.is_err() {
                            break;
                        }
                        if timestamp_ms.unwrap().timestamp_ms()
                            < minutes_ago
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64
                        {
                            break;
                        }
                        latest_timestamps.push(timestamp_ms.unwrap().timestamp_ms().to_string());
                    }
                    // for (count, (ulid, _)) in metrics_data_item_map.iter().rev().enumerate() {
                    //     if count > 10 {
                    //         break;
                    //     }
                    //     let timestamp_ms = Ulid::from_string(ulid);
                    //     if timestamp_ms.is_err() {
                    //         break;
                    //     }
                    //     latest_timestamps.push(timestamp_ms.unwrap().timestamp_ms().to_string());
                    // }
                    let (_, metrics_data_item) = metrics_data_item_map.iter().last().unwrap();
                    last_value = metrics_data_item.json_value.to_string();
                }
                metrics_data_stats.insert(metric_id.to_string(), (latest_timestamps, last_value));
            });
        Ok(metrics_data_stats)
    }

    pub async fn get_metrics_data_by_metric_id(
        &self,
        metric_id: String,
        from: u64,
        to: u64,
    ) -> Result<BTreeMap<String, serde_json::Value>> {
        let metrics_data = self.metrics_data.read().unwrap();
        // Use BTreeMap to sort the keys
        let mut metric_values: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        if let Some(metrics_data_item_map) = metrics_data.metrics_data_map.get(&metric_id) {
            for (ulid, metrics_data_item) in metrics_data_item_map.iter().rev() {
                let timestamp_ms = Ulid::from_string(ulid);
                if timestamp_ms.is_err() {
                    break;
                }
                let timestamp_ms = timestamp_ms.unwrap().timestamp_ms();
                // iter returns the data in descending order
                if timestamp_ms > to {
                    continue;
                }
                if timestamp_ms < from {
                    break;
                }
                let json_value = serde_json::from_str(&metrics_data_item.json_value);
                if json_value.is_err() {
                    break;
                }
                let timestamp_ms = timestamp_ms.to_string();
                metric_values.insert(timestamp_ms, json_value.unwrap());
            }
        }
        Ok(metric_values)
    }

    // Add a SourceMetric to the database
    pub async fn add_metrics_data_into_db(
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
            "INSERT INTO metrics_data (id, collector, metric_id, json_value) VALUES ($1,$2,$3,$4)";
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
        debug!("Added a metrics data: {}", metric_id);
        Ok(())
    }
    // Get a latest metrics by collector and metric_id from the database
    pub async fn get_metrics_data_from_db(
        &self,
        metric_ids: Vec<String>,
        time_greater_than: u64,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Subtract the duration from the current time
        //
        let offset_time = SystemTime::now() - Duration::from_millis(time_greater_than);
        let ulid = Ulid::from_datetime(offset_time);
        let query_string =
            "SELECT metric_id, id, json_value FROM metrics_data WHERE id >= $1 and metric_id in ($2) ORDER BY id DESC LIMIT 1";
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
}

#[cfg(test)]
mod tests {
    use crate::data_layer::METRICS_DATA;

    use super::DataLayer;
    use super::*;
    use ulid::Ulid;

    const DEFAULT_ENABLE_METRICS_LOG: bool = false;

    #[tokio::test]
    async fn test_add_metrics_data() {
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
                .add_metrics_data(collector, metric_id, json_value)
                .await;
        }
        let mut total_metrics_data_size = 0;
        METRICS_DATA
            .read()
            .unwrap()
            .metrics_data_map
            .iter()
            .for_each(|metrics_data_item_map| {
                metrics_data_item_map
                    .1
                    .iter()
                    .for_each(|(ulid, metrics_data_item)| {
                        total_metrics_data_size += metrics_data_item.get_heap_size()
                            + (metric_id.get_heap_size() * 2)
                            + (ulid.get_heap_size() * 2);
                    });
            });

        let measure_json_value_size = MetricsDataItem {
            json_value: json_value.to_string(),
        }
        .get_heap_size();
        let measure_metric_id_size = metric_id.get_heap_size() * 2;
        let measure_ulid_size = Ulid::new().to_string().get_heap_size() * 2;
        let measure_size = measure_json_value_size + measure_metric_id_size + measure_ulid_size;
        assert!(total_metrics_data_size < (METRIC_BUFFER_SIZE_KB * 1000) as usize); // metrics_data of size is less than METRIC_BUFFER_SIZE_KB
        assert!(total_metrics_data_size % measure_size == 0); // metrics_data of size is multiple of measure_size
    }

    async fn add_sample_metrics_data(
        data_layer: &DataLayer,
        metric_id: String,
        json_value: String,
        ulid_size: usize,
    ) -> usize {
        // sample data 1
        let metric_id_size = metric_id.get_heap_size();
        let json_value_data = MetricsDataItem {
            json_value: json_value.to_string(),
        };
        let sample_data_total_size =
            json_value_data.get_heap_size() + (metric_id_size * 2) + (ulid_size * 2);

        // save sample data 1
        let _ = data_layer
            .add_metrics_data("vector", &metric_id, &json_value)
            .await;
        sample_data_total_size
    }

    async fn get_metrics_data_size(_data_layer: &DataLayer) -> usize {
        let mut total_metrics_data_size = 0;
        for (metric_id, btree_map) in METRICS_DATA.read().unwrap().metrics_data_map.iter() {
            for (ulid, metrics_data_item) in btree_map.iter() {
                let metrics_data_item_size = metrics_data_item.get_heap_size()
                    + (metric_id.get_heap_size() * 2)
                    + (ulid.get_heap_size() * 2);
                total_metrics_data_size += metrics_data_item_size;
            }
        }
        total_metrics_data_size
    }

    async fn get_metrics_data_metadata_size(_data_layer: &DataLayer) -> usize {
        let mut total_metrics_data_metadata_size = 0;
        METRICS_DATA
            .read()
            .unwrap()
            .metrics_data_metadata
            .iter()
            .for_each(|(_metric_id, _ulid, size)| {
                total_metrics_data_metadata_size += size;
            });
        total_metrics_data_metadata_size
    }

    #[tokio::test]
    async fn test_add_metrics_data_with_multiple_data() {
        const DB_URL: &str = "sqlite://tests/temp/test.db";
        const METRIC_BUFFER_SIZE_KB: u64 = 1;
        let data_layer =
            DataLayer::new(DB_URL, METRIC_BUFFER_SIZE_KB, DEFAULT_ENABLE_METRICS_LOG).await;
        let ulid_size = Ulid::new().to_string().get_heap_size();

        // sample data 1
        let sample_1_metric_id = "sample_1".to_string();
        let sample_1_json_value =
            json!([{"name": "test", "tags": {"tag1": "value","tag2": "value","tag3": "value"}, "value": 1.0}]).to_string();
        let sample_1_total_size = add_sample_metrics_data(
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
        let sample_2_total_size = add_sample_metrics_data(
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
        let sample_3_total_size = add_sample_metrics_data(
            &data_layer,
            sample_3_metric_id,
            sample_3_json_value,
            ulid_size,
        )
        .await;

        // check save data size = metrics_data
        let total_metrics_data_size = get_metrics_data_size(&data_layer).await;

        // check save data size => metrics_data_metadata
        let total_metrics_data_metadata_size = get_metrics_data_metadata_size(&data_layer).await;

        // sample data size(1 + 2 + 3) = total metrics data size
        assert!(
            (sample_1_total_size + sample_2_total_size + sample_3_total_size)
                == total_metrics_data_size
        );
        // sample data size(1 + 2 + 3) = total metrics data metadata size
        assert!(
            (sample_1_total_size + sample_2_total_size + sample_3_total_size)
                == total_metrics_data_metadata_size
        );

        // sample data 4
        let sample_4_metric_id = "sample_4".to_string();
        let sample_4_json_value =
            json!([{"name": "test", "tags": {"tag1": "value","tag2": "value","tag3": "value","tag4": "value","tag5": "value","tag6": "value","tag7": "value","tag8": "value","tag9": "value","tag10": "value","tag11": "value","tag12": "value","tag13": "value","tag14": "value","tag15": "value"}, "value": 1.0}]).to_string();
        let sample_4_total_size = add_sample_metrics_data(
            &data_layer,
            sample_4_metric_id,
            sample_4_json_value,
            ulid_size,
        )
        .await;
        // check save data size = metrics_data
        let total_metrics_data_size_2 = get_metrics_data_size(&data_layer).await;

        // check save data size => metrics_data_metadata
        let total_metrics_data_metadata_size_2 = get_metrics_data_metadata_size(&data_layer).await;

        // sample data size(3 + 4) = total metrics data size
        assert!((sample_3_total_size + sample_4_total_size) == total_metrics_data_size_2);
        // sample data size(3 + 4) = total metrics data metadata size
        assert!((sample_3_total_size + sample_4_total_size) == total_metrics_data_metadata_size_2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_add_metrics_data_in_multi_thread() {
        let loop_cnt = 10;
        let mut hendle_vec = vec![];
        for idx in 0..loop_cnt {
            let hendle = tokio::spawn(async move {
                let json_value1 =
                    json!([{"name": format!("test_{}", idx), "value": 1.0}]).to_string();
                let mut metrics_data_data = METRICS_DATA.write().unwrap();
                let mut metrics_data_map = BTreeMap::new();
                metrics_data_map.insert(
                    Ulid::new().to_string(),
                    MetricsDataItem {
                        json_value: json_value1,
                    },
                );
                metrics_data_data
                    .metrics_data_map
                    .insert(format!("metric_{}", idx), metrics_data_map);
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
        METRICS_DATA
            .read()
            .unwrap()
            .metrics_data_map
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
}
