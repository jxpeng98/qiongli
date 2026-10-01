//! Confirmed DSH npm installation; the official manager owns profile/cache writes.
use std::fs;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use qiongli_content::EmbeddedContent;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::cli_content::{confirm, line, show_json};
use crate::command::CommandEnvironment;

#[derive(Eq, PartialEq, Serialize)]
struct InstallPlan {
    executable: PathBuf,
    executable_sha256: String,
    profile: String,
    profile_directory: PathBuf,
    profile_files: Vec<(String, Option<String>)>,
    commands: Vec<Vec<String>>,
    skill_language: Option<String>,
}

pub(crate) fn install(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    language: &str,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<bool, &'static str> {
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let root = environment
        .dsh_config_root()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".dsh"));
    let discovered = environment
        .client_executable("dsh")
        .ok_or("host-plugin-executable-unavailable")?;
    let executable = crate::desktop::resolve_host_plugin_executable(home, "dsh", &discovered)?;
    let version = run(environment, &executable, &["--version".into()])?;
    let version =
        semver::Version::parse(version.trim()).map_err(|_| "deepseek-version-unsupported")?;
    if version.major == 0 && version.minor < 2 {
        return Err("deepseek-version-unsupported");
    }
    let default = if root.join("profiles/desktop/package.json").is_file() {
        "desktop"
    } else {
        "web"
    };
    let profile = crate::cli_inventory::choice(
        reader,
        writer,
        &format!("DSH profile (0 cancels) [{default}]: "),
    )
    .map_err(|_| "installation-input-failed")?;
    if profile == "0" {
        return Ok(false);
    }
    let profile = if profile.is_empty() {
        default
    } else {
        &profile
    };
    let mut plan = prepare(&executable, &root, profile)?;
    if !qiongli_content::skill_language_valid(language) {
        return Err("skill-language-invalid");
    }
    plan.skill_language = Some(language.into());
    review(environment, content, &root, plan, reader, writer)
}

fn review(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    root: &Path,
    plan: InstallPlan,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<bool, &'static str> {
    show_json(writer, &serde_json::json!({
        "host": "DeepSeek Harness", "profile": plan.profile, "profile_directory": plan.profile_directory,
        "executable": plan.executable, "package": format!("qiongli@{}", env!("CARGO_PKG_VERSION")),
        "commands": plan.commands.iter().map(|args| serde_json::json!({"arguments": serde_json::json!(args).to_string()})).collect::<Vec<_>>(),
        "skill_language": plan.skill_language,
        "language_preference_file": plan.profile_directory.join(".qiongli-skill-language.json"),
        "components": "22 Skills and native Full MCP; loaded by DSH on a new session",
        "approvals_required": ["client-config-change", "host-trust"],
        "plan_digest_sha256": format!("{:x}", Sha256::digest(serde_json_canonicalizer::to_vec(&plan).map_err(|_| "installation-preview-invalid")?))
    }).to_string())?;
    line(
        writer,
        "DSH will install/update this exact npm version and register its bundle in the selected profile. Existing sessions need a restart.\n",
    )?;
    let reviewed_at = Instant::now();
    if !confirm(
        reader,
        writer,
        "Trust this Plugin and let the official DSH manager install it? [y/N] ",
    )? {
        line(
            writer,
            "Cancelled; no DSH installation commands were run.\n",
        )?;
        return Ok(false);
    }
    let mut current = prepare(&plan.executable, root, &plan.profile)?;
    current.skill_language.clone_from(&plan.skill_language);
    if reviewed_at.elapsed() > Duration::from_secs(600) || current != plan {
        return Err("local-host-precondition-changed");
    }
    line(writer, "Installing through the official DSH manager…\n")?;
    for args in &plan.commands {
        run(environment, &plan.executable, args)?;
    }
    verify_installed(&plan.profile_directory, content)?;
    if let Some(language) = &plan.skill_language {
        write_language(&plan, language)?;
    }
    line(
        writer,
        &format!(
            "DeepSeek Harness: qiongli {} installed and registered in profile {}.\nProfile: {}\nSession tools: not checked. Start a new DSH session, check 22 Skills/33 Full MCP tools and call qiongli_config_status.\n",
            env!("CARGO_PKG_VERSION"),
            plan.profile,
            plan.profile_directory.display()
        ),
    )?;
    Ok(true)
}

fn prepare(executable: &Path, root: &Path, profile: &str) -> Result<InstallPlan, &'static str> {
    if !root.is_absolute()
        || profile.is_empty()
        || profile.len() > 64
        || !profile
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("deepseek-profile-invalid");
    }
    let directory = root.join("profiles").join(profile);
    let mut target = directory.as_path();
    while target.parent().is_some_and(|parent| !parent.exists()) {
        target = target.parent().ok_or("deepseek-profile-invalid")?;
    }
    qiongli_content::approve_materialization_target(target)
        .map_err(|_| "deepseek-profile-unsafe")?;
    let metadata = read_optional(&directory.join("package.json"))?;
    if profile == "desktop" && metadata.is_none() {
        return Err("deepseek-desktop-profile-unavailable");
    }
    if let Some(bytes) = &metadata {
        let metadata: Value =
            serde_json::from_slice(bytes).map_err(|_| "deepseek-profile-invalid")?;
        let bundles = metadata["dsh"]["profile"]["bundles"]
            .as_array()
            .ok_or("deepseek-profile-invalid")?;
        if bundles.iter().any(|value| {
            value
                .as_str()
                .is_some_and(|name| name.starts_with("dsh-qiongli-"))
        }) {
            return Err("deepseek-plugin-conflict");
        }
    }
    if let Some(bytes) = read_optional(&directory.join(".qiongli-skill-language.json"))? {
        let preference: Value =
            serde_json::from_slice(&bytes).map_err(|_| "skill-language-invalid")?;
        if !preference.as_object().is_some_and(|o| o.len() == 1)
            || !preference["language"]
                .as_str()
                .is_some_and(qiongli_content::skill_language_valid)
        {
            return Err("skill-language-invalid");
        }
    }
    let profile_files = [
        "package.json",
        ".qiongli-skill-language.json",
        "pnpm-lock.yaml",
        "pnpm-workspace.yaml",
        "cordis.yml",
        "cordis.patch.yml",
    ]
    .into_iter()
    .map(|name| {
        Ok((
            name.into(),
            read_optional(&directory.join(name))?
                .map(|bytes| format!("{:x}", Sha256::digest(bytes))),
        ))
    })
    .collect::<Result<_, &'static str>>()?;
    let mut commands = Vec::new();
    if metadata.is_none() {
        commands.push(vec![
            "--profile".into(),
            profile.into(),
            "--from-default-profile".into(),
            "web".into(),
            "--dump-config".into(),
        ]);
    }
    commands.push(vec![
        "plugin".into(),
        "--profile".into(),
        profile.into(),
        "add".into(),
        format!("qiongli@{}", env!("CARGO_PKG_VERSION")),
        "--registry".into(),
        "https://registry.npmjs.org".into(),
        "--ignore-scripts".into(),
    ]);
    Ok(InstallPlan {
        executable: executable.into(),
        executable_sha256: crate::cli_install::regular_file_sha256(executable)?,
        profile: profile.into(),
        profile_directory: directory,
        profile_files,
        commands,
        skill_language: None,
    })
}

fn write_language(plan: &InstallPlan, language: &str) -> Result<(), &'static str> {
    if !qiongli_content::skill_language_valid(language) {
        return Err("skill-language-invalid");
    }
    let path = plan.profile_directory.join(".qiongli-skill-language.json");
    qiongli_content::approve_materialization_target(&plan.profile_directory)
        .map_err(|_| "deepseek-profile-unsafe")?;
    let lock_path = plan.profile_directory.join(".qiongli-skill-language.lock");
    let lock = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
        .map_err(|_| "local-host-precondition-changed")?;
    let result = (|| {
        let expected = plan
            .profile_files
            .iter()
            .find(|(name, _)| name == ".qiongli-skill-language.json")
            .ok_or("deepseek-profile-invalid")?
            .1
            .as_deref();
        let digest = read_optional(&path)?.map(|bytes| format!("{:x}", Sha256::digest(bytes)));
        if digest.as_deref() != expected {
            return Err("local-host-precondition-changed");
        }
        let temporary = plan
            .profile_directory
            .join(format!(".qiongli-language-{}.tmp", std::process::id()));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| "deepseek-profile-unavailable")?;
        let result = (|| {
            use std::io::Write as _;
            file.write_all(
                serde_json::json!({"language": language})
                    .to_string()
                    .as_bytes(),
            )
            .map_err(|_| "deepseek-profile-unavailable")?;
            file.sync_all()
                .map_err(|_| "deepseek-profile-unavailable")?;
            // Recheck after staging; stale approved preferences refuse replacement.
            if read_optional(&path)?
                .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
                .as_deref()
                != expected
            {
                return Err("local-host-precondition-changed");
            }
            fs::rename(&temporary, &path).map_err(|_| "deepseek-profile-unavailable")
        })();
        drop(file);
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result
    })();
    drop(lock);
    let cleanup = fs::remove_file(lock_path).map_err(|_| "deepseek-profile-unavailable");
    result.and(cleanup)
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, &'static str> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("deepseek-profile-unavailable"),
    };
    if !metadata.is_file() || metadata.len() > 1024 * 1024 {
        return Err("deepseek-profile-unsafe");
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|_| "deepseek-profile-unavailable")?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "deepseek-profile-unavailable")?;
    if bytes.len() > 1024 * 1024 {
        return Err("deepseek-profile-unsafe");
    }
    Ok(Some(bytes))
}

fn read_json(path: &Path) -> Result<Value, &'static str> {
    serde_json::from_slice(&read_optional(path)?.ok_or("deepseek-install-not-verified")?)
        .map_err(|_| "deepseek-install-not-verified")
}

fn verify_installed(directory: &Path, content: &EmbeddedContent) -> Result<(), &'static str> {
    let profile = read_json(&directory.join("package.json"))?;
    if !profile["dsh"]["profile"]["bundles"]
        .as_array()
        .is_some_and(|bundles| bundles.iter().any(|name| name == "qiongli"))
    {
        return Err("deepseek-install-not-verified");
    }
    let package = directory.join("node_modules/qiongli");
    let metadata = read_json(&package.join("package.json"))?;
    let receipt = read_json(&package.join("dsh/.qiongli-marketplace.json"))?;
    if metadata["name"] != "qiongli"
        || metadata["version"] != env!("CARGO_PKG_VERSION")
        || metadata["main"] != "dsh/index.mjs"
        || metadata["dsh"]["bundle"]["patch"] != "./dsh/cordis.patch.yml"
        || receipt["source"]["pack_sha256"] != content.pack().pack_sha256()
    {
        return Err("deepseek-install-not-verified");
    }
    Ok(())
}

fn run(
    environment: &CommandEnvironment,
    executable: &Path,
    args: &[String],
) -> Result<String, &'static str> {
    crate::desktop::bounded_host_os_command_with_timeout(
        environment,
        executable,
        &args
            .iter()
            .map(std::ffi::OsString::from)
            .collect::<Vec<_>>(),
        Duration::from_secs(120),
    )
    .map_err(|error| error.reason_code())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    #[test]
    fn official_installation_requires_current_approval_and_verified_registration() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/deepseek-install-review");
        fs::create_dir_all(&root).unwrap();
        let home = root
            .canonicalize()
            .unwrap()
            .join(std::process::id().to_string());
        fs::create_dir(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home.clone()), None);
        let content = crate::embedded_content().unwrap();
        let dsh_root = home.join(".dsh");
        let executable = home.join("dsh");
        let directory = dsh_root.join("profiles/probe");
        let package = home.join("payload");
        fs::create_dir_all(package.join("dsh")).unwrap();
        fs::write(package.join("package.json"), serde_json::json!({"name":"qiongli", "version":env!("CARGO_PKG_VERSION"), "main":"dsh/index.mjs", "dsh":{"bundle":{"patch":"./dsh/cordis.patch.yml"}}}).to_string()).unwrap();
        fs::write(
            package.join("dsh/.qiongli-marketplace.json"),
            serde_json::json!({"source":{"pack_sha256":content.pack().pack_sha256()}}).to_string(),
        )
        .unwrap();
        fs::write(&executable, "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$HOME/calls\"\nif [ \"$1\" = plugin ]; then\n mkdir -p \"$HOME/.dsh/profiles/probe/node_modules\"\n printf '%s' '{\"dsh\":{\"profile\":{\"bundles\":[\"qiongli\"]}}}' > \"$HOME/.dsh/profiles/probe/package.json\"\n ln -s \"$HOME/payload\" \"$HOME/.dsh/profiles/probe/node_modules/qiongli\"\nfi\n").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        for response in ["\n", "n\n", "y", ""] {
            let plan = prepare(&executable, &dsh_root, "probe").unwrap();
            assert!(
                !review(
                    &environment,
                    &content,
                    &dsh_root,
                    plan,
                    &mut response.as_bytes(),
                    &mut Vec::new()
                )
                .unwrap()
            );
            assert!(!home.join("calls").exists());
            assert!(!dsh_root.exists());
        }
        for profile in ["../desktop", "a/b", "", "bad name"] {
            assert!(prepare(&executable, &dsh_root, profile).is_err());
        }
        assert!(prepare(&executable, &dsh_root, "desktop").is_err());
        let stale = prepare(&executable, &dsh_root, "probe").unwrap();
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("cordis.patch.yml"), "[]").unwrap();
        assert_eq!(
            review(
                &environment,
                &content,
                &dsh_root,
                stale,
                &mut "y\n".as_bytes(),
                &mut Vec::new()
            )
            .unwrap_err(),
            "local-host-precondition-changed"
        );
        assert!(!home.join("calls").exists());
        fs::write(
            directory.join("package.json"),
            "{\"dsh\":{\"profile\":{\"bundles\":[\"dsh-qiongli-macos-arm64\"]}}}",
        )
        .unwrap();
        assert!(matches!(
            prepare(&executable, &dsh_root, "probe"),
            Err("deepseek-plugin-conflict")
        ));
        fs::remove_file(directory.join("package.json")).unwrap();
        symlink(
            home.join("payload/package.json"),
            directory.join("package.json"),
        )
        .unwrap();
        assert!(matches!(
            prepare(&executable, &dsh_root, "probe"),
            Err("deepseek-profile-unsafe")
        ));
        fs::remove_file(directory.join("package.json")).unwrap();
        let plan = prepare(&executable, &dsh_root, "probe").unwrap();
        assert_eq!(plan.commands.len(), 2);
        assert!(
            review(
                &environment,
                &content,
                &dsh_root,
                plan,
                &mut "yes\n".as_bytes(),
                &mut Vec::new()
            )
            .unwrap()
        );
        let calls = fs::read_to_string(home.join("calls")).unwrap();
        assert_eq!(
            calls.lines().collect::<Vec<_>>(),
            vec![
                "--profile probe --from-default-profile web --dump-config".to_owned(),
                format!(
                    "plugin --profile probe add qiongli@{} --registry https://registry.npmjs.org --ignore-scripts",
                    env!("CARGO_PKG_VERSION")
                )
            ]
        );
        assert_eq!(
            prepare(&executable, &dsh_root, "probe")
                .unwrap()
                .commands
                .len(),
            1
        );
        let plan = prepare(&executable, &dsh_root, "probe").unwrap();
        write_language(&plan, "zh").unwrap();
        assert_eq!(
            read_json(&directory.join(".qiongli-skill-language.json")).unwrap()["language"],
            "zh"
        );
        assert_eq!(
            write_language(&plan, "en").unwrap_err(),
            "local-host-precondition-changed"
        );
        let plan = prepare(&executable, &dsh_root, "probe").unwrap();
        write_language(&plan, "en").unwrap();
        fs::write(directory.join(".qiongli-skill-language.json"), "{}").unwrap();
        assert_eq!(
            prepare(&executable, &dsh_root, "probe").err(),
            Some("skill-language-invalid")
        );
        fs::remove_file(directory.join(".qiongli-skill-language.json")).unwrap();
        fs::write(package.join("dsh/.qiongli-marketplace.json"), "{}").unwrap();
        assert_eq!(
            verify_installed(&directory, &content).unwrap_err(),
            "deepseek-install-not-verified"
        );
        fs::write(&executable, "#!/bin/sh\nexit 1\n").unwrap();
        assert_eq!(
            review(
                &environment,
                &content,
                &dsh_root,
                prepare(&executable, &dsh_root, "probe").unwrap(),
                &mut "y\n".as_bytes(),
                &mut Vec::new()
            )
            .unwrap_err(),
            "host-command-nonzero-exit"
        );
        fs::remove_dir_all(home).unwrap();
    }
}
