#!/usr/bin/env node
import { spawn } from "node:child_process";
import { lstat, readFile, readdir, stat } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const ALLOWED_SLUGS = new Set(["qiongli", "qiongli-next"]);
const REQUIRED_FILES = [
  ".codex-plugin/plugin.json",
  ".mcp.json",
  "mcp/qiongli-literature-provider/index.mjs",
  "mcp/qiongli-literature-provider/query.mjs",
  "commands/paper.md",
  "skills/qiongli-workflow/SKILL.md",
  "skills/qiongli-workflow/VERSION",
  "skills/qiongli-workflow/skills/registry.yaml"
];

function usage() {
  return `Usage:
  node scripts/publish-codex-dist-ref.mjs --version <version> --slug <qiongli|qiongli-next> --source <plugin-dir> [options]

Options:
  --repo <dir>       Git repository used to create/update the dist ref (default: cwd)
  --remote <name>    Remote to push to when pushing is enabled (default: origin)
  --force            Replace an existing different dist ref
  --no-push          Update only the local ref
  -h, --help         Show this message`;
}

function parseArgs(argv) {
  const options = {
    repo: process.cwd(),
    remote: "origin",
    force: false,
    push: true,
    version: "",
    slug: "",
    source: ""
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "-h" || arg === "--help") {
      options.help = true;
      continue;
    }
    if (arg === "--force") {
      options.force = true;
      continue;
    }
    if (arg === "--no-push") {
      options.push = false;
      continue;
    }
    if (["--repo", "--remote", "--version", "--slug", "--source"].includes(arg)) {
      const value = argv[index + 1];
      if (!value || value.startsWith("--")) {
        throw new Error(`missing value for ${arg}`);
      }
      options[arg.slice(2)] = value;
      index += 1;
      continue;
    }
    throw new Error(`unknown option: ${arg}`);
  }

  return options;
}

function normalizeVersion(rawVersion) {
  const version = rawVersion.trim().replace(/^v/, "");
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`unsupported version: ${rawVersion}`);
  }
  return version;
}

function normalizeSlug(slug) {
  if (!ALLOWED_SLUGS.has(slug)) {
    throw new Error(`unsupported slug: ${slug}`);
  }
  return slug;
}

async function runGit(repo, args, { input, allowFailure = false, env = {} } = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn("git", args, {
      cwd: repo,
      env: { ...process.env, ...env },
      stdio: ["pipe", "pipe", "pipe"]
    });
    let stdout = "";
    let stderr = "";
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", reject);
    child.on("close", (code) => {
      const result = { code, stdout, stderr };
      if (code === 0 || allowFailure) {
        resolve(result);
        return;
      }
      reject(new Error(`git ${args.join(" ")} failed with exit ${code}\n${stderr}${stdout}`));
    });
    if (input === undefined) {
      child.stdin.end();
    } else {
      child.stdin.end(input);
    }
  });
}

async function readJson(filePath) {
  return JSON.parse(await readFile(filePath, "utf8"));
}

async function assertFile(source, relPath) {
  const filePath = path.join(source, relPath);
  const fileStat = await lstat(filePath);
  if (!fileStat.isFile()) {
    throw new Error(`required file is not a regular file: ${relPath}`);
  }
  return filePath;
}

async function validateSource(source, slug, version) {
  const sourceStat = await lstat(source);
  if (!sourceStat.isDirectory()) {
    throw new Error(`source must be a plugin directory: ${source}`);
  }

  for (const relPath of REQUIRED_FILES) {
    await assertFile(source, relPath);
  }

  const manifest = await readJson(path.join(source, ".codex-plugin", "plugin.json"));
  if (manifest.name !== slug) {
    throw new Error(`manifest name mismatch: expected ${slug}, found ${manifest.name}`);
  }
  if (manifest.version !== version) {
    throw new Error(`manifest version mismatch: expected ${version}, found ${manifest.version}`);
  }
  if (manifest.skills !== "./skills/") {
    throw new Error(`manifest skills must be ./skills/, found ${manifest.skills}`);
  }
  if (manifest.mcpServers !== "./.mcp.json") {
    throw new Error(`manifest mcpServers must be ./.mcp.json, found ${manifest.mcpServers}`);
  }

  const skillVersion = (await readFile(path.join(source, "skills", "qiongli-workflow", "VERSION"), "utf8")).trim();
  if (skillVersion !== `v${version}`) {
    throw new Error(`skill VERSION mismatch: expected v${version}, found ${skillVersion}`);
  }

  const registry = await readFile(
    path.join(source, "skills", "qiongli-workflow", "skills", "registry.yaml"),
    "utf8"
  );
  const registryVersion = new RegExp(`^\\s*version:\\s*["']?${version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}["']?\\s*$`, "m");
  if (!registryVersion.test(registry)) {
    throw new Error(`registry version mismatch: expected ${version}`);
  }
}

function assertSafeTreeName(name) {
  if (!name || name.includes("\n") || name.includes("\t") || name === ".git") {
    throw new Error(`unsafe tree entry name: ${name}`);
  }
}

async function gitTreeForDirectory(repo, directory) {
  const entries = [];
  const dirents = await readdir(directory, { withFileTypes: true });
  dirents.sort((left, right) => left.name.localeCompare(right.name));

  for (const dirent of dirents) {
    assertSafeTreeName(dirent.name);
    const item = path.join(directory, dirent.name);
    if (dirent.isSymbolicLink()) {
      throw new Error(`symlinks are not allowed in Codex dist refs: ${item}`);
    }
    if (dirent.isDirectory()) {
      const tree = await gitTreeForDirectory(repo, item);
      entries.push(`040000 tree ${tree}\t${dirent.name}`);
      continue;
    }
    if (!dirent.isFile()) {
      throw new Error(`unsupported file type in Codex dist ref: ${item}`);
    }
    const itemStat = await stat(item);
    const mode = itemStat.mode & 0o111 ? "100755" : "100644";
    const hash = (await runGit(repo, ["hash-object", "-w", item])).stdout.trim();
    entries.push(`${mode} blob ${hash}\t${dirent.name}`);
  }

  return (await runGit(repo, ["mktree"], { input: `${entries.join("\n")}\n` })).stdout.trim();
}

async function gitTreeWithSingleDirectory(repo, name, treeHash) {
  assertSafeTreeName(name);
  return (await runGit(repo, ["mktree"], { input: `040000 tree ${treeHash}\t${name}\n` })).stdout.trim();
}

async function buildDistRootTree(repo, source, slug) {
  const pluginTree = await gitTreeForDirectory(repo, source);
  const pluginsTree = await gitTreeWithSingleDirectory(repo, slug, pluginTree);
  return gitTreeWithSingleDirectory(repo, "plugins", pluginsTree);
}

async function existingRefTree(repo, fullRef) {
  const refResult = await runGit(repo, ["rev-parse", "--verify", fullRef], { allowFailure: true });
  if (refResult.code !== 0) {
    return null;
  }
  const tree = await runGit(repo, ["show", "-s", "--format=%T", fullRef]);
  return tree.stdout.trim();
}

async function syncRemoteRef(repo, remote, refName) {
  const result = await runGit(repo, ["ls-remote", "--heads", remote, refName], { allowFailure: true });
  if (result.code !== 0 || !result.stdout.trim()) {
    return;
  }
  await runGit(repo, [
    "fetch",
    "--force",
    "--no-tags",
    remote,
    `+refs/heads/${refName}:refs/heads/${refName}`
  ]);
}

async function sourceCommit(repo) {
  const result = await runGit(repo, ["rev-parse", "HEAD"], { allowFailure: true });
  return result.code === 0 ? result.stdout.trim() : "unknown";
}

async function commitTree(repo, tree, refName, slug, version) {
  const commitMessage = [
    `Publish Codex dist ref ${refName} for ${slug}`,
    "",
    `Version: ${version}`,
    `Source: ${await sourceCommit(repo)}`
  ].join("\n");
  return (
    await runGit(repo, ["commit-tree", tree, "-m", commitMessage], {
      env: {
        GIT_AUTHOR_NAME: "qiongli-dist-ref",
        GIT_AUTHOR_EMAIL: "qiongli-dist-ref@example.invalid",
        GIT_COMMITTER_NAME: "qiongli-dist-ref",
        GIT_COMMITTER_EMAIL: "qiongli-dist-ref@example.invalid"
      }
    })
  ).stdout.trim();
}

async function publish(options) {
  const repo = path.resolve(options.repo);
  const source = path.resolve(options.source);
  const slug = normalizeSlug(options.slug);
  const version = normalizeVersion(options.version);
  const refName = `codex/v${version}`;
  const fullRef = `refs/heads/${refName}`;

  await validateSource(source, slug, version);
  if (options.push) {
    await syncRemoteRef(repo, options.remote, refName);
  }
  const newTree = await buildDistRootTree(repo, source, slug);
  const currentTree = await existingRefTree(repo, fullRef);

  if (currentTree && currentTree !== newTree && !options.force) {
    throw new Error(`${refName} already exists with different content; rerun with --force to replace it`);
  }

  let commit = "";
  if (currentTree === newTree) {
    commit = (await runGit(repo, ["rev-parse", "--verify", fullRef])).stdout.trim();
    console.log(`[codex-dist-ref] ${refName} already matches ${slug}@${version}`);
  } else {
    commit = await commitTree(repo, newTree, refName, slug, version);
    await runGit(repo, ["update-ref", "-m", `publish ${refName}`, fullRef, commit]);
    console.log(`[codex-dist-ref] updated ${fullRef} -> ${commit}`);
  }

  if (options.push) {
    const refspec = `${options.force ? "+" : ""}${fullRef}:${fullRef}`;
    await runGit(repo, ["push", options.remote, refspec]);
    console.log(`[codex-dist-ref] pushed ${refName} to ${options.remote}`);
  }

  return { refName, fullRef, commit };
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    console.log(usage());
    return 0;
  }
  for (const required of ["version", "slug", "source"]) {
    if (!options[required]) {
      throw new Error(`missing required option --${required}`);
    }
  }
  await publish(options);
  return 0;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  try {
    process.exitCode = await main();
  } catch (error) {
    console.error(`[codex-dist-ref] ${error.message}`);
    process.exitCode = 1;
  }
}
