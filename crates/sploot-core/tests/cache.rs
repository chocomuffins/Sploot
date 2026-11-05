use sploot_core::cache::CacheKey;
use sploot_core::stats::{DEFAULT_BINS, Histograms, ImageStats};
use sploot_core::{ColorSpace, Compression, Quality, SPLOOT_VERSION};
use std::path::{Path, PathBuf};

const CHECKER_TEXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../samples/sploot_checker.png"
);

#[test]
fn hash_test() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../samples/sploot_checker.png"
    );

    let path_hashed = blake3::hash(path.as_bytes());
    println!("Path: {0}", path_hashed);

    let modified_time = std::fs::metadata(path).expect("sample missing").modified();
    println!("Last modified: {:?}", modified_time);

    let version_hashed = blake3::hash(SPLOOT_VERSION.as_bytes());
    println!("Version: {0}", version_hashed);
}

fn checker_key() -> CacheKey {
    let (path, file_size, modified) =
        CacheKey::fingerprint(Path::new(CHECKER_TEXTURE)).expect("sample missing");

    CacheKey {
        path,
        file_size,
        modified,
        width: 512,
        height: 512,
        sploot_version: SPLOOT_VERSION.to_string(),
        color_space: ColorSpace::Srgb,
        compression: Compression::Bc1,
        quality: Quality::Normal,
        mip_level: 0,
        bins: DEFAULT_BINS,
    }
}

const PIXELS: [[f32; 4]; 3] = [
    [0.2, 0.4, 0.6, 1.0],
    [0.8, 0.1, 0.3, 1.0],
    [0.5, 0.5, 0.5, 1.0],
];

fn some_stats() -> ImageStats {
    ImageStats::from_pixels(&PIXELS, None).expect("pixels are not empty")
}

fn some_histograms() -> Histograms {
    Histograms::from_pixels(&PIXELS, DEFAULT_BINS, 0.0, 1.0)
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("cache")
        .join(name);

    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn file_stem_is_stable_hex() {
    let stem = checker_key().file_stem();
    assert_eq!(stem.len(), 64);
    assert!(stem.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(stem, checker_key().file_stem(), "same key, same name");
}

#[test]
fn file_stem_changes_with_settings() {
    let a = checker_key();
    let b = CacheKey {
        color_space: ColorSpace::Linear,
        ..a.clone()
    };
    let c = CacheKey {
        mip_level: 1,
        ..a.clone()
    };
    let d = CacheKey {
        bins: 64,
        ..a.clone()
    };
    assert_ne!(a.file_stem(), b.file_stem());
    assert_ne!(a.file_stem(), c.file_stem());
    assert_ne!(a.file_stem(), d.file_stem());
}

#[test]
fn save_then_load_round_trips() {
    let dir = temp_dir("round_trip");
    let (key, stats, histograms) = (checker_key(), some_stats(), some_histograms());
    key.save(&dir, &stats, &histograms).unwrap();
    assert_eq!(key.load(&dir), Some((stats, histograms)));
}

#[test]
fn save_leaves_no_tmp_file() {
    let dir = temp_dir("no_tmp");
    checker_key()
        .save(&dir, &some_stats(), &some_histograms())
        .unwrap();
    let names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names.len(), 1);
    assert!(names[0].to_string_lossy().ends_with(".json"));
}

#[test]
fn load_misses() {
    let dir = temp_dir("misses");
    let key = checker_key();
    assert_eq!(key.load(&dir), None, "empty folder");

    key.save(&dir, &some_stats(), &some_histograms()).unwrap();
    let other = CacheKey {
        color_space: ColorSpace::Linear,
        ..key
    };
    assert_eq!(other.load(&dir), None, "different settings");
}

#[test]
fn fingerprint_changes_when_file_changes() {
    let file = temp_dir("fingerprint").join("texture.bin");
    std::fs::write(&file, b"one").unwrap();
    let before = CacheKey::fingerprint(&file).unwrap();
    std::fs::write(&file, b"three").unwrap();
    let after = CacheKey::fingerprint(&file).unwrap();
    assert_eq!(before.0, after.0, "same path");
    assert_ne!(before.1, after.1, "different size");
}
