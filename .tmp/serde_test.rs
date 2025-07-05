use bingo_api::types::{EvaluateRequest, ResponseFormat};
use serde_json::json;
fn main() {
    let val = json!({"facts": [], "rules": [], "response_format": "json"});
    let req: EvaluateRequest = serde_json::from_value(val).unwrap();
    match req.response_format {
        Some(ResponseFormat::Standard) => println!("Standard"),
        Some(ResponseFormat::Stream) => println!("Stream"),
        Some(ResponseFormat::Auto) => println!("Auto"),
        None => println!("None"),
    }
}
