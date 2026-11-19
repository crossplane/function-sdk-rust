//! Regenerates the checked-in gRPC and serde code in `sdk/src/generated`.
//!
//! Maintainer tool: requires `protoc` on PATH. SDK users never run this
//! because the generated code is committed.
//!
//! prost-build, tonic and pbjson generate the code. `messages` then rewrites
//! the messages so that they keep unknown fields, and what prost wrote for
//! them is kept beside the tests as the reference the rewrite is held to.

mod messages;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const PROTO: &str = "sdk/proto/v1/run_function.proto";
const INCLUDE: &str = "sdk/proto";
const OUT: &str = "sdk/src/generated";
/// The messages as prost-build derives them, which discard unknown fields:
/// what `sdk/tests/generated_messages.rs` compares the rewritten ones with.
const REFERENCE: &str = "sdk/tests/generated/prost_v1.rs";
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

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir(tmp.path())
        .compile_with_config(config, &[root.join(PROTO)], &[root.join(INCLUDE)])?;

    pbjson_build::Builder::new()
        .register_descriptors(&fs::read(&descriptor)?)?
        .out_dir(tmp.path())
        .build(&[".apiextensions"])?;

    // prost-build names its file after the proto package, a keyword in it
    // escaped, and pbjson adds `.serde` to that name.
    let (mut prost, mut serde) = (None, None);
    for entry in fs::read_dir(tmp.path())? {
        let path = entry?.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if name.ends_with(".serde.rs") {
            serde = Some(fs::read_to_string(&path)?);
        } else if name.ends_with(".rs") {
            prost = Some(fs::read_to_string(&path)?);
        }
    }
    let prost = prost.ok_or("prost-build wrote no .rs file")?;
    let serde = serde.ok_or("pbjson-build wrote no .serde.rs file")?;

    let (header, types, services) = split(&prost);
    let rewritten = messages::rewrite(types)?;

    let out = root.join(OUT);
    fs::create_dir_all(&out)?;
    write(
        &out.join("v1.rs"),
        &format!("{header}{}{services}", rewritten.source),
    )?;
    write(
        &out.join("v1.serde.rs"),
        &init_unknown_fields(&serde, &rewritten.messages)?,
    )?;
    let reference = root.join(REFERENCE);
    fs::create_dir_all(reference.parent().expect("the reference is in a directory"))?;
    write(&reference, &format!("{header}{types}"))?;
    fs::copy(&descriptor, out.join("fileset.bin"))?;
    println!("wrote {}", out.join("fileset.bin").display());
    Ok(())
}

fn write(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    fs::write(path, content)?;
    println!("wrote {}", path.display());
    Ok(())
}

/// Splits what prost-build wrote into its leading comment, the messages and
/// enumerations, and the gRPC client and server modules tonic appends. Only
/// the middle part is rewritten: a parser keeps no plain comment, and the
/// service code has no message in it.
fn split(prost: &str) -> (&str, &str, &str) {
    let header: usize = prost
        .split_inclusive('\n')
        .take_while(|line| line.starts_with("//") && !line.starts_with("///"))
        .map(str::len)
        .sum();
    let mut services = prost.len();
    let mut offset = 0;
    let mut docs = None;
    for line in prost.split_inclusive('\n') {
        if line.starts_with("///") {
            docs.get_or_insert(offset);
        } else {
            let service = line.starts_with("pub mod ")
                && (line.trim_end().ends_with("_client {")
                    || line.trim_end().ends_with("_server {"));
            if service {
                services = docs.unwrap_or(offset);
                break;
            }
            docs = None;
        }
        offset += line.len();
    }
    (
        &prost[..header],
        &prost[header..services],
        &prost[services..],
    )
}

/// Makes the generated serde constructor of every message initialize
/// `unknown_fields`, a field the descriptor does not have.
fn init_unknown_fields(serde: &str, messages: &[String]) -> Result<String, Box<dyn Error>> {
    let mut patched = serde.to_owned();
    for message in messages {
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

    fn both() -> Vec<String> {
        vec!["State".to_string(), "Resource".to_string()]
    }

    #[test]
    fn split_cuts_at_the_comment_and_at_the_services() {
        let prost = "// This file is @generated by prost-build.\n/// A message.\npub struct M {}\n/// Generated client implementations.\npub mod m_client {\n}\n/// Generated server implementations.\npub mod m_server {\n}\n";
        let (header, types, services) = split(prost);
        assert_eq!(header, "// This file is @generated by prost-build.\n");
        assert_eq!(types, "/// A message.\npub struct M {}\n");
        assert!(services.starts_with("/// Generated client implementations.\npub mod m_client {"));
        assert_eq!(split("pub struct M {}\n"), ("", "pub struct M {}\n", ""));
    }

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
        assert_eq!(
            init_unknown_fields(GENERATED, &both()).expect("patches"),
            want
        );
    }

    #[test]
    fn init_unknown_fields_fails_without_a_constructor() {
        assert!(init_unknown_fields("Ok(State {\n})", &both()).is_err());
    }
}
