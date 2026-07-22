use std::{env, path::PathBuf};

use schema_rust::build::{CargoSchemaMetadata, GenerationDriver, GenerationPlan, ModuleEmission};

fn main() {
    SchemaBuild::from_environment().run();
}

struct SchemaBuild {
    crate_root: PathBuf,
}

impl SchemaBuild {
    fn from_environment() -> Self {
        Self {
            crate_root: PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir set")),
        }
    }

    fn run(&self) {
        println!("cargo:rerun-if-changed=schema/assembly.schema");
        println!("cargo:rerun-if-changed=src/schema/assembly.rs");

        let plan = GenerationPlan::new(&self.crate_root, "skills", "0.2.0")
            .with_module(ModuleEmission::declaration_module("assembly"));

        GenerationDriver::new(plan)
            .generate()
            .expect("generate skills schema artifacts")
            .write_or_check("SKILLS_UPDATE_SCHEMA_ARTIFACTS")
            .expect("checked-in skills schema artifacts are fresh");
        CargoSchemaMetadata::new("skills").emit_schema_directory(&self.crate_root);
    }
}
