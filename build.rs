// build.rs
fn main() {
    // Define the path to the directory with .proto files
    tonic_build::configure()
        .out_dir("src/proto")
        .compile_protos(&["proto/txproc.proto"], &["proto"])
        .unwrap();
}
