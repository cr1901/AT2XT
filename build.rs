use std::env;
use std::path::PathBuf;

use postcard_infomem_host::*;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let im = generate_from_env(EnvConfig::default()).unwrap();
    write_info_to_file(&im, out_dir.join("version.bin"), WriterConfig::default()).unwrap();

    let ld_cfg = BareSectionConfig::default()
        .set_max_size(Some(192));
    generate_infomem_ldscript(out_dir.join("info.x"), ld_cfg).unwrap();
    println!("cargo:rerun-if-changed=src");
}
