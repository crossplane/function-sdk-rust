//! Regenerates the checked-in gRPC and serde code in `sdk/src/generated`.
//!
//! Maintainer tool: requires `protoc` on PATH. SDK users never run this
//! because the generated code is committed.

use std::error::Error;
use std::fs;
use std::path::PathBuf;

const PROTO: &str = "sdk/proto/v1/run_function.proto";
const INCLUDE: &str = "sdk/proto";
const OUT: &str = "sdk/src/generated";

/// Messages written by hand in `sdk/src/proto/state.rs` so that they keep
/// unknown fields, which prost-generated messages discard.
const HAND_WRITTEN: [&str; 2] = ["State", "Resource"];
const UNKNOWN_FIELDS_INIT: &str = "unknown_fields: Default::default(),";

fn main() -> Result<(), Box<dyn Error>> {
    let root = workspace_root();
    let tmp = tempfile::tempdir()?;
    let descriptor = tmp.path().join("fileset.bin");

    let mut config = prost_build::Config::new();
    config
        .file_descriptor_set_path(&descriptor)
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types");
    for message in HAND_WRITTEN {
        config.extern_path(
            format!(".apiextensions.fn.proto.v1.{message}"),
            format!("crate::proto::state::{message}"),
        );
    }

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir(tmp.path())
        .compile_with_config(config, &[root.join(PROTO)], &[root.join(INCLUDE)])?;

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
        if target == "v1.serde.rs" {
            let serde = init_unknown_fields(&fs::read_to_string(&path)?)?;
            fs::write(out.join(target), serde)?;
        } else {
            fs::copy(&path, out.join(target))?;
        }
        println!("wrote {}", out.join(target).display());
    }
    fs::copy(&descriptor, out.join("fileset.bin"))?;
    println!("wrote {}", out.join("fileset.bin").display());
    Ok(())
}

/// Makes the generated serde constructors of the hand-written messages
/// initialize `unknown_fields`, a field the descriptor does not have.
fn init_unknown_fields(serde: &str) -> Result<String, Box<dyn Error>> {
    let mut patched = serde.to_owned();
    for message in HAND_WRITTEN {
        let constructor = format!("Ok({message} {{\n");
        if patched.matches(&constructor).count() != 1 {
            return Err(format!("expected one serde constructor for {message}").into());
        }
        let start = patched.find(&constructor).expect("counted above");
        let end = start
            + patched[start..]
                .find("})")
                .ok_or_else(|| format!("unterminated serde constructor for {message}"))?;
        let line_start = patched[..end].rfind('\n').map_or(0, |i| i + 1);
        let indent = &patched[line_start..end];
        let init = format!("{indent}    {UNKNOWN_FIELDS_INIT}\n");
        patched.insert_str(line_start, &init);
    }
    Ok(patched)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("codegen crate lives one level below the workspace root")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENERATED: &str = "\
        Ok(Resource {
            ready: ready__.unwrap_or_default(),
        })
        Ok(State {
            composite: composite__,
        })
";

    #[test]
    fn init_unknown_fields_extends_both_constructors() {
        let want = "\
        Ok(Resource {
            ready: ready__.unwrap_or_default(),
            unknown_fields: Default::default(),
        })
        Ok(State {
            composite: composite__,
            unknown_fields: Default::default(),
        })
";
        assert_eq!(init_unknown_fields(GENERATED).expect("patches"), want);
    }

    #[test]
    fn init_unknown_fields_fails_without_a_constructor() {
        assert!(init_unknown_fields("Ok(State {\n})").is_err());
    }
}
