use vergen::{vergen, Config};

fn main() {
    tauri_build::build();
    vergen(Config::default()).unwrap();
    println!("cargo:rustc-link-lib=framework=ServiceManagement");
}
