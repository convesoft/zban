#!/usr/bin/env node

"use strict";

const path = require("node:path");
const { spawn } = require("node:child_process");

const packages = new Map([
  ["linux:x64", "@convesoft/zban-linux-x64"],
  ["linux:arm64", "@convesoft/zban-linux-arm64"],
  ["darwin:arm64", "@convesoft/zban-darwin-arm64"],
  ["darwin:x64", "@convesoft/zban-darwin-x64"],
]);

const packageName = packages.get(`${process.platform}:${process.arch}`);
if (packageName === undefined) {
  console.error(
    `zban does not provide an npm binary for ${process.platform}/${process.arch}. ` +
      "Supported targets are Linux (x64, arm64) and macOS (arm64, x64).",
  );
  process.exitCode = 1;
} else {
  let binary;
  try {
    const manifest = require.resolve(`${packageName}/package.json`);
    binary = path.join(path.dirname(manifest), "bin", "zban");
  } catch {
    console.error(
      `The optional package ${packageName} is missing. ` +
        "Reinstall @convesoft/zban with optional dependencies enabled.",
    );
    process.exitCode = 1;
  }

  if (binary !== undefined) {
    const child = spawn(binary, process.argv.slice(2), { stdio: "inherit" });
    const handlers = new Map();
    for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
      const handler = () => child.kill(signal);
      handlers.set(signal, handler);
      process.on(signal, handler);
    }

    child.once("error", (error) => {
      console.error(`Could not start ${packageName}: ${error.message}`);
      process.exitCode = 1;
    });

    child.once("exit", (code, signal) => {
      for (const [name, handler] of handlers) {
        process.off(name, handler);
      }
      if (signal !== null) {
        process.kill(process.pid, signal);
      } else {
        process.exitCode = code ?? 1;
      }
    });
  }
}
