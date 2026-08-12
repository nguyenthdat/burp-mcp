'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { test } = require('node:test');

const projectRoot = path.join(__dirname, '..');
const releaseWorkflow = fs.readFileSync(
  path.join(projectRoot, '.github', 'workflows', 'release.yml'),
  'utf8',
);
const readme = fs.readFileSync(path.join(projectRoot, 'README.md'), 'utf8');
const gradleBuild = fs.readFileSync(path.join(projectRoot, 'build.gradle.kts'), 'utf8');

test('stages dual-use releases through an immutable reviewed commit', () => {
  const stageJob = releaseWorkflow.slice(releaseWorkflow.indexOf('  stage-npm:'));
  const stageStepNames = [...stageJob.matchAll(/^\s+- name: (.+)$/gm)].map((match) => match[1]);

  assert.match(releaseWorkflow, /npm stage publish/);
  assert.doesNotMatch(releaseWorkflow, /^\s+npm publish\b/m);
  assert.doesNotMatch(releaseWorkflow, /NPM_TOKEN|NODE_AUTH_TOKEN|_authToken/);
  assert.match(releaseWorkflow, /ref: \$\{\{ github\.sha \}\}/);
  assert.match(releaseWorkflow, /persist-credentials: false/);
  assert.match(releaseWorkflow, /git merge-base --is-ancestor/);
  assert.match(releaseWorkflow, /permissions:\n\s+contents: read\n\s+id-token: write/);
  assert.match(readme, /staged through Trusted Publishing with provenance/);
  assert.match(readme, /2FA approval/);
  assert.match(releaseWorkflow, /GITHUB_STEP_SUMMARY/);
  assert.doesNotMatch(releaseWorkflow, /STAGE_ID/);
  assert.equal(stageStepNames.at(-1), 'Stage npm package with provenance');
});

test('derives the JAR version from package.json and verifies downloadable assets', () => {
  assert.match(gradleBuild, /JsonSlurper\(\)\.parse\(file\("package\.json"\)\)/);
  assert.match(releaseWorkflow, /MANIFEST_VERSION/);
  assert.match(releaseWorkflow, /sha256sum burp-mcp\.jar/);
  assert.doesNotMatch(releaseWorkflow, /sha256sum build\/libs\/burp-mcp\.jar/);
  assert.match(releaseWorkflow, /--repo \"\$\{GITHUB_REPOSITORY\}\"/);
  assert.ok(
    releaseWorkflow.indexOf('Upload GitHub Release assets') <
      releaseWorkflow.indexOf('Stage npm package with provenance'),
  );
});
