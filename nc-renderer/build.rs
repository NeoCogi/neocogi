use std::env;

extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks, StaticGenerator};
use std::fs::File;
use std::path::Path;


fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();

    Registry::new(Api::Gles2, (3, 0), Profile::Core, Fallbacks::All, [])
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();

    let target = env::var("TARGET");
    match target {
         Ok(s) if s.contains("wasm32") => (),
         _ => {
            println!("cargo:rustc-link-lib=glfw");  // the "-l" flag
            println!("cargo:rustc-link-lib=c");     // the "-l" flag
            println!("cargo:rustc-link-lib=GLESv2"); // the "-l" flag
         }
    }
}