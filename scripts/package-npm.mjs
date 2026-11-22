#!/usr/bin/env node

import { chmod, copyFile, cp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// Matches ADR-RUST-BINARY and ADR-NPM-DISTRIBUTION. Linux binaries are
// statically linked with musl and run on both glibc and musl distributions, so
// no `libc` restriction is declared.
const targets = new Map([
  [
    "x86_64-unknown-linux-musl",
    { name: "@convesoft/zban-linux-x64", os: ["linux"], cpu: ["x64"] },
  ],
  [
    "aarch64-unknown-linux-musl",
    { name: "@convesoft/zban-linux-arm64", os: ["linux"], cpu: ["arm64"] },
  ],
  [
    "aarch64-apple-darwin",
    { name: "@convesoft/zban-darwin-arm64", os: ["darwin"], cpu: ["arm64"] },
  ],
  [
    "x86_64-apple-darwin",
    { name: "@convesoft/zban-darwin-x64", os: ["darwin"], cpu: ["x64"] },
  ],
]);

const cargo = await readFile(path.join(root, "Cargo.toml"), "utf8");
const packageSection = cargo.match(/\[package\]([\s\S]*?)(?:\n\[|$)/)?.[1] ?? "";
const readField = (key) => packageSection.match(new RegExp(`^${key}\\s*=\\s*"([^"]+)"`, "m"))?.[1];

const version = readField("version");
const description = readField("description");
const license = readField("license");
const repositoryUrl = readField("repository");

if (!version || version === "0.0.0" || !description || !license || !repositoryUrl) {
  throw new Error("Cargo.toml must declare non-zero version, description, license and repository");
}

function manifest(name, pkgDescription) {
  return {
    name,
    version,
    description: pkgDescription,
    license,
    repository: { type: "git", url: `git+${repositoryUrl}.git` },
    homepage: repositoryUrl,
    publishConfig: { access: "public" },
  };
}

async function prepareDir(output, name) {
  const directory = path.join(path.resolve(output), name.replace("@convesoft/", ""));
  await rm(directory, { recursive: true, force: true });
  await mkdir(path.join(directory, "bin"), { recursive: true });
  await Promise.all(
    ["README.md", "LICENSE-MIT", "LICENSE-APACHE"].map((file) =>
      copyFile(path.join(root, file), path.join(directory, file)),
    ),
  );
  return directory;
}

async function writeManifest(directory, value) {
  await writeFile(path.join(directory, "package.json"), `${JSON.stringify(value, null, 2)}\n`);
  process.stdout.write(`${directory}\n`);
}

const [command, ...args] = process.argv.slice(2);

if (command === "main" && args.length === 1) {
  const name = "@convesoft/zban";
  const directory = await prepareDir(args[0], name);
  const binPath = path.join(directory, "bin/zban.cjs");
  await copyFile(path.join(root, "npm/zban.cjs"), binPath);
  await chmod(binPath, 0o755);
  // The documentation of this version ships whole, in the main package only
  // (ADR-NPM-DISTRIBUTION).
  await cp(path.join(root, "docs"), path.join(directory, "docs"), { recursive: true });
  await writeManifest(directory, {
    ...manifest(name, description),
    bin: { zban: "bin/zban.cjs" },
    engines: { node: ">=18" },
    optionalDependencies: Object.fromEntries(
      [...targets.values()].map(({ name: pkgName }) => [pkgName, version]),
    ),
    files: ["bin/zban.cjs", "docs/", "README.md", "LICENSE-MIT", "LICENSE-APACHE"],
  });
} else if (command === "platform" && args.length === 4) {
  const [target, binaryPath, noticesPath, output] = args;
  const config = targets.get(target);
  if (!config) {
    throw new Error(`unsupported Rust target: ${target}`);
  }
  const directory = await prepareDir(output, config.name);
  const binPath = path.join(directory, "bin/zban");
  await copyFile(path.resolve(binaryPath), binPath);
  await chmod(binPath, 0o755);
  await copyFile(path.resolve(noticesPath), path.join(directory, "THIRD-PARTY-LICENSES"));
  await writeManifest(directory, {
    ...manifest(config.name, `zban native binary for ${target}`),
    os: config.os,
    cpu: config.cpu,
    files: ["bin/zban", "README.md", "LICENSE-MIT", "LICENSE-APACHE", "THIRD-PARTY-LICENSES"],
  });
} else {
  console.error(
    "usage: package-npm.mjs main <output-dir> | platform <target> <binary> <third-party-licenses> <output-dir>",
  );
  process.exitCode = 2;
}
