//! Regenerates the checked-in gRPC and serde code in `sdk/src/generated`.
//!
//! Maintainer tool: requires `protoc` on PATH. SDK users never run this
//! because the generated code is committed.

use std::cell::RefCell;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use prost_build::{Service, ServiceGenerator};

const PROTO: &str = "sdk/proto/v1/run_function.proto";
const INCLUDE: &str = "sdk/proto";
const OUT: &str = "sdk/src/generated";

fn main() -> Result<(), Box<dyn Error>> {
    let root = workspace_root();
    let tmp = tempfile::tempdir()?;
    let descriptor = tmp.path().join("fileset.bin");

    // prost hands its service generator the file the messages go into. tonic's
    // output is routed to a buffer of its own instead, so that the gRPC client
    // and server land in a file the SDK includes only with its `server`
    // feature, while the messages stay feature-free.
    let grpc = Rc::new(RefCell::new(String::new()));
    let tonic = tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .service_generator();

    let mut config = prost_build::Config::new();
    config
        .file_descriptor_set_path(&descriptor)
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types")
        .service_generator(Box::new(Redirected {
            inner: tonic,
            out: Rc::clone(&grpc),
        }))
        .out_dir(tmp.path())
        .compile_protos(&[root.join(PROTO)], &[root.join(INCLUDE)])?;

    pbjson_build::Builder::new()
        .register_descriptors(&fs::read(&descriptor)?)?
        .out_dir(tmp.path())
        .build(&[".apiextensions"])?;

    let out = root.join(OUT);
    fs::create_dir_all(&out)?;
    for entry in fs::read_dir(tmp.path())? {
        let path = entry?.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let target = if name.ends_with(".serde.rs") {
            "v1.serde.rs"
        } else if name.ends_with(".rs") {
            "v1.rs"
        } else {
            continue;
        };
        fs::copy(&path, out.join(target))?;
        println!("wrote {}", out.join(target).display());
    }
    fs::write(out.join("v1.tonic.rs"), grpc.borrow().as_bytes())?;
    println!("wrote {}", out.join("v1.tonic.rs").display());
    fs::copy(&descriptor, out.join("fileset.bin"))?;
    println!("wrote {}", out.join("fileset.bin").display());
    Ok(())
}

/// A service generator that writes to its own buffer rather than to the
/// message file prost hands it.
struct Redirected {
    inner: Box<dyn ServiceGenerator>,
    out: Rc<RefCell<String>>,
}

impl ServiceGenerator for Redirected {
    fn generate(&mut self, service: Service, _message_file: &mut String) {
        self.inner.generate(service, &mut self.out.borrow_mut());
    }

    fn finalize(&mut self, _message_file: &mut String) {
        self.inner.finalize(&mut self.out.borrow_mut());
    }

    fn finalize_package(&mut self, package: &str, _message_file: &mut String) {
        self.inner
            .finalize_package(package, &mut self.out.borrow_mut());
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("codegen crate lives one level below the workspace root")
        .to_path_buf()
}
