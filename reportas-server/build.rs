fn main() {
    tonic_build::compile_protos("./src/proto/report1_0.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    tonic_build::compile_protos("./src/proto/report.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
