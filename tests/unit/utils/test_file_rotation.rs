//! Unit tests for file rotation utility
//!
//! Target: ≥90% coverage per file

use arrow_zerobus_sdk_wrapper::utils::file_rotation::rotate_file_if_needed;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_rotate_file_if_needed_file_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nonexistent.txt");
    
    let result = rotate_file_if_needed(&file_path, 1000).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_rotate_file_if_needed_file_smaller_than_max() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("small.txt");
    
    // Create a small file
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(b"small content").unwrap();
    file.sync_all().unwrap();
    
    let result = rotate_file_if_needed(&file_path, 1000).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_rotate_file_if_needed_file_larger_than_max() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large.txt");
    
    // Create a large file (2KB)
    let mut file = fs::File::create(&file_path).unwrap();
    let large_content = vec![b'A'; 2048];
    file.write_all(&large_content).unwrap();
    file.sync_all().unwrap();
    
    let result = rotate_file_if_needed(&file_path, 1000).unwrap();
    assert!(result.is_some());
    
    let new_path = result.unwrap();
    assert!(new_path.exists() == false); // New path doesn't exist yet, just created
    assert!(new_path.file_name().unwrap().to_string_lossy().contains("large_"));
    assert!(new_path.extension().unwrap() == "txt");
}

#[test]
fn test_rotate_file_if_needed_exact_size() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("exact.txt");
    
    // Create a file exactly at max size
    let mut file = fs::File::create(&file_path).unwrap();
    let content = vec![b'B'; 1000];
    file.write_all(&content).unwrap();
    file.sync_all().unwrap();
    
    // File is exactly max size, should not rotate (needs to exceed)
    let result = rotate_file_if_needed(&file_path, 1000).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_rotate_file_if_needed_timestamp_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.arrow");
    
    // Create a large file
    let mut file = fs::File::create(&file_path).unwrap();
    let large_content = vec![b'C'; 2048];
    file.write_all(&large_content).unwrap();
    file.sync_all().unwrap();
    
    let result = rotate_file_if_needed(&file_path, 1000).unwrap();
    assert!(result.is_some());
    
    let new_path = result.unwrap();
    let filename = new_path.file_name().unwrap().to_string_lossy();
    
    // Check timestamp format: test_YYYYMMDD_HHMMSS.arrow
    assert!(filename.starts_with("test_"));
    assert!(filename.ends_with(".arrow"));
    assert!(filename.len() > "test_".len() + ".arrow".len());
}

