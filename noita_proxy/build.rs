fn main() {
    println!("cargo:rerun-if-env-changed=EW_RELEASE_REPOSITORY");
    println!("cargo:rerun-if-env-changed=EW_RELEASE_TAG");
    println!("cargo:rerun-if-env-changed=GITHUB_REPOSITORY");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");

    if let Some(repository) =
        std::env::var_os("EW_RELEASE_REPOSITORY").or_else(|| std::env::var_os("GITHUB_REPOSITORY"))
    {
        println!(
            "cargo:rustc-env=EW_RELEASE_REPOSITORY={}",
            repository.to_string_lossy()
        );
    }

    if let Some(tag) = std::env::var_os("EW_RELEASE_TAG").or_else(|| {
        std::env::var("GITHUB_REF_NAME")
            .ok()
            .filter(|ref_name| ref_name.starts_with('v'))
            .map(Into::into)
    }) {
        println!("cargo:rustc-env=EW_RELEASE_TAG={}", tag.to_string_lossy());
    }

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");

        res.compile().unwrap();
    }
}
