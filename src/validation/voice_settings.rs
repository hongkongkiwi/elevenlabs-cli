//! Voice settings validation utilities

use anyhow::Result;

/// Valid range for voice settings
const MIN_VALUE: f32 = 0.0;
const MAX_VALUE: f32 = 1.0;

/// Validate stability value is within valid range (0.0-1.0)
pub fn validate_stability(value: f32) -> Result<f32> {
    if !(MIN_VALUE..=MAX_VALUE).contains(&value) {
        Err(anyhow::anyhow!("Stability must be between 0.0 and 1.0"))
    } else {
        Ok(value)
    }
}

/// Validate similarity boost value is within valid range (0.0-1.0)
pub fn validate_similarity_boost(value: f32) -> Result<f32> {
    if !(MIN_VALUE..=MAX_VALUE).contains(&value) {
        Err(anyhow::anyhow!(
            "Similarity boost must be between 0.0 and 1.0"
        ))
    } else {
        Ok(value)
    }
}

/// Validate style value is within valid range (0.0-1.0)
pub fn validate_style(value: f32) -> Result<f32> {
    if !(MIN_VALUE..=MAX_VALUE).contains(&value) {
        Err(anyhow::anyhow!("Style must be between 0.0 and 1.0"))
    } else {
        Ok(value)
    }
}

/// Validate all voice settings if provided
pub fn validate_voice_settings(
    stability: Option<f32>,
    similarity_boost: Option<f32>,
    style: Option<f32>,
) -> Result<()> {
    if let Some(s) = stability {
        validate_stability(s)?;
    }
    if let Some(sb) = similarity_boost {
        validate_similarity_boost(sb)?;
    }
    if let Some(st) = style {
        validate_style(st)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_stability_valid() {
        assert!(validate_stability(0.0).is_ok());
        assert!(validate_stability(0.5).is_ok());
        assert!(validate_stability(1.0).is_ok());
    }

    #[test]
    fn test_validate_stability_invalid() {
        assert!(validate_stability(-0.1).is_err());
        assert!(validate_stability(1.1).is_err());
    }

    #[test]
    fn test_validate_similarity_boost_valid() {
        assert!(validate_similarity_boost(0.0).is_ok());
        assert!(validate_similarity_boost(0.75).is_ok());
    }

    #[test]
    fn test_validate_style_invalid() {
        assert!(validate_style(-1.0).is_err());
        assert!(validate_style(2.0).is_err());
    }

    #[test]
    fn test_validate_voice_settings_all_valid() {
        assert!(validate_voice_settings(Some(0.5), Some(0.75), Some(0.25)).is_ok());
    }

    #[test]
    fn test_validate_voice_settings_partial() {
        assert!(validate_voice_settings(Some(0.5), None, None).is_ok());
        assert!(validate_voice_settings(None, Some(0.75), None).is_ok());
    }

    #[test]
    fn test_validate_voice_settings_one_invalid() {
        assert!(validate_voice_settings(Some(2.0), None, None).is_err());
    }
}
