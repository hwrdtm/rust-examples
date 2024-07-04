use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Gauge {
    value: Something,
}

#[derive(Debug, Serialize, Deserialize)]
struct Counter {
    cumulative: Something,
}

#[derive(Debug, Serialize, Deserialize)]
struct Sum {
    values: Vec<Something>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Something {
    property: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Data {
    Gauge(Gauge),
    Counter(Counter),
    Sum(Sum),
}

#[derive(Debug, Serialize, Deserialize)]
struct MetricContainer {
    #[serde(flatten)]
    data: Option<Data>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Metrics {
    metrics: Vec<MetricContainer>,
}

fn main() {
    // Read the local file sample.json as Metrics
    let json = std::fs::read_to_string("sample.json").unwrap();
    let metrics: Metrics = serde_json::from_str(&json).unwrap();
    println!("Serialized metrics: {:#?}", metrics);

    // Write the object back to a file called sample_serialized.json
    let serialized = serde_json::to_string_pretty(&metrics).unwrap();
    std::fs::write("sample_serialized.json", serialized).unwrap();

    // Now, do the same for sample_incorrect.json and sample_incorrect_serialized.json
    let json = std::fs::read_to_string("sample_incorrect.json").unwrap();
    let metrics: Metrics = serde_json::from_str(&json).unwrap();
    println!("Serialized metrics: {:#?}", metrics);
    let serialized = serde_json::to_string_pretty(&metrics).unwrap();
    std::fs::write("sample_incorrect_serialized.json", serialized).unwrap();
}
