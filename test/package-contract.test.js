'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { test } = require('node:test');

const projectRoot = path.join(__dirname, '..');

test('publishes a Bun-native scoped CLI without a generated root bridge', () => {
  // Given
  const packageJson = JSON.parse(fs.readFileSync(path.join(projectRoot, 'package.json'), 'utf8'));

  // When
  const rootBridgeExists = fs.existsSync(path.join(projectRoot, 'mcp-bridge.js'));

  // Then
  assert.equal(packageJson.name, '@nguyenthdat/burpmcp');
  assert.deepEqual(packageJson.bin, { burpmcp: 'bridge/src/main.ts' });
  assert.deepEqual(packageJson.files, ['bridge/src', 'README.md']);
  assert.deepEqual(packageJson.publishConfig, { access: 'public' });
  assert.equal(packageJson.private, undefined);
  assert.equal(rootBridgeExists, false);
});
