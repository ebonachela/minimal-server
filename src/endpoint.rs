#[derive(Clone, Debug)]
pub struct Endpoint {
  pub file: String,
  pub path: String,
  pub method: String,
  pub content: Vec<String>,
}