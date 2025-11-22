def main [--platform: string = "windows", --release] {
    # Build rust
    if $release {
        cargo build --release --features production
    } else {
        cargo build --features production
    }

    cp -u -r assets/data godot

    # Build godot
    let target_dir = if $release {
        "export/release"
    } else {
        "export/debug"
    }
    if ($target_dir | path exists) {
        print "Clean existing export files..."
        rm -r $target_dir # clean up old export
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

    # Copy external files
    cp -r godot/locales $target_dir
    cp -u LICENSE $target_dir
    cp -u README.md $target_dir
}
