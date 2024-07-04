//! Structure that models the body of parameters of a POST request to create a pull request

use serde::{Deserialize, Serialize};

/// Models the body of a POST request
/// that asks to create a pull request
#[derive(Deserialize, Serialize, Debug)]
pub struct BodyParameters {
    pub head: String,
    pub base: String,
    pub title: String,
}
