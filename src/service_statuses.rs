use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

/// Service status used in the status server
#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceStatuses {
    pub names: HashSet<String>,
}
