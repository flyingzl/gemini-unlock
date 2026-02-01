// Integration tests: Test complete workflow

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

use gemini_unlock::patcher::apply_patches;
use serde_json::json;

/// Create test JSON configuration file
fn create_test_config(dir: &PathBuf, name: &str) -> PathBuf {
    let config_path = dir.join(name);
    let content = r#"{
  "is_glic_eligible": false,
  "variations_country": "cn",
  "variations_permanent_consistency_country": ["cn"],
  "profile": {
    "name": "Test Profile"
  }
}"#;
    let mut file = fs::File::create(&config_path).expect("Unable to create test file");
    file.write_all(content.as_bytes()).expect("Unable to write test file");
    config_path
}

#[test]
fn test_complete_patch_workflow() {
    // Create temporary directory
    let temp_dir = TempDir::new().expect("Unable to create temporary directory");
    let config_path = create_test_config(&temp_dir.path().to_path_buf(), "Local State");

    // Read original content
    let original_content = fs::read_to_string(&config_path).expect("Unable to read config file");

    // Apply patches
    let report = apply_patches(&original_content).expect("Patch application failed");

    // Verify modifications
    assert!(report.changed_is_glic, "Should modify is_glic_eligible");
    assert!(report.changed_variations_country, "Should modify variations_country");
    assert!(
        report.changed_variations_permanent_country,
        "Should modify variations_permanent_consistency_country"
    );

    // Verify output is valid JSON
    let output: serde_json::Value = serde_json::from_str(&report.content)
        .expect("Output should be valid JSON");

    // Verify specific modifications
    assert_eq!(output["is_glic_eligible"], true);
    assert_eq!(output["variations_country"], "us");
    assert_eq!(output["variations_permanent_consistency_country"], json!(["us"]));

    // Verify other fields are preserved
    assert_eq!(output["profile"]["name"], "Test Profile");
}

#[test]
fn test_patch_preserves_structure() {
    let temp_dir = TempDir::new().expect("Unable to create temporary directory");
    let config_path = create_test_config(&temp_dir.path().to_path_buf(), "Local State");

    let content = fs::read_to_string(&config_path).expect("Unable to read config file");
    let report = apply_patches(&content).expect("Patch application failed");

    // Parse original and modified JSON
    let original: serde_json::Value = serde_json::from_str(&content).expect("Original JSON is invalid");
    let modified: serde_json::Value = serde_json::from_str(&report.content).expect("Modified JSON is invalid");

    // Verify object structure is preserved
    assert!(original.is_object());
    assert!(modified.is_object());

    // Verify all original fields exist in modified file
    if let Some(obj) = original.as_object() {
        for key in obj.keys() {
            assert!(
                modified.get(key).is_some(),
                "Field {} should be preserved",
                key
            );
        }
    }
}

#[test]
fn test_patch_minimal_config() {
    // Test configuration with only required fields
    let content = r#"{"is_glic_eligible": false}"#;
    let report = apply_patches(content).expect("Patch application failed");

    assert!(report.changed_is_glic);

    let output: serde_json::Value = serde_json::from_str(&report.content).expect("Output is invalid");
    assert_eq!(output["is_glic_eligible"], true);
}

#[test]
fn test_patch_with_nested_fields() {
    // Test configuration containing nested fields
    let content = r#"{
  "is_glic_eligible": false,
  "variations_country": "cn",
  "nested": {
    "is_glic_eligible": false,
    "variations_country": "nested"
  }
}"#;

    let report = apply_patches(content).expect("Patch application failed");

    let output: serde_json::Value = serde_json::from_str(&report.content).expect("Output is invalid");

    // Verify only top-level fields are modified
    assert_eq!(output["is_glic_eligible"], true);
    assert_eq!(output["variations_country"], "us");
    assert_eq!(output["nested"]["is_glic_eligible"], false);
    assert_eq!(output["nested"]["variations_country"], "nested");
}

#[test]
fn test_error_handling_invalid_json() {
    let invalid_json = r#"{invalid json}"#;
    let result = apply_patches(invalid_json);

    assert!(result.is_err(), "Should reject invalid JSON");
}

#[test]
fn test_error_handling_non_boolean_is_glic() {
    // Test case where is_glic_eligible is not boolean
    let content = r#"{"is_glic_eligible": "false"}"#;
    let report = apply_patches(content).expect("Should handle successfully");

    // Should not modify non-boolean value
    assert!(!report.changed_is_glic);

    let output: serde_json::Value = serde_json::from_str(&report.content).expect("Output is invalid");
    assert_eq!(output["is_glic_eligible"], "false");
}

#[test]
fn test_patch_empty_config() {
    let content = r#"{}"#;
    let report = apply_patches(content).expect("Should handle empty config");

    assert!(!report.changed_is_glic);
    assert!(!report.changed_variations_country);
    assert!(!report.changed_variations_permanent_country);

    // Verify output is still valid JSON object
    let output: serde_json::Value = serde_json::from_str(&report.content).expect("Output is invalid");
    assert!(output.is_object());
    assert!(output.as_object().unwrap().is_empty());
}

#[test]
fn test_patch_large_config() {
    // Test configuration containing many fields
    let mut config = serde_json::json!({
        "is_glic_eligible": false,
        "variations_country": "cn"
    });

    // Add 100 additional fields
    for i in 0..100 {
        config[format!("field_{}", i)] = serde_json::json!(i);
    }

    let content = serde_json::to_string(&config).expect("Serialization failed");
    let report = apply_patches(&content).expect("Patch application failed");

    assert!(report.changed_is_glic);
    assert!(report.changed_variations_country);

    // Verify all fields are preserved
    let output: serde_json::Value = serde_json::from_str(&report.content).expect("Output is invalid");
    for i in 0..100 {
        assert!(
            output.get(format!("field_{}", i)).is_some(),
            "Field {} should exist",
            i
        );
    }
}
