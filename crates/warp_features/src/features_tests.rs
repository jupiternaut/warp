use super::*;

#[test]
#[ignore = "CORE-3768 - need to clean up PREVIEW_FLAGS, but this is a temporary fix for the cluttered changelog"]
fn test_all_preview_flags_have_a_description() {
    for flag in PREVIEW_FLAGS {
        assert!(
            flag.flag_description()
                .is_some_and(|description| !description.is_empty()),
            "Missing description for preview-enabled flag {flag:?}"
        );
    }
}

#[test]
fn vs_code_extensions_flag_is_canonical_feature_flag_variant() {
    assert!(enum_iterator::all::<FeatureFlag>().any(|flag| flag == FeatureFlag::VsCodeExtensions));
    assert_eq!(
        format!("{:?}", FeatureFlag::VsCodeExtensions),
        "VsCodeExtensions"
    );
}
