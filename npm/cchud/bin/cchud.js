#!/usr/bin/env node
'use strict';

const { spawnSync } = require('child_process');

function detectPackage() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'linux' && arch === 'x64') {
    try {
      const report = process.report.getReport();
      const glibc = report.header.glibcVersionRuntime;
      return glibc ? '@cchud/cli-linux-x64' : '@cchud/cli-linux-x64-musl';
    } catch {
      return '@cchud/cli-linux-x64';
    }
  }
  if (platform === 'darwin' && arch === 'arm64') return '@cchud/cli-darwin-arm64';
  if (platform === 'darwin' && arch === 'x64')   return '@cchud/cli-darwin-x64';
  if (platform === 'win32'  && arch === 'x64')   return '@cchud/cli-win32-x64';

  process.stderr.write(
    `cchud: unsupported platform ${platform}-${arch}.\n` +
    `Supported: darwin-arm64, darwin-x64, linux-x64 (gnu+musl), win32-x64.\n` +
    `See https://github.com/IGoRFonin/cchud/issues to request support.\n`
  );
  process.exit(1);
}

function main() {
  const pkg = process.env.CCHUD_NPM_PACKAGE || detectPackage();
  let binPath;
  try {
    const ext = process.platform === 'win32' ? '.exe' : '';
    binPath = require.resolve(`${pkg}/bin/cchud${ext}`);
  } catch (e) {
    process.stderr.write(
      `cchud: native binary package "${pkg}" not installed.\n` +
      `Try: npm install --include=optional ${pkg}@<version>\n` +
      `Or reinstall: npx --yes cchud@<version> install\n`
    );
    process.exit(1);
  }

  const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: 'inherit',
    windowsHide: true,
  });
  if (result.error) {
    process.stderr.write(`cchud: failed to spawn ${binPath}: ${result.error.message}\n`);
    process.exit(1);
  }
  process.exit(result.status === null ? 1 : result.status);
}

main();
