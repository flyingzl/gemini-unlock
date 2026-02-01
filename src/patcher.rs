use serde_json::{json, Value};

use crate::error::{AppError, AppResult};

/// Patch execution result.
///
/// # Examples
///
/// ```text
/// PatchReport { changed_is_glic: true, .. }
/// ```
#[derive(Debug)]
pub struct PatchReport {
    /// Complete content after changes.
    pub content: String,
    /// Whether is_glic_eligible was modified.
    pub changed_is_glic: bool,
    /// Whether variations_country was modified.
    pub changed_variations_country: bool,
    /// Whether variations_permanent_consistency_country was modified.
    pub changed_variations_permanent_country: bool,
}

/// Apply Gemini unlock patch.
///
/// This function uses serde_json to safely parse and modify JSON configuration,
/// avoiding format corruption issues that may be caused by regular expressions.
///
/// # Examples
///
/// ```
/// use gemini_unlock::patcher::apply_patches;
///
/// let input = r#"{"is_glic_eligible": false}"#;
/// let report = apply_patches(input).unwrap();
/// assert!(report.changed_is_glic);
/// ```
pub fn apply_patches(input: &str) -> AppResult<PatchReport> {
    // 1. Parse JSON, validate input
    let mut json: Value = serde_json::from_str(input)
        .map_err(|e| AppError::InvalidJson(format!("Input JSON parsing failed: {e}")))?;

    let mut report = PatchReport {
        content: String::new(),
        changed_is_glic: false,
        changed_variations_country: false,
        changed_variations_permanent_country: false,
    };

    // 2. Ensure it's a JSON object
    let obj = json.as_object_mut()
        .ok_or_else(|| AppError::InvalidJson("Config file root is not an object".to_string()))?;

    // 3. Safely modify is_glic_eligible
    if let Some(value) = obj.get_mut("is_glic_eligible") {
        if value.is_boolean() {
            *value = json!(true);
            report.changed_is_glic = true;
        }
    }

    // 4. Safely modify variations_country
    if obj.contains_key("variations_country") {
        obj.insert("variations_country".into(), json!("us"));
        report.changed_variations_country = true;
    }

    // 5. Safely modify variations_permanent_consistency_country
    if let Some(arr) = obj.get_mut("variations_permanent_consistency_country") {
        if arr.is_array() {
            *arr = json!(["us"]);
            report.changed_variations_permanent_country = true;
        }
    }

    // 6. Serialize back to JSON (auto-format and validate)
    report.content = serde_json::to_string_pretty(obj)
        .map_err(|e| AppError::InvalidJson(format!("Output JSON serialization failed: {e}")))?;

    // 7. Validate output again to ensure it's valid JSON
    serde_json::from_str::<Value>(&report.content)
        .map_err(|e| AppError::InvalidJson(format!("Generated JSON validation failed: {e}")))?;

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches_all_fields() {
        let input = r#"{"is_glic_eligible": false, "variations_country":"cn", "variations_permanent_consistency_country":["cn"]}"#;
        let report = apply_patches(input).expect("Patch application failed");

        assert!(report.changed_is_glic);
        assert!(report.changed_variations_country);
        assert!(report.changed_variations_permanent_country);

        // Verify output is valid JSON
        let output: Value = serde_json::from_str(&report.content).expect("Output is invalid");
        assert_eq!(output["is_glic_eligible"], true);
        assert_eq!(output["variations_country"], "us");
        assert_eq!(output["variations_permanent_consistency_country"], json!(["us"]));
    }

    #[test]
    fn handles_missing_fields() {
        let input = r#"{"unrelated":123}"#;
        let report = apply_patches(input).expect("Should handle missing fields");

        assert!(!report.changed_is_glic);
        assert!(!report.changed_variations_country);
        assert!(!report.changed_variations_permanent_country);
    }

    #[test]
    fn preserves_other_fields() {
        let input = r#"{"is_glic_eligible": false, "other_field": "unchanged", "number": 42}"#;
        let report = apply_patches(input).expect("Should preserve other fields");

        let output: Value = serde_json::from_str(&report.content).expect("Output is invalid");
        assert_eq!(output["other_field"], "unchanged");
        assert_eq!(output["number"], 42);
    }

    #[test]
    fn handles_escaped_quotes() {
        // Scenario where regex would fail, but JSON parsing handles correctly
        let input = r#"{"comment": "The value \"is_glic_eligible\" should be true", "is_glic_eligible": false}"#;
        let report = apply_patches(input).expect("Should handle escaped quotes correctly");

        assert!(report.changed_is_glic);
        let output: Value = serde_json::from_str(&report.content).expect("Output is invalid");
        assert_eq!(output["is_glic_eligible"], true);
        assert_eq!(output["comment"], "The value \"is_glic_eligible\" should be true");
    }

    #[test]
    fn handles_nested_structures() {
        let input = r#"{"nested": {"is_glic_eligible": false}, "is_glic_eligible": false}"#;
        let report = apply_patches(input).expect("Should handle nested structures correctly");

        // Only modify top-level fields
        let output: Value = serde_json::from_str(&report.content).expect("Output is invalid");
        assert_eq!(output["is_glic_eligible"], true);
        assert_eq!(output["nested"]["is_glic_eligible"], false);
    }

    #[test]
    fn rejects_invalid_json_input() {
        let input = r#"{invalid json}"#;
        let result = apply_patches(input);

        assert!(result.is_err(), "Should reject invalid JSON input");
    }

    #[test]
    fn handles_array_variations_country() {
        let input = r#"{"variations_country": ["cn", "us"], "is_glic_eligible": false}"#;
        let report = apply_patches(input).expect("Should handle array type variations_country");

        // Replace if field exists, regardless of original type
        let output: Value = serde_json::from_str(&report.content).expect("Output is invalid");
        assert_eq!(output["variations_country"], "us");
    }

    #[test]
    fn handles_non_boolean_is_glic() {
        // If is_glic_eligible is not boolean, should not modify
        let input = r#"{"is_glic_eligible": "false"}"#;
        let report = apply_patches(input).expect("Should handle successfully");

        assert!(!report.changed_is_glic);
        let output: Value = serde_json::from_str(&report.content).expect("Output is invalid");
        assert_eq!(output["is_glic_eligible"], "false");
    }
}
