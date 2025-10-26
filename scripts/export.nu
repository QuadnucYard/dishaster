def main [--platform: string = "windows", --release] {
    # Build rust
    if $release {
        cargo build --release
    } else {
        cargo build
    }

    # Build godot
    let target_dir = if $release {
        "export/release"
    } else {
        "export/debug"
    }
    mkdir $target_dir

    let preset_name = if $platform == "windows" {
        "Windows Desktop"
    } else {
        fail $"Unsupported platform: ($platform)"
    }
    cd godot
    let export_flag = if $release {
        "--export-release"
    } else {
        "--export-debug"
    }
    godot --headless $export_flag $preset_name $"../($target_dir)/dishaster.exe"
    cd ..

    # Copy resources
    cp -r assets/data $"($target_dir)/data"
    cp -r godot/locales $"($target_dir)/locales"
}
