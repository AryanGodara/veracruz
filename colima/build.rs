//! Colima build script
//!
//! ## Authors
//!
//! The Veracruz Development Team.
//!
//! ## Licensing and copyright notice
//!
//! See the `LICENSE.markdown` file in the Veracruz root directory for
//! information on licensing and copyright.

extern crate protoc_rust;

fn main() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/",
        input: &["protos/colima.proto"],
        includes: &["protos"],
        customize: protoc_rust::Customize {
            ..Default::default()
        },
    })
    .expect("protoc");
}
