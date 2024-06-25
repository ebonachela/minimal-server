#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Endpoint {
  pub file: String,
  pub path: String,
  pub method: String,
  pub content: Vec<String>,
}