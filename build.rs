fn main() {
    println!("cargo:rustc-link-search=/Projects/stack-overflow/using-c-static/");
    let apama_home = std::env::var("APAMA_HOME").expect("Should be run from Apama shell (APAMA_HOME env not set).");
    cc::Build::new()
        .cpp(true)
        .file("RustTransport.cpp")
        .flag("-D__unix__")
        .flag("-D__OSVERSION__=2")
        .flag("-D__STDC_FORMAT_MACROS")
        .flag("-D__STDC_CONSTANT_MACROS")
        .flag("-D__STDC_LIMIT_MACROS")
        .flag(&format!("-L{}/lib -lapclient", apama_home))
        .include(".")
        .include(&format!("{}/include", apama_home))
        .pic(true)
        .flag("-std=c++11")
        .flag("-pedantic")
        .warnings(false)
        .flag("-Werror")
        .flag("-Wall")
        .compile("cpplayer");
}