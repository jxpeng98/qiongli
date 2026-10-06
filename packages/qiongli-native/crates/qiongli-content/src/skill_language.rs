//! Install-time Skill presentation; invocation names and workflow bodies stay stable.
use serde_json::Value;

use crate::LoadedResourcePack;

pub const SKILL_DESCRIPTIONS_PATH: &str = "workflow/references/skill-descriptions.json";

#[must_use]
pub fn skill_language_valid(language: &str) -> bool {
    matches!(language, "en" | "zh")
}

/// Unsupported system locales use English. Explicit CLI choices are validated separately.
#[must_use]
pub fn skill_language_for_locale(locale: &str) -> &'static str {
    if locale
        .trim()
        .split(['-', '_', '.', ':'])
        .next()
        .is_some_and(|s| s.eq_ignore_ascii_case("zh"))
    {
        "zh"
    } else {
        "en"
    }
}

pub fn localize_skill_metadata(
    pack: &LoadedResourcePack<'_>,
    path: &str,
    bytes: &[u8],
    language: &str,
) -> Result<Vec<u8>, &'static str> {
    if !skill_language_valid(language) {
        return Err("skill-language-invalid");
    }
    let name = match path {
        "workflow/SKILL.md" | "workflow/agents/openai.yaml" => "qiongli".to_owned(),
        "workflow/no-qiongli/SKILL.md" => "no-qiongli".to_owned(),
        _ => match path
            .strip_prefix("workflow/workflows/")
            .and_then(|p| p.strip_suffix(".md"))
        {
            Some("qiongli") | None => return Ok(bytes.to_vec()),
            Some(slug) if !slug.contains('/') => format!("qiongli-{slug}"),
            _ => return Ok(bytes.to_vec()),
        },
    };
    let Some(catalog) = pack
        .resource_for_profile("full", SKILL_DESCRIPTIONS_PATH)
        .map_err(|_| "skill-descriptions-invalid")?
    else {
        // Historical packs have no translated metadata and keep their exact bytes.
        return Ok(bytes.to_vec());
    };
    let catalog: Value =
        serde_json::from_slice(catalog.bytes()).map_err(|_| "skill-descriptions-invalid")?;
    let entry = &catalog[&name][language];
    let text = std::str::from_utf8(bytes).map_err(|_| "skill-descriptions-invalid")?;
    let fields: &[(&str, &str)] = if path.ends_with("openai.yaml") {
        &[
            ("  display_name: ", "display_name"),
            ("  short_description: ", "short_description"),
            ("  default_prompt: ", "default_prompt"),
        ]
    } else {
        &[("description: ", "description")]
    };
    let mut output = text.to_owned();
    for (prefix, field) in fields {
        let value = entry[field]
            .as_str()
            .filter(|s| !s.is_empty() && !s.chars().any(char::is_control))
            .ok_or("skill-descriptions-invalid")?;
        let old = output
            .lines()
            .find(|line| line.starts_with(prefix))
            .ok_or("skill-descriptions-invalid")?;
        let value = if path == "workflow/SKILL.md" && *field == "description" {
            if language == "zh" {
                format!("Qiongli v{}。{value}", pack.manifest().content_version)
            } else {
                format!(
                    "Qiongli version: v{}. {value}",
                    pack.manifest().content_version
                )
            }
        } else {
            value.to_owned()
        };
        let replacement = format!(
            "{prefix}{}",
            serde_json::to_string(&value).map_err(|_| "skill-descriptions-invalid")?
        );
        output = output.replacen(old, &replacement, 1);
    }
    Ok(output.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn supported_and_fallback_locales() {
        for locale in ["zh_CN.UTF-8", "zh-Hant-TW", "ZH", " zh "] {
            assert_eq!(skill_language_for_locale(locale), "zh");
        }
        for locale in ["en_GB.UTF-8", "C", "", "fr", "zhx"] {
            assert_eq!(skill_language_for_locale(locale), "en");
        }
        assert!(!skill_language_valid("auto"));
    }
}
