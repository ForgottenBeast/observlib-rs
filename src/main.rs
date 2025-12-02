use opentelemetry::KeyValue;
pub fn main(){
    let attrs = vec![KeyValue::new("env","dev")];
    let otel_manager = observlib::initialize_telemetry("blah","127.0.0.1:4318",attrs);
    otel_manager.shutdown().unwrap();
}
