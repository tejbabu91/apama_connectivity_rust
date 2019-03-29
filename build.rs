fn main() {
    println!("cargo:rustc-link-search=/Projects/stack-overflow/using-c-static/");
    let apama_home = std::env::var("APAMA_HOME").unwrap();
    cc::Build::new()
        .cpp(true)
        .file("RustTransport.cpp")
        .flag("-D__linux__ -D__unix__ -D__OSVERSION__=2 -D__STDC_FORMAT_MACROS -D__STDC_CONSTANT_MACROS -D__STDC_LIMIT_MACROS")
        .flag(&format!("-L{}/lib -lapclient", apama_home))
        .include(".")
        .include(&format!("{}/include", apama_home))
        .pic(true).flag("-std=c++11")
        .compile("cpplayer");
}