fn main() {
    capnpc::CompilerCommand::new()
        .file("schema/sitegraph.capnp")
        .run()
        .expect("failed to compile sitegraph Cap'n Proto schema");
}
