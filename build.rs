fn main() {
    println!("cargo:rerun-if-changed=images/dubPlanetar.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ico = std::path::Path::new(&manifest)
        .join("images")
        .join("dubPlanetar.ico");
    let ico = ico.to_string_lossy().replace('\\', "/");
    let rc = out.join("resource.rc");
    std::fs::write(&rc, format!("1 ICON \"{ico}\"\n")).expect("écriture du fichier de ressources");

    let obj = out.join("resource.o");
    let windres = if cfg!(windows) {
        "windres.exe"
    } else {
        "windres"
    };
    let status = std::process::Command::new(windres)
        .arg(&rc)
        .arg(&obj)
        .status()
        .expect("windres est introuvable");
    if !status.success() {
        panic!("windres n'a pas pu compiler l'icône");
    }

    let lib = out.join("libresource.a");
    let ar = if cfg!(windows) { "ar.exe" } else { "ar" };
    let status = std::process::Command::new(ar)
        .arg("rcs")
        .arg(&lib)
        .arg(&obj)
        .status()
        .expect("ar est introuvable");
    if !status.success() {
        panic!("ar n'a pas pu archiver l'icône");
    }

    println!("cargo:rustc-link-search=native={}", out.display());
    // Sans +whole-archive, ld --gc-sections jette l'icône : rien dans le
    // programme ne la référence.
    println!("cargo:rustc-link-lib=static:+whole-archive=resource");
}
