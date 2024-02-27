pub mod autoscaling_history_definition;
pub mod metric;
pub mod metric_definition;
pub mod metrics_data_item;
pub mod object_kind;
pub mod plan_item_definition;
pub mod scaling_component;
pub mod scaling_component_definition;
pub mod scaling_plan_definition;
use lazy_static::lazy_static;

lazy_static! {
    static ref VALIDATE_ID_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
}

pub fn validate_id_regex(id: &str) -> Result<(), serde_valid::validation::Error> {
    if VALIDATE_ID_REGEX.is_match(id) {
        Ok(())
    } else {
        Err(serde_valid::validation::Error::Custom(
            "Only alphanumeric and underscores are allowed".to_owned(),
        ))
    }
}
