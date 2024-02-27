use data_layer::data_layer::METRICS_DATA;
use serde_json::Value;
use std::ops::Bound::Included;
use std::time::Duration;
use std::{collections::HashMap, str::FromStr};
use tracing::{debug, error};
use ulid::Ulid;

/**
 * PlanExpressionStats
 */
#[derive(Debug, Clone)]
enum PlanExpressionStats {
    Latest,
    Average,
    Sum,
    Count,
    Minimum,
    Maximum,
    LinearSlope,
    MovingAverageSlope,
}
impl std::fmt::Display for PlanExpressionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PlanExpressionStats::Latest => write!(f, "latest"),
            PlanExpressionStats::Average => write!(f, "avg"),
            PlanExpressionStats::Sum => write!(f, "sum"),
            PlanExpressionStats::Count => write!(f, "count"),
            PlanExpressionStats::Minimum => write!(f, "min"),
            PlanExpressionStats::Maximum => write!(f, "max"),
            PlanExpressionStats::LinearSlope => write!(f, "linear_slope"),
            PlanExpressionStats::MovingAverageSlope => write!(f, "moving_average_slope"),
        }
    }
}

// Constants
const PLAN_EXPRESSION_PERIOD_SEC: u64 = 5 * 60;

pub fn get_in_js(args: rquickjs::Object<'_>) -> Result<f64, rquickjs::Error> {
    let metric_id = args
        .get::<String, String>("metric_id".to_string())
        .map_err(|_| {
            error!("[ScalingPlan expression error] Failed to get metric_id");
            rquickjs::Error::new_loading("Failed to get metric_id")
        })?;
    let name = args.get::<String, String>("name".to_string()).ok();
    // tags, stats, period_sec is optional
    let tags = match args.get::<String, HashMap<String, String>>("tags".to_string()) {
        Ok(tags) => tags,
        Err(_) => HashMap::new(),
    };
    let stats = args
        .get::<String, String>("stats".to_string())
        .unwrap_or("latest".to_string());
    let period_sec = args
        .get::<String, u64>("period_sec".to_string())
        .unwrap_or(PLAN_EXPRESSION_PERIOD_SEC); // default 5 min

    let Ok(metrics_data) = METRICS_DATA.read() else {
        error!("[get_in_js] Failed to get metrics_data");
        return Err(rquickjs::Error::new_loading("Failed to get the metrics data"));
    };
    let start_time = Ulid::from_datetime(
        std::time::SystemTime::now() - Duration::from_millis(1000 * period_sec),
    );
    let end_time = Ulid::new();

    debug!(
        "[get_in_js] - metric_id: {}, name: {:?}, tags: {:?}, stats: {}, period_sec: {}",
        metric_id, name, tags, stats, period_sec
    );

    // find metric_id
    let Some(metric_values) = metrics_data.metrics_data_map.get(&metric_id) else {
        return Err(rquickjs::Error::new_loading("Failed to get metric_id from the metrics data"));
    };

    // Filtered metric values
    let mut target_value_arr: Vec<f64> = Vec::new();

    // Validate whether the start_time is before the last item in the metric_values.
    // If the start_time is after the last item, then BTreeMap will panic.
    let last_item = metric_values.iter().last();
    if last_item.is_none() {
        return Err(rquickjs::Error::new_loading(
            "Failed to get the last item in the metric_values",
        ));
    }
    let last_item = last_item.unwrap();
    let last_item_time = Ulid::from_str(last_item.0.as_str()).unwrap();
    if last_item_time < start_time {
        return Err(rquickjs::Error::new_loading(
            "The start_time is before the last item in the metric_values",
        ));
    }

    // Find the metric values between the time range (current time - period_sec, current time)
    metric_values
        .range((Included(start_time.to_string()), Included(end_time.to_string())))
        .for_each(|(_ulid, metrics_data_item)| {
            // Get the json string
            let Ok(value) = serde_json::to_value(metrics_data_item.clone()) else {
                error!(
                    "[ScalingPlan expression error] Failed to convert metric_data_item to serde value"
                );
                return;
            };
            let Some(json_value_str) = value
                .get("json_value")
                .and_then(|value| value.as_str()) else {
                return;
            };
            // Transform the json string to serde value
            let json_value_str = serde_json
                ::from_str::<Value>(json_value_str)
                .map_err(|_| {
                    error!(
                        "[ScalingPlan expression error] Failed to convert json_value to serde value"
                    );
                    rquickjs::Error::Exception
                })
                .unwrap();

            // Get the value array
            let Some(json_values_arr) = json_value_str.as_array() else {
                return;
            };

            for json_value_item in json_values_arr.iter() {
                // Check if the name
                let item_name = json_value_item.get("name").and_then(Value::as_str);
                if
                    name.is_some() &&
                    item_name.is_some() &&
                    item_name.unwrap() != name.as_ref().unwrap()
                {
                    continue;
                }

                // Check if the tags match
                let item_tags = json_value_item.get("tags").and_then(Value::as_object);
                if !tags.is_empty() {
                    // If the tags are not empty but the item_tags is None, then it means that it doesn't match
                    if item_tags.is_none() {
                        continue;
                    }
                    let item_tags = item_tags.unwrap();

                    let mut match_tags = true;
                    for (key, value) in tags.iter() {
                        let item_value = item_tags.get(key).and_then(Value::as_str);
                        if item_value.is_none() || item_value.unwrap() != value {
                            match_tags = false;
                            break;
                        }
                    }

                    // If the tags don't match, then skip
                    if !match_tags {
                        continue;
                    }
                }

                // Put the value in the target_value_arr
                let item_value = json_value_item.get("value").and_then(Value::as_f64);
                if item_value.is_some() {
                    target_value_arr.append(&mut vec![item_value.unwrap()]);
                }
            }
        });
    let metric_stats =
        match stats.to_lowercase() {
            ms if PlanExpressionStats::Latest.to_string() == ms => {
                let Some(latest_value) = target_value_arr.iter().last() else {
                return Err(rquickjs::Error::new_loading("Failed to get the value with the stats"));
            };
                Ok(latest_value.to_owned())
            }
            ms if PlanExpressionStats::Average.to_string() == ms => {
                let sum_value: f64 = target_value_arr.iter().sum();
                let ms_num: f64 = sum_value / (target_value_arr.len() as f64);
                Ok(ms_num)
            }
            ms if PlanExpressionStats::Sum.to_string() == ms => {
                let sum_value: f64 = target_value_arr.iter().sum();
                Ok(sum_value)
            }
            ms if PlanExpressionStats::Count.to_string() == ms => Ok(target_value_arr.len() as f64),
            ms if PlanExpressionStats::Minimum.to_string() == ms => {
                let min_value = target_value_arr.into_iter().reduce(f64::min).ok_or(
                    rquickjs::Error::new_loading("Failed to get the value with the stats"),
                );
                match min_value {
                    Ok(min_value) => Ok(min_value),
                    Err(_) => Err(rquickjs::Error::new_loading(
                        "Failed to get the value with the stats",
                    )),
                }
            }
            ms if PlanExpressionStats::Maximum.to_string() == ms => {
                let max_value = target_value_arr.into_iter().reduce(f64::max).ok_or(
                    rquickjs::Error::new_loading("Failed to get the value with the stats"),
                );
                match max_value {
                    Ok(max_value) => Ok(max_value),
                    Err(_) => Err(rquickjs::Error::new_loading(
                        "Failed to get the value with the stats",
                    )),
                }
            }
            // LinearSlope (Simple Linear Regression)
            ms if PlanExpressionStats::LinearSlope.to_string() == ms => {
                calculate_slope(&target_value_arr)
                    .map_err(|_| rquickjs::Error::new_loading("Failed to calculate the slope"))
            }
            // MovingAverageSlope
            ms if PlanExpressionStats::MovingAverageSlope.to_string() == ms => {
                // Moving average
                let moving_average_window_size = 3;

                if target_value_arr.len() < moving_average_window_size {
                    return Err(rquickjs::Error::new_loading(
                        "The target_value_arr is less than the moving_average_window_size",
                    ));
                }

                let mut moving_average: Vec<f64> = Vec::new();

                for index in (moving_average_window_size - 1)..target_value_arr.len() {
                    let start_index = index - (moving_average_window_size - 1);
                    let end_index = index + 1;
                    let average: f64 = target_value_arr[start_index..end_index].iter().sum();
                    moving_average.append(&mut vec![average / moving_average_window_size as f64]);
                }

                calculate_slope(&moving_average)
                    .map_err(|_| rquickjs::Error::new_loading("Failed to calculate the slope"))
            }
            _ => {
                error!("[get_in_js] stats is valid: {}", stats);
                Err(rquickjs::Error::new_loading(
                    "Failed to get the value with the stats",
                ))
            }
        };
    debug!("[get_in_js] metric_stats: {:?}", metric_stats);
    metric_stats
}

/**
Calculate the slope
- x: index
- y: value
- n: length
- slope: (n * Σxy - Σx * Σy) / (n * Σx^2 - (Σx)^2)
 */
fn calculate_slope(values: &[f64]) -> Result<f64, rquickjs::Error> {
    let mut x: Vec<f64> = Vec::new();
    let mut y: Vec<f64> = Vec::new();
    for (index, value) in values.iter().enumerate() {
        x.append(&mut vec![(index + 1) as f64]);
        y.append(&mut vec![value.to_owned()]);
    }
    let x_sum: f64 = x.iter().sum();
    let y_sum: f64 = y.iter().sum();
    let x_y_sum: f64 = x.iter().zip(y.iter()).map(|(x, y)| x * y).sum();
    let x_square_sum: f64 = x.iter().map(|x| x * x).sum();
    let x_sum_square: f64 = x_sum * x_sum;
    let slope: f64 =
        (x.len() as f64 * x_y_sum - x_sum * y_sum) / (x.len() as f64 * x_square_sum - x_sum_square);
    Ok(slope)
}
