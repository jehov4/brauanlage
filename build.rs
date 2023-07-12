fn main() {
    tonic_build::configure()
        //        .type_attribute("brauanlage.RcpStep", "#[derive(Hash, Clone)]")
        //        .type_attribute("brauanlage.TempStatus", "#[derive(Hash)]")
        //        .type_attribute("brauanlage.RelayStatus", "#[derive(Hash)]")
        .compile(&["proto/brauanlage.proto"], &["proto"])
        .unwrap();

    //    tonic_build::compile_protos("proto/brauanlage.proto")
    //        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
