'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const expectedFiles = [
  'DISCLOSURE',
  'LICENSE',
  'README.md',
  'bridge/src/burp-provider.ts',
  'bridge/src/burp-tool-descriptions.ts',
  'bridge/src/burp-tool-schemas.ts',
  'bridge/src/config.ts',
  'bridge/src/cyberchef-bun-compat.mjs',
  'bridge/src/cyberchef-catalog.mjs',
  'bridge/src/cyberchef-engine.mjs',
  'bridge/src/cyberchef-http.ts',
  'bridge/src/cyberchef-magic.ts',
  'bridge/src/cyberchef-provider.ts',
  'bridge/src/cyberchef-runtime.ts',
  'bridge/src/cyberchef-schemas.ts',
  'bridge/src/cyberchef-worker.mjs',
  'bridge/src/json.ts',
  'bridge/src/main.ts',
  'bridge/src/rpc.ts',
  'bridge/src/stdio.ts',
  'bridge/src/tool-directory.ts',
  'bridge/src/types.ts',
  'package.json',
];

function validatePackReport(report, packageJson, artifactDirectory) {
  assert.equal(report.length, 1, 'npm pack must produce exactly one artifact');
  const [artifact] = report;
  assert.equal(artifact.id, `${packageJson.name}@${packageJson.version}`);
  assert.deepEqual(
    artifact.files.map((file) => file.path).sort(),
    expectedFiles,
    'npm artifact contents changed; update the release contract deliberately',
  );
  assert.match(artifact.integrity, /^sha512-/);
  assert.match(artifact.shasum, /^[a-f0-9]{40}$/);
  assert.ok(fs.existsSync(path.join(artifactDirectory, artifact.filename)), 'npm tarball is missing');
  return artifact.filename;
}

if (require.main === module) {
  const [reportPath, artifactDirectory] = process.argv.slice(2);
  assert.ok(reportPath && artifactDirectory, 'usage: validate-pack-report.cjs <report.json> <artifact-dir>');
  const report = JSON.parse(fs.readFileSync(reportPath, 'utf8'));
  const packageJson = JSON.parse(fs.readFileSync(path.join(process.cwd(), 'package.json'), 'utf8'));
  process.stdout.write(`${validatePackReport(report, packageJson, artifactDirectory)}\n`);
}

module.exports = { expectedFiles, validatePackReport };
