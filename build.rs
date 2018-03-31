extern crate bindgen;
extern crate cc;
extern crate walkdir;

use walkdir::{WalkDir};

use std::env;
use std::mem::{size_of};
use std::os::raw::{c_short, c_int, c_long, c_longlong};
use std::path::{PathBuf};

fn main() {
  let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  println!("cargo:rustc-link-search=native={}", out_dir.display());
  println!("cargo:rustc-link-lib=static=pillowimage_native");
  println!("cargo:rerun-if-changed=build.rs");

  //let imaging_src_dir = manifest_dir.join("pillow").join("src").join("libImaging");
  let imaging_src_dir = manifest_dir.join("pillow-simd").join("libImaging");
  let mut imaging_src_paths = vec![];
  for entry in WalkDir::new(imaging_src_dir.to_str().unwrap()) {
    let entry = entry.unwrap();
    match entry.path().extension() {
      None => continue,
      Some(ext) => if ext.to_str().unwrap() == "c" {
        if entry.path().file_name().unwrap().to_str().unwrap() == "codec_fd.c" {
          continue;
        }
        if entry.path().file_name().unwrap().to_str().unwrap().contains("Decode") {
          continue;
        }
        if entry.path().file_name().unwrap().to_str().unwrap().contains("Encode") {
          continue;
        }
        if entry.path().file_name().unwrap().to_str().unwrap().contains("FilterSIMD") {
          continue;
        }
        if entry.path().file_name().unwrap().to_str().unwrap().contains("ResampleSIMD") {
          continue;
        }
        println!("cargo:rerun-if-changed={}", entry.path().display());
        imaging_src_paths.push(entry.path().as_os_str().to_str().unwrap().to_owned());
      } else if ext.to_str().unwrap() == "h" {
        println!("cargo:rerun-if-changed={}", entry.path().display());
      } else {
        continue;
      }
    }
  }

  let sz_c_short = size_of::<c_short>();
  let sz_c_int = size_of::<c_int>();
  let sz_c_long = size_of::<c_long>();
  let sz_c_longlong = size_of::<c_longlong>();

  let mut compiler = cc::Build::new();
  compiler
    .opt_level(2)
    .pic(true)
    .flag("-g")
    //.flag("-fwrapv")
    //.flag("-fstack-protector-strong")
    .flag("-fno-strict-aliasing")
    .flag("-msse4.1")
    .flag("-pthread")
    //.flag("-D_FORTIFY_SOURCE=2")
    .flag("-DPILLOW_VERSION='5.1.0.dev0'")
    .flag("-DPILLOW_DISABLE_PYTHON=1")

    // FIXME: these macros are normally provided by cpython?
    .flag("-DHAVE_PROTOTYPES")
    .flag("-DSTDC_HEADERS")
    .flag("-DNDEBUG")
    .flag(&format!("-DSIZEOF_SHORT={}", sz_c_short))
    .flag(&format!("-DSIZEOF_INT={}", sz_c_int))
    .flag(&format!("-DSIZEOF_LONG={}", sz_c_long))
    .flag(&format!("-DSIZEOF_LONG_LONG={}", sz_c_longlong))

    .flag("-Wall")
    .flag("-Werror")
    .flag("-Wno-sign-compare")
    .flag("-Wno-unused-parameter")
    //.flag("-Wformat")
    //.flag("-Wstrict-prototypes")
    //.flag("-Werror=format-security")
    .include(imaging_src_dir.as_os_str().to_str().unwrap())
    //.include("/usr/include")
  ;
  for path in imaging_src_paths.iter() {
    compiler.file(path);
  }
  compiler.compile("libpillowimage_native.a");

  bindgen::Builder::default()
    .header("wrapped.h")
    .clang_arg("-DPILLOW_DISABLE_PYTHON")
    .clang_arg("-DHAVE_PROTOTYPES")
    .clang_arg("-DSTDC_HEADERS")
    .clang_arg("-DNDEBUG")
    .clang_arg(format!("-DSIZEOF_SHORT={}", sz_c_short))
    .clang_arg(format!("-DSIZEOF_INT={}", sz_c_int))
    .clang_arg(format!("-DSIZEOF_LONG={}", sz_c_long))
    .clang_arg(format!("-DSIZEOF_LONG_LONG={}", sz_c_longlong))
    .clang_arg(format!("-I{}", imaging_src_dir.as_os_str().to_str().unwrap()))
    //.clang_arg("-I/usr/include")
    .whitelist_type("Imaging")
    //.whitelist_type("ImagingAccess")
    //.whitelist_type("ImagingCodecState")
    .whitelist_type("ImagingSectionCookie")
    .whitelist_type("ImagingShuffle")
    .whitelist_function("ImagingSectionEnter")
    .whitelist_function("ImagingSectionLeave")
    .whitelist_function("ImagingNew")
    .whitelist_function("ImagingNew2")
    .whitelist_function("ImagingDelete")
    //.whitelist_function("ImagingAccessNew")
    //.whitelist_function("_ImagingAccessDelete")
    .whitelist_function("ImagingFlipLeftRight")
    .whitelist_function("ImagingFlipTopBottom")
    .whitelist_function("ImagingRotate90")
    .whitelist_function("ImagingRotate180")
    .whitelist_function("ImagingRotate270")
    .whitelist_function("ImagingResample")
    .whitelist_function("ImagingTransform")
    .whitelist_var("IMAGING_TYPE_UINT8")
    .whitelist_var("IMAGING_TYPE_INT32")
    .whitelist_var("IMAGING_TYPE_FLOAT32")
    .whitelist_var("IMAGING_TYPE_SPECIAL")
    .whitelist_var("IMAGING_TRANSFORM_NEAREST")
    .whitelist_var("IMAGING_TRANSFORM_BOX")
    .whitelist_var("IMAGING_TRANSFORM_BILINEAR")
    .whitelist_var("IMAGING_TRANSFORM_HAMMING")
    .whitelist_var("IMAGING_TRANSFORM_BICUBIC")
    .whitelist_var("IMAGING_TRANSFORM_LANCZOS")
    .generate()
    .unwrap()
    .write_to_file(out_dir.join("imaging_bind.rs"))
    .unwrap();
}
