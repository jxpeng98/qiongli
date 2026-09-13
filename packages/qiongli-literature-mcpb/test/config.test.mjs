import test from "node:test";
import assert from "node:assert/strict";
import {
  chmodSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync
} from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  atomicWriteProviderConfig,
  assertSecureProviderConfigWritesSupported,
  isFullyQualifiedConfigHomePath,
  platformUserHome,
  providerConfigPath,
  providerStatus,
  readConfig,
  sameFileIdentity,
  saveProviderValue,
  secureProviderConfigWritesSupported
} from "../server/config.mjs";

test("provider config path expands the platform home and never defaults to cwd", (t) => {
  const home = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-config-home-"));
  t.after(() => rmSync(home, { recursive: true, force: true }));
  const homeEnv = process.platform === "win32" ? { USERPROFILE: home } : { HOME: home };

  assert.equal(
    providerConfigPath({ ...homeEnv, QIONGLI_CONFIG_HOME: "~/shared-config" }),
    path.join(home, "shared-config", "providers.json")
  );
  assert.equal(
    providerConfigPath({ ...homeEnv, QIONGLI_CONFIG_HOME: "~" }),
    path.join(home, "providers.json")
  );
  const defaultPath = providerConfigPath(homeEnv);
  assert.equal(defaultPath, path.join(home, ".config", "qiongli", "providers.json"));
  assert.notEqual(path.dirname(defaultPath), process.cwd());
});

test("Windows config paths require a drive and root or a UNC share", () => {
  for (const value of ["C:\\config", "C:/config", "\\\\server\\share\\config"]) {
    assert.equal(isFullyQualifiedConfigHomePath(value, path.win32), true, value);
  }
  for (const value of ["\\config", "/config", "C:relative", "relative"]) {
    assert.equal(isFullyQualifiedConfigHomePath(value, path.win32), false, value);
  }
  assert.equal(isFullyQualifiedConfigHomePath("/config", path.posix), true);
  assert.equal(isFullyQualifiedConfigHomePath("relative", path.posix), false);
});

test("Windows rooted-only USERPROFILE fails with a fixed redacted error", () => {
  const userProfile = "\\profile-path-canary";
  assert.throws(
    () => platformUserHome({ USERPROFILE: userProfile }, path.win32, "C:\\fallback"),
    (error) => {
      assert.equal(
        error.message,
        "platform user home directory must be a fully qualified absolute path"
      );
      assert.equal(error.message.includes(userProfile), false);
      return true;
    }
  );
});

test("relative provider config home fails closed without cwd writes or path leaks", (t) => {
  const expectedError =
    "QIONGLI_CONFIG_HOME must be a fully qualified absolute path or use '~' home notation";
  const relativeHome = `.qiongli-relative-config-canary-${process.pid}-${Date.now()}`;
  const attemptedPath = path.join(process.cwd(), relativeHome);
  const home = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-invalid-config-home-"));
  const homeEnv = process.platform === "win32" ? { USERPROFILE: home } : { HOME: home };
  const invalidHomes = [
    relativeHome,
    "~//abs",
    "~/\\abs",
    "~/C:\\abs",
    "~/C:relative"
  ];
  const secret = "provider-secret-canary";
  t.after(() => {
    rmSync(attemptedPath, { recursive: true, force: true });
    rmSync(home, { recursive: true, force: true });
  });

  assert.equal(existsSync(attemptedPath), false);
  for (const invalidHome of invalidHomes) {
    const env = { ...homeEnv, QIONGLI_CONFIG_HOME: invalidHome };
    for (const [action, actionError] of [
      [() => providerConfigPath(env), expectedError],
      [() => readConfig(env), expectedError],
      [
        () => saveProviderValue({ provider: "openalex", field: "api-key", value: secret, env }),
        process.platform === "win32"
          ? "provider configuration could not be saved"
          : expectedError
      ]
    ]) {
      assert.throws(action, (error) => {
        assert.equal(error.message, actionError);
        for (const canary of invalidHomes) {
          assert.equal(error.message.includes(canary), false);
        }
        assert.equal(error.message.includes(secret), false);
        assert.equal(error.message.includes(process.cwd()), false);
        return true;
      });
    }
  }
  assert.equal(existsSync(attemptedPath), false);
  assert.deepEqual(readdirSync(home), []);
});

test("provider status redacts configured secrets", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-status-config-"));
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  const config = readConfig({
    QIONGLI_CONFIG_HOME: configHome,
    QIONGLI_MCPB_OPENALEX_API_KEY: " openalex-secret-key ",
    QIONGLI_MCPB_OPENALEX_EMAIL: " person@example.com ",
    QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: " secret-key ",
    QIONGLI_MCPB_CROSSREF_EMAIL: " crossref@example.com ",
    QIONGLI_MCPB_PUBMED_API_KEY: " pubmed-secret-key ",
    QIONGLI_MCPB_DEFAULT_LIMIT: "12"
  });

  const status = providerStatus(config);
  const serialized = JSON.stringify(status);

  assert.equal(config.openalexApiKey, "openalex-secret-key");
  assert.equal(config.openalexEmail, "person@example.com");
  assert.equal(config.semanticScholarApiKey, "secret-key");
  assert.equal(config.crossrefEmail, "crossref@example.com");
  assert.equal(config.pubmedApiKey, "pubmed-secret-key");
  assert.equal(config.defaultLimit, 12);
  assert.equal(status.status, "ok");
  assert.equal(status.capability_mode, "provider_connected");
  assert.equal(status.providers.openalex, "configured");
  assert.equal(status.providers.semantic_scholar, "configured");
  assert.equal(status.providers.crossref, "configured");
  assert.equal(status.providers.pubmed, "configured");
  assert.equal(status.providers.arxiv, "configured");
  assert.equal(serialized.includes("secret-key"), false);
  assert.equal(serialized.includes("pubmed-secret-key"), false);
  assert.equal(serialized.includes("openalex-secret-key"), false);
  assert.equal(serialized.includes("person@example.com"), false);
  assert.equal(serialized.includes("crossref@example.com"), false);
});

test("readConfig defaults invalid and blank limits and clamps numeric limits", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-limit-config-"));
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  const env = { QIONGLI_CONFIG_HOME: configHome };

  assert.equal(readConfig({ ...env, QIONGLI_MCPB_DEFAULT_LIMIT: "" }).defaultLimit, 25);
  assert.equal(readConfig({ ...env, QIONGLI_MCPB_DEFAULT_LIMIT: "invalid" }).defaultLimit, 25);
  assert.equal(readConfig({ ...env, QIONGLI_MCPB_DEFAULT_LIMIT: "0" }).defaultLimit, 1);
  assert.equal(readConfig({ ...env, QIONGLI_MCPB_DEFAULT_LIMIT: "51" }).defaultLimit, 50);
  assert.equal(readConfig({ ...env, QIONGLI_MCPB_DEFAULT_LIMIT: "12" }).defaultLimit, 12);
});

test("provider status keeps arXiv available without provider credentials", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-empty-config-"));
  t.after(() => rmSync(configHome, { recursive: true, force: true }));

  const status = providerStatus(readConfig({ QIONGLI_CONFIG_HOME: configHome }));

  assert.equal(status.capability_mode, "provider_connected");
  assert.deepEqual(status.providers, {
    openalex: "missing",
    semantic_scholar: "missing",
    crossref: "missing",
    pubmed: "missing",
    arxiv: "configured"
  });
  assert.deepEqual(status.missing, [
    "openalex.api_key",
    "semantic_scholar.api_key",
    "crossref.email",
    "pubmed.api_key"
  ]);
});

test("readConfig reads Crossref and PubMed values from shared config", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-provider-config-"));
  t.after(() => rmSync(configHome, { recursive: true, force: true }));

  const configPath = path.join(configHome, "providers.json");
  mkdirSync(path.dirname(configPath), { recursive: true });
  writeFileSync(
    configPath,
    `${JSON.stringify({
      providers: {
        crossref: {
          enabled: true,
          email: "stored-crossref@example.com"
        },
        pubmed: {
          enabled: true,
          api_key: "stored-pubmed-key"
        }
      }
    })}\n`
  );

  const config = readConfig({ QIONGLI_CONFIG_HOME: configHome });

  assert.equal(config.crossrefEmail, "stored-crossref@example.com");
  assert.equal(config.pubmedApiKey, "stored-pubmed-key");
});

test("malformed shared config fails closed and save never overwrites it", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-malformed-config-"));
  const configPath = path.join(configHome, "providers.json");
  const env = { QIONGLI_CONFIG_HOME: configHome };
  const secret = "credential-canary";
  t.after(() => rmSync(configHome, { recursive: true, force: true }));

  const malformedPayloads = [
    `{not-json ${secret}`,
    "[]",
    '{"version":0}',
    '{"version":2}',
    '{"version":true}',
    '{"providers":[]}',
    '{"providers":{"openalex":[]}}',
    `{"providers":{"openalex":{"enabled":"${secret}"}}}`,
    `{"providers":{"openalex":{"api_key":{"secret":"${secret}"}}}}`,
    '{"providers":{"semantic-scholar":{"api_key":"first"},"semantic_scholar":{"api_key":"second"}}}',
    '{"providers":{"openalex":{"api-key":"first","api_key":"second"}}}',
    '{"search":[]}',
    '{"search":{"minimum_productive_providers":true}}',
    '{"search":{"minimum_productive_providers":0}}',
    '{"search":{"minimum-productive-providers":2,"minimum_productive_providers":3}}',
    '{"search":{"allow_platform_search_supplement":1}}'
  ];

  for (const payload of malformedPayloads) {
    writeFileSync(configPath, payload);
    const original = readFileSync(configPath);

    assert.throws(() => readConfig(env), (error) => {
      assert.equal(error.message, "provider configuration is unavailable");
      assert.equal(error.message.includes(configHome), false);
      assert.equal(error.message.includes(secret), false);
      return true;
    });
    assert.throws(
      () => saveProviderValue({ provider: "crossref", field: "email", value: secret, env }),
      (error) => {
        assert.equal(error.message, "provider configuration could not be saved");
        assert.equal(error.message.includes(configHome), false);
        assert.equal(error.message.includes(secret), false);
        return true;
      }
    );

    assert.deepEqual(readFileSync(configPath), original);
    assert.deepEqual(readdirSync(configHome), ["providers.json"]);
  }
});

test("saving canonicalizes known aliases while preserving future extensions", (t) => {
  if (process.platform === "win32") {
    t.skip("Legacy Node writes fail closed on Windows; canonicalization is covered on POSIX");
    return;
  }
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-alias-config-"));
  const configPath = path.join(configHome, "providers.json");
  const env = { QIONGLI_CONFIG_HOME: configHome };
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  writeFileSync(
    configPath,
    JSON.stringify({
      providers: {
        "semantic-scholar": {
          enabled: false,
          "api-key": "legacy-key",
          "future-field": { keep: true }
        },
        "future-provider": { keep: true }
      },
      search: {
        "minimum-productive-providers": 2,
        "future-setting": { keep: true }
      }
    })
  );

  saveProviderValue({ provider: "s2", field: "api-key", value: "replacement-key", env });
  const payload = JSON.parse(readFileSync(configPath, "utf8"));

  assert.equal(payload.version, 1);
  assert.equal(payload.providers["semantic-scholar"], undefined);
  assert.equal(payload.providers.semantic_scholar.enabled, true);
  assert.equal(payload.providers.semantic_scholar["api-key"], undefined);
  assert.equal(payload.providers.semantic_scholar.api_key, "replacement-key");
  assert.deepEqual(payload.providers.semantic_scholar["future-field"], { keep: true });
  assert.deepEqual(payload.providers["future-provider"], { keep: true });
  assert.equal(payload.search["minimum-productive-providers"], undefined);
  assert.equal(payload.search.minimum_productive_providers, 2);
  assert.deepEqual(payload.search["future-setting"], { keep: true });
  assert.deepEqual(readdirSync(configHome), ["providers.json"]);
});

test("provider and field labels cannot resolve through object prototypes", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-label-config-"));
  const env = { QIONGLI_CONFIG_HOME: configHome };
  t.after(() => rmSync(configHome, { recursive: true, force: true }));

  for (const [provider, field] of [
    ["constructor", "prototype"],
    ["openalex", "constructor"],
    ["__proto__", "api_key"]
  ]) {
    assert.throws(
      () => saveProviderValue({ provider, field, value: "secret-canary", env }),
      { message: "unsupported provider field" }
    );
  }
  assert.deepEqual(readdirSync(configHome), []);
});

test("persisted enabled false disables configured providers including arXiv", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-disabled-config-"));
  const configPath = path.join(configHome, "providers.json");
  const env = { QIONGLI_CONFIG_HOME: configHome };
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  writeFileSync(
    configPath,
    JSON.stringify({
      version: 1,
      providers: {
        openalex: { enabled: false, api_key: "stored-key" },
        arxiv: { enabled: false }
      }
    })
  );

  const config = readConfig(env);
  const status = providerStatus(config);

  assert.equal(config.providerStates.openalex.enabled, false);
  assert.equal(config.providerStates.openalex.configured, true);
  assert.equal(config.providerStates.arxiv.enabled, false);
  assert.equal(config.providerStates.arxiv.configured, true);
  assert.equal(status.providers.openalex, "configured");
  assert.equal(status.providers.arxiv, "configured");
  assert.deepEqual(status.active_providers, []);
  assert.equal(status.capability_mode, "strategy_only");
});

test("canonical environment aliases explicitly enable a persisted disabled provider", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-env-config-"));
  const configPath = path.join(configHome, "providers.json");
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  writeFileSync(
    configPath,
    JSON.stringify({
      version: 1,
      providers: { semantic_scholar: { enabled: false, api_key: "stored-key" } }
    })
  );

  const config = readConfig({
    QIONGLI_CONFIG_HOME: configHome,
    S2_API_KEY: "environment-key"
  });
  const status = providerStatus(config);

  assert.equal(config.semanticScholarApiKey, "environment-key");
  assert.equal(config.providerStates.semantic_scholar.enabled, true);
  assert.equal(config.providerStates.semantic_scholar.configured, true);
  assert.equal(status.active_providers.includes("semantic_scholar"), true);
});

test("optional environment fields do not reactivate a persisted disabled provider", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-optional-env-config-"));
  const configPath = path.join(configHome, "providers.json");
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  writeFileSync(
    configPath,
    JSON.stringify({
      version: 1,
      providers: {
        openalex: {
          enabled: false,
          api_key: "stored-key",
          email: "stored@example.invalid"
        }
      }
    })
  );

  const config = readConfig({
    QIONGLI_CONFIG_HOME: configHome,
    OPENALEX_EMAIL: "environment@example.invalid"
  });
  const status = providerStatus(config);

  assert.equal(config.openalexApiKey, "stored-key");
  assert.equal(config.openalexEmail, "environment@example.invalid");
  assert.equal(config.providerStates.openalex.configured, true);
  assert.equal(config.providerStates.openalex.enabled, false);
  assert.equal(status.active_providers.includes("openalex"), false);
});

test("existing and replacement config files use owner-only permissions on POSIX", (t) => {
  if (process.platform === "win32") {
    t.skip("POSIX permission bits are not portable to Windows ACLs");
    return;
  }
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-mode-config-"));
  const configPath = path.join(configHome, "providers.json");
  const env = { QIONGLI_CONFIG_HOME: configHome };
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  writeFileSync(configPath, '{"version":1,"providers":{}}\n', { mode: 0o644 });
  chmodSync(configPath, 0o644);

  readConfig(env);
  assert.equal(statSync(configPath).mode & 0o777, 0o600);

  saveProviderValue({ provider: "crossref", field: "email", value: "person@example.com", env });
  assert.equal(statSync(configPath).mode & 0o777, 0o600);
  assert.deepEqual(readdirSync(configHome), ["providers.json"]);
});

test("unreadable config fails closed without overwriting its contents", (t) => {
  if (process.platform === "win32" || process.getuid?.() === 0) {
    t.skip("permission denial is not deterministic for this account");
    return;
  }
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-unreadable-config-"));
  const configPath = path.join(configHome, "providers.json");
  const env = { QIONGLI_CONFIG_HOME: configHome };
  const original = '{"version":1,"providers":{"arxiv":{"enabled":false}}}\n';
  t.after(() => {
    try {
      chmodSync(configPath, 0o600);
    } catch {
      // The path may already have been removed by a failed setup.
    }
    rmSync(configHome, { recursive: true, force: true });
  });
  writeFileSync(configPath, original, { mode: 0o600 });
  chmodSync(configPath, 0o000);

  assert.throws(() => readConfig(env), { message: "provider configuration is unavailable" });
  assert.throws(
    () => saveProviderValue({ provider: "crossref", field: "email", value: "secret", env }),
    { message: "provider configuration could not be saved" }
  );
  chmodSync(configPath, 0o600);
  assert.equal(readFileSync(configPath, "utf8"), original);
});

test("symlink and non-regular config targets fail closed without path disclosure", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-unsafe-config-"));
  const configPath = path.join(configHome, "providers.json");
  const env = { QIONGLI_CONFIG_HOME: configHome };
  const secret = "target-secret-canary";
  t.after(() => rmSync(configHome, { recursive: true, force: true }));

  if (process.platform !== "win32") {
    const targetPath = path.join(configHome, "target.json");
    writeFileSync(targetPath, secret);
    symlinkSync(targetPath, configPath);

    for (const action of [
      () => readConfig(env),
      () => saveProviderValue({ provider: "crossref", field: "email", value: secret, env })
    ]) {
      assert.throws(action, (error) => {
        assert.equal(error.message.includes(configHome), false);
        assert.equal(error.message.includes(secret), false);
        return true;
      });
    }
    assert.equal(readFileSync(targetPath, "utf8"), secret);
    rmSync(configPath);
    rmSync(targetPath);
  }

  mkdirSync(configPath);
  for (const action of [
    () => readConfig(env),
    () => saveProviderValue({ provider: "crossref", field: "email", value: secret, env })
  ]) {
    assert.throws(action, (error) => {
      assert.equal(error.message.includes(configHome), false);
      assert.equal(error.message.includes(secret), false);
      return true;
    });
  }
  assert.equal(lstatSync(configPath).isDirectory(), true);
});

test("Windows junction config target is rejected before reading its contents", (t) => {
  if (process.platform !== "win32") {
    t.skip("Windows junction behavior requires a Windows filesystem");
    return;
  }
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-junction-config-"));
  const targetHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-junction-target-"));
  const configPath = path.join(configHome, "providers.json");
  const targetPayload = path.join(targetHome, "credential-canary.json");
  const secret = "junction-secret-canary";
  t.after(() => {
    rmSync(configHome, { recursive: true, force: true });
    rmSync(targetHome, { recursive: true, force: true });
  });
  writeFileSync(targetPayload, secret);
  symlinkSync(targetHome, configPath, "junction");

  assert.equal(lstatSync(configPath).isSymbolicLink(), true);
  assert.throws(
    () => readConfig({ QIONGLI_CONFIG_HOME: configHome }),
    (error) => {
      assert.equal(error.message, "provider configuration is unavailable");
      assert.equal(error.message.includes(configHome), false);
      assert.equal(error.message.includes(targetHome), false);
      assert.equal(error.message.includes(secret), false);
      return true;
    }
  );
  assert.equal(readFileSync(targetPayload, "utf8"), secret);
});

test("opened config identity must match the path that was inspected", () => {
  assert.equal(sameFileIdentity({ dev: 1, ino: 2 }, { dev: 1, ino: 2 }, false), true);
  assert.equal(sameFileIdentity({ dev: 1, ino: 2 }, { dev: 1, ino: 3 }, false), false);
  assert.equal(sameFileIdentity({ dev: 1, ino: 2 }, { dev: 2, ino: 2 }, false), false);
  assert.equal(sameFileIdentity({ dev: 0, ino: 0 }, { dev: 0, ino: 0 }, true), false);
});

test("atomic replacement failure preserves original bytes and removes temporary files", (t) => {
  if (process.platform === "win32") {
    t.skip("Legacy Node writes fail before replacement on Windows");
    return;
  }
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-atomic-failure-"));
  const configPath = path.join(configHome, "providers.json");
  const original = '{"version":1,"providers":{"arxiv":{"enabled":false}}}\n';
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  writeFileSync(configPath, original, { mode: 0o600 });

  assert.throws(
    () => atomicWriteProviderConfig(
      configPath,
      '{"version":1,"providers":{}}\n',
      () => {
        throw new Error("injected replacement failure");
      }
    ),
    /injected replacement failure/
  );

  assert.equal(readFileSync(configPath, "utf8"), original);
  assert.deepEqual(readdirSync(configHome), ["providers.json"]);
});

test("legacy Node Windows config writes fail closed before replacing files", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-windows-write-limit-"));
  const configPath = path.join(configHome, "providers.json");
  const original = '{"version":1,"providers":{}}\n';
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  writeFileSync(configPath, original, { mode: 0o600 });

  assert.equal(secureProviderConfigWritesSupported("win32"), false);
  assert.throws(
    () => assertSecureProviderConfigWritesSupported("win32"),
    { message: "provider configuration could not be saved" }
  );
  assert.throws(
    () => atomicWriteProviderConfig(configPath, "credential-canary", undefined, "win32"),
    { message: "provider configuration could not be saved" }
  );
  assert.equal(readFileSync(configPath, "utf8"), original);
  assert.deepEqual(readdirSync(configHome), ["providers.json"]);
});

test("actual Windows save entry point cannot downgrade the shared config ACL", (t) => {
  if (process.platform !== "win32") {
    t.skip("Windows-only fail-closed behavior");
    return;
  }
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-windows-save-entry-"));
  const configPath = path.join(configHome, "providers.json");
  const original = '{"version":1,"providers":{}}\n';
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  writeFileSync(configPath, original);

  assert.throws(
    () => saveProviderValue({
      provider: "crossref",
      field: "email",
      value: "credential-canary@example.invalid",
      env: { QIONGLI_CONFIG_HOME: configHome }
    }),
    { message: "provider configuration could not be saved" }
  );
  assert.equal(readFileSync(configPath, "utf8"), original);
  assert.deepEqual(readdirSync(configHome), ["providers.json"]);
});
