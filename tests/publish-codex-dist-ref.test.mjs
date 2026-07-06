import { execFile } from "node:child_process";
import { mkdir, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const scriptPath = path.join(root, "scripts/publish-codex-dist-ref.mjs");

async function git(cwd, args, options = {}) {
  return execFileAsync("git", args, {
    cwd,
    encoding: "utf8",
    ...options
  });
}

async function writeText(filePath, content) {
  await mkdir(path.dirname(filePath), { recursive: true });
  await writeFile(filePath, content);
}

async function createRepo() {
  const tmp = await mkdtemp(path.join(tmpdir(), "qiongli-codex-dist-ref-"));
  const repo = path.join(tmp, "repo");
  const remote = path.join(tmp, "remote.git");

  await mkdir(repo, { recursive: true });
  await git(repo, ["init"]);
  await git(repo, ["config", "user.email", "test@example.com"]);
  await git(repo, ["config", "user.name", "Test User"]);
  await writeText(path.join(repo, "README.md"), "source branch content\n");
  await git(repo, ["add", "README.md"]);
  await git(repo, ["commit", "-m", "initial source"]);

  await git(tmp, ["init", "--bare", remote]);
  await git(repo, ["remote", "add", "origin", remote]);

  return { tmp, repo, remote };
}

async function createPluginSource(tmp, slug, version, extraSkillText = "") {
  const source = path.join(tmp, "source", slug);
  const skillRoot = path.join(source, "skills", "qiongli-workflow");
  const manifest = {
    name: slug,
    version,
    skills: "./skills/",
    mcpServers: "./.mcp.json",
    interface: {
      displayName: slug === "qiongli-next" ? "Qiongli Next" : "Qiongli"
    }
  };

  await writeText(
    path.join(source, ".codex-plugin", "plugin.json"),
    `${JSON.stringify(manifest, null, 2)}\n`
  );
  await writeText(
    path.join(source, ".mcp.json"),
    JSON.stringify(
      {
        mcpServers: {
          [slug]: {
            command: "node",
            args: ["./mcp/qiongli-literature-provider/index.mjs"]
          }
        }
      },
      null,
      2
    ) + "\n"
  );
  await writeText(path.join(source, "mcp", "qiongli-literature-provider", "index.mjs"), "export {};\n");
  await writeText(path.join(source, "mcp", "qiongli-literature-provider", "query.mjs"), "export {};\n");
  await writeText(path.join(source, "commands", "qiongli.md"), `Load the \`${slug}\` skill.\n`);
  await writeText(path.join(source, "commands", "paper.md"), `Load the \`${slug}\` skill.\n`);
  await writeText(
    path.join(skillRoot, "SKILL.md"),
    `---\nname: ${slug}\ndescription: test\n---\n${extraSkillText}\n`
  );
  await writeText(path.join(skillRoot, "VERSION"), `v${version}\n`);
  await writeText(path.join(skillRoot, "skills", "registry.yaml"), `version: "${version}"\n`);

  return source;
}

async function runPublisher(repo, args) {
  return execFileAsync("node", [scriptPath, "--repo", repo, ...args], {
    cwd: root,
    encoding: "utf8"
  });
}

test("publishes a plugin payload to an orphan codex version branch", async () => {
  const { tmp, repo, remote } = await createRepo();
  const source = await createPluginSource(tmp, "qiongli", "1.2.3");

  await runPublisher(repo, [
    "--version",
    "1.2.3",
    "--slug",
    "qiongli",
    "--source",
    source,
    "--remote",
    "origin"
  ]);

  const localManifest = await git(repo, [
    "show",
    "refs/heads/codex/v1.2.3:plugins/qiongli/.codex-plugin/plugin.json"
  ]);
  assert.equal(JSON.parse(localManifest.stdout).name, "qiongli");

  const remoteManifest = await git(remote, [
    "show",
    "refs/heads/codex/v1.2.3:plugins/qiongli/.codex-plugin/plugin.json"
  ]);
  assert.equal(JSON.parse(remoteManifest.stdout).version, "1.2.3");

  await assert.rejects(
    git(repo, ["show", "refs/heads/codex/v1.2.3:README.md"]),
    /exists on disk, but not in/
  );

  const parents = await git(repo, ["rev-list", "--parents", "-n", "1", "refs/heads/codex/v1.2.3"]);
  assert.equal(parents.stdout.trim().split(/\s+/).length, 1);
});

test("rejects a payload whose manifest version does not match the ref version", async () => {
  const { tmp, repo } = await createRepo();
  const source = await createPluginSource(tmp, "qiongli-next", "9.9.9");

  await assert.rejects(
    runPublisher(repo, [
      "--version",
      "1.5.0-beta.1",
      "--slug",
      "qiongli-next",
      "--source",
      source,
      "--no-push"
    ]),
    /manifest version mismatch/
  );

  await assert.rejects(git(repo, ["show-ref", "--verify", "refs/heads/codex/v1.5.0-beta.1"]));
});

test("requires force before replacing an existing different dist ref", async () => {
  const { tmp, repo } = await createRepo();
  const firstSource = await createPluginSource(tmp, "qiongli", "1.2.3", "first payload\n");

  await runPublisher(repo, [
    "--version",
    "1.2.3",
    "--slug",
    "qiongli",
    "--source",
    firstSource,
    "--no-push"
  ]);

  const secondSource = await createPluginSource(tmp, "qiongli", "1.2.3", "second payload\n");
  await assert.rejects(
    runPublisher(repo, [
      "--version",
      "1.2.3",
      "--slug",
      "qiongli",
      "--source",
      secondSource,
      "--no-push"
    ]),
    /already exists with different content/
  );

  await runPublisher(repo, [
    "--version",
    "1.2.3",
    "--slug",
    "qiongli",
    "--source",
    secondSource,
    "--no-push",
    "--force"
  ]);

  const skill = await git(repo, ["show", "refs/heads/codex/v1.2.3:plugins/qiongli/skills/qiongli-workflow/SKILL.md"]);
  assert.match(skill.stdout, /second payload/);
});

test("reuses an existing remote dist ref when rerun from a fresh checkout", async () => {
  const { tmp, repo, remote } = await createRepo();
  const source = await createPluginSource(tmp, "qiongli-next", "1.5.0-beta.1");

  await runPublisher(repo, [
    "--version",
    "1.5.0-beta.1",
    "--slug",
    "qiongli-next",
    "--source",
    source,
    "--remote",
    "origin"
  ]);
  const firstRemoteRef = (await git(remote, ["rev-parse", "refs/heads/codex/v1.5.0-beta.1"])).stdout.trim();

  const freshRepo = path.join(tmp, "fresh-repo");
  await git(tmp, ["clone", remote, freshRepo]);
  await git(freshRepo, ["config", "user.email", "test@example.com"]);
  await git(freshRepo, ["config", "user.name", "Test User"]);
  await assert.rejects(git(freshRepo, ["rev-parse", "--verify", "refs/heads/codex/v1.5.0-beta.1"]));

  await runPublisher(freshRepo, [
    "--version",
    "1.5.0-beta.1",
    "--slug",
    "qiongli-next",
    "--source",
    source,
    "--remote",
    "origin"
  ]);

  const secondRemoteRef = (await git(remote, ["rev-parse", "refs/heads/codex/v1.5.0-beta.1"])).stdout.trim();
  assert.equal(secondRemoteRef, firstRemoteRef);
});
