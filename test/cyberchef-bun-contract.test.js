'use strict';

const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const path = require('node:path');
const { test } = require('node:test');

const root = path.join(__dirname, '..');

test('uses the official CyberChef package without a Node worker dependency', () => {
  // Given
  const packageJson = JSON.parse(readFileSync(path.join(root, 'package.json'), 'utf8'));
  const engine = readFileSync(path.join(root, 'bridge/src/cyberchef-engine.mjs'), 'utf8');
  const catalog = readFileSync(path.join(root, 'bridge/src/cyberchef-catalog.mjs'), 'utf8');
  const runtime = readFileSync(path.join(root, 'bridge/src/cyberchef-runtime.ts'), 'utf8');

  // When
  const dependencies = packageJson.dependencies;

  // Then
  assert.equal(dependencies.cyberchef, '11.3.0');
  assert.equal(dependencies['cyberchef-node'], undefined);
  assert.equal(packageJson.engines.node, undefined);
  assert.equal(packageJson.patchedDependencies, undefined);
  assert.equal(packageJson.overrides, undefined);
  assert.doesNotMatch(`${engine}\n${catalog}`, /cyberchef-node|src\/core\//);
  assert.match(runtime, /runtimeCommand = process\.execPath/);
  assert.doesNotMatch(runtime, /nodeCommand|cyberchef-loader/);
});
