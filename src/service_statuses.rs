use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceStatuses {
    pub names: HashSet<String>,
}
