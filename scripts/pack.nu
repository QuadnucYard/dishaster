def main [--release] {
    let export_dir = "export"
    let target_dir = if $release {
        $"($export_dir)/release"
    } else {
        $"($export_dir)/debug"
    }

    if not ($target_dir | path exists) {
        panic $"Target directory does not exist: ($target_dir). Please run the export script first."
    }

    ^7z a $"($export_dir)/dishaster.7z" $"($target_dir)/*"
}
