'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { test } = require('node:test');

const projectRoot = path.join(__dirname, '..');

test('publishes a Bun-native scoped CLI without a generated root bridge', () => {
  // Given
  const packageJson = JSON.parse(fs.readFileSync(path.join(projectRoot, 'package.json'), 'utf8'));
  const license = fs.readFileSync(path.join(projectRoot, 'LICENSE'), 'utf8');

  // When
  const rootBridgeExists = fs.existsSync(path.join(projectRoot, 'mcp-bridge.js'));

  // Then
  assert.equal(packageJson.name, '@nguyenthdat/burpmcp');
  assert.deepEqual(packageJson.bin, { burpmcp: 'bridge/src/main.ts' });
  assert.deepEqual(packageJson.files, ['bridge/src', 'README.md', 'DISCLOSURE', 'LICENSE']);
  assert.equal(packageJson.license, 'MIT');
  assert.match(license, /^MIT License/);
  assert.match(license, /Copyright \(c\) 2026 Dat Nguyen/);
  assert.equal(packageJson.author.name, 'Dat Nguyen');
  assert.match(packageJson.description, /Burp Suite MCP server/);
  assert.ok(packageJson.keywords.includes('application-security'));
  assert.ok(packageJson.keywords.includes('mcp-server'));
  assert.deepEqual(packageJson.contentPolicy, { class: 'dual-use' });
  assert.deepEqual(packageJson.publishConfig, {
    access: 'public',
    registry: 'https://registry.npmjs.org/',
  });
  assert.equal(packageJson.private, undefined);
  assert.equal(rootBridgeExists, false);
});
