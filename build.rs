fn main() {
    tonic_build::configure()
        //        .type_attribute("brauanlage.RcpStep", "#[derive(Hash, Clone)]")
        //        .type_attribute("brauanlage.TempStatus", "#[derive(Hash)]")
        //        .type_attribute("brauanlage.RelayStatus", "#[derive(Hash)]")
        .build_server(true)
        .out_dir("./src")
        .compile(&["proto/brauanlage.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

    //    tonic_build::compile_protos("proto/brauanlage.proto")
    //        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
