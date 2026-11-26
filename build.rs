fn main() {
    println!("cargo:rerun-if-changed=images/dubPlanetar.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("images/dubPlanetar.ico");
        res.compile().expect("l'icône du programme");
    }
}
