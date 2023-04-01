use serde::{Deserialize, Serialize};
use serde_valid::Validate;

use super::{object_kind::ObjectKind, validate_id_regex};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SloDefinition {
    pub kind: ObjectKind,
    #[validate(custom(validate_id_regex))]
    #[validate(min_length = 2)]
    pub id: String,
}
