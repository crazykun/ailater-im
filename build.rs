// build.rs - Build script for ailater-im

fn main() {
    // Check if pkg-config is available
    if std::env::var("CARGO_FEATURE_FCITX5").is_ok() {
        println!("cargo:info=Building with fcitx5 support");
        
        // Check if fcitx5 headers exist (Deepin/Debian path)
        let fcitx_header = "/usr/include/Fcitx5/Core/fcitx/addonfactory.h";
        if !std::path::Path::new(fcitx_header).exists() {
            println!("cargo:warning=fcitx5 headers not found at {}", fcitx_header);
            println!("cargo:warning=Please install fcitx5 development package:");
            println!("cargo:warning=  Debian/Ubuntu: apt install libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev");
            println!("cargo:warning=  Arch Linux: pacman -S fcitx5");
            println!("cargo:warning=Building without fcitx5 C++ wrapper");
            return;
        }
        
        println!("cargo:info=Compiling C++ wrapper");
        
        // Get OUT_DIR for static library location
        let out_dir = std::env::var("OUT_DIR").unwrap();
        
        // Compile the C++ wrapper
        cc::Build::new()
            .cpp(true)
            .file("src/ffi_wrapper.cpp")
            .flag("-std=c++17")
            .flag("-fPIC")
            .flag("-fvisibility=default")  // Make symbols visible
            .include("/usr/include")
            .include("/usr/include/Fcitx5/Core")
            .include("/usr/include/Fcitx5/Utils")
            .include("/usr/include/Fcitx5/Config")
            .out_dir(&out_dir)
            .compile("ffi_wrapper");
        
        // Link fcitx5 libraries
        let lib_output = std::process::Command::new("pkg-config")
            .args(["--libs", "Fcitx5Core", "Fcitx5Utils", "Fcitx5Config"])
            .output();
        
        if let Ok(output) = lib_output {
            if output.status.success() {
                let libs = String::from_utf8_lossy(&output.stdout);
                for lib in libs.split_whitespace() {
                    if lib.starts_with("-l") {
                        println!("cargo:rustc-link-lib=dylib={}", &lib[2..]);
                    } else if lib.starts_with("-L") {
                        println!("cargo:rustc-link-search=native={}", &lib[2..]);
                    }
                }
            }
        }
        
        // Link standard C++ library
        println!("cargo:rustc-link-lib=dylib=stdc++");
        
        // Link the static library
        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=static=ffi_wrapper");
    }
    
    println!("cargo:rerun-if-changed=src/ffi_wrapper.cpp");
    println!("cargo:rerun-if-changed=build.rs");
}