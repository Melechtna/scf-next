fn main() {
    tauri_build::build();

    let sdl_dir = std::path::Path::new("libs/SDL");
    if !sdl_dir.join("CMakeLists.txt").exists() {
        eprintln!("[scf-next] SDL3 submodule not found at libs/SDL — run: git submodule update --init --recursive");
        return;
    }

    let target = std::env::var("TARGET").unwrap_or_default();
    let is_windows = target.contains("windows");
    let is_macos = target.contains("apple");

    let mut config = cmake::Config::new(sdl_dir);
    config
        .generator("Ninja")
        .define("SDL_STATIC", "ON")
        .define("SDL_SHARED", "OFF")
        .define("SDL_HIDAPI", "ON")
        .define("SDL_JOYSTICK", "ON")
        .define("SDL_AUDIO", "OFF")
        .define("SDL_RENDER", "OFF")
        .define("SDL_VIDEO", "OFF")
        .define("SDL_CAMERA", "OFF")
        .define("SDL_OPENGL", "OFF")
        .define("SDL_VULKAN", "OFF")
        .define("SDL_METAL", "OFF");

    if is_windows {
        config
            .define("CMAKE_SYSTEM_NAME", "Windows")
            .define("CMAKE_C_COMPILER", "x86_64-w64-mingw32-gcc")
            .define("CMAKE_CXX_COMPILER", "x86_64-w64-mingw32-g++")
            .define("CMAKE_RC_COMPILER", "x86_64-w64-mingw32-windres")
            .define("SDL_DIRECTX", "ON")
            .define("SDL_XINPUT", "ON")
            .define("SDL_WAYLAND", "OFF")
            .define("SDL_KMSDRM", "OFF")
            .define("SDL_UNIX_CONSOLE_BUILD", "OFF");
    } else if is_macos {
        config
            .define("SDL_X11", "OFF")
            .define("SDL_WAYLAND", "OFF")
            .define("SDL_KMSDRM", "OFF")
            .define("SDL_UNIX_CONSOLE_BUILD", "OFF");
    } else {
        config
            .define("SDL_WAYLAND", "OFF")
            .define("SDL_KMSDRM", "OFF")
            .define("SDL_UNIX_CONSOLE_BUILD", "ON");
    }

    let dst = config.build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=SDL3");

    if is_windows {
        println!("cargo:rustc-link-lib=dylib=setupapi");
        println!("cargo:rustc-link-lib=dylib=imm32");
        println!("cargo:rustc-link-lib=dylib=winmm");
        println!("cargo:rustc-link-lib=dylib=gdi32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=kernel32");
        println!("cargo:rustc-link-lib=dylib=ole32");
        println!("cargo:rustc-link-lib=dylib=oleaut32");
        println!("cargo:rustc-link-lib=dylib=comctl32");
        println!("cargo:rustc-link-lib=dylib=version");
        println!("cargo:rustc-link-lib=dylib=hid");
        println!("cargo:rustc-link-lib=dylib=dinput8");
        println!("cargo:rustc-link-lib=dylib=dxguid");
        println!("cargo:rustc-link-lib=dylib=uuid");
        println!("cargo:rustc-link-lib=dylib=shell32");
        println!("cargo:rustc-link-lib=dylib=shlwapi");
    } else if is_macos {
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=ForceFeedback");
        println!("cargo:rustc-link-lib=framework=Carbon");
        println!("cargo:rustc-link-lib=framework=GameController");
        println!("cargo:rustc-link-lib=framework=CoreHaptics");
        println!("cargo:rustc-link-lib=framework=UniformTypeIdentifiers");
        println!("cargo:rustc-link-lib=pthread");
    } else {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=dl");
        println!("cargo:rustc-link-lib=rt");
    }

    println!(
        "cargo:rerun-if-changed={}",
        sdl_dir.join("CMakeLists.txt").display()
    );
}
