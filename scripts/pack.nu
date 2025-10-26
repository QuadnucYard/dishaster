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

    cd $target_dir
    let output_file = "../dishaster.7z"
    if ($output_file | path exists) {
        print "Removing existing archive..."
        rm $output_file
    }
    ^7z a $output_file * -r
}
