#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
))]
use std::path::PathBuf;

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
))]
use zeroshot_clipboard::paths_from_uri_list;

#[test]
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
))]
fn test_decoding_uri_list() {
    // Test that paths_from_uri_list correctly decodes
    // differents percent encoded characters
    let file_list = vec![
        "file:///tmp/bar.log",
        "file:///tmp/test%5C.txt",
        "file:///tmp/foo%3F.png",
        "file:///tmp/white%20space.txt",
    ];

    let paths = vec![
        PathBuf::from("/tmp/bar.log"),
        PathBuf::from("/tmp/test\\.txt"),
        PathBuf::from("/tmp/foo?.png"),
        PathBuf::from("/tmp/white space.txt"),
    ];
    assert_eq!(paths_from_uri_list(file_list.join("\n")), paths);
}
