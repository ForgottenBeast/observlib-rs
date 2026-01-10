use observlib::KeyValue;
use opentelemetry::global;

#[test]
pub fn basic_instantiation() {
    let attrs = vec![KeyValue::new("env", "dev")];
    let otel_manager = observlib::initialize_telemetry("blah", "127.0.0.1:4318", attrs);
    let counter = global::meter("my meter").u64_counter("my_counter").build();
    counter.add(1,&[]);
    otel_manager.shutdown().unwrap();
}
