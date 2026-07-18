fn main() {
    prost_build::compile_protos(&["proto/nova_bus.proto"], &["proto/"])
        .expect("failed to compile nova_bus.proto — is protoc on PATH or PROTOC set?");
}
