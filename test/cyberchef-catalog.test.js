'use strict';

const assert = require('node:assert/strict');
const { readdirSync } = require('node:fs');
const { createRequire } = require('node:module');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { test } = require('node:test');

test('maps every advertised CyberChef operation to an exact source file', async () => {
  const requireFromRepo = createRequire(__filename);
  const cyberchefRoot = path.dirname(requireFromRepo.resolve('cyberchef-node/package.json'));
  const operationFiles = new Set(readdirSync(path.join(cyberchefRoot, 'src/core/operations')));
  const catalogUrl = pathToFileURL(path.join(__dirname, '..', 'bridge', 'src', 'cyberchef-catalog.mjs'));
  const { getOperationDescriptor, listOperations } = await import(catalogUrl.href);

  for (const operation of listOperations()) {
    const descriptor = getOperationDescriptor(operation.name);
    assert.notEqual(descriptor, null, operation.name);
    assert.equal(operationFiles.has(descriptor.file), true, operation.name);
  }
});

test('ranks both directions for a compound Base64 encode decode query', async () => {
  // Given
  const catalogUrl = pathToFileURL(path.join(__dirname, '..', 'bridge', 'src', 'cyberchef-catalog.mjs'));
  const { searchOperations } = await import(catalogUrl.href);

  // When
  const names = searchOperations('Base64 encode decode', 5).matches.map(({ name }) => name);

  // Then
  assert.deepEqual(names.slice(0, 2).sort(), ['From Base64', 'To Base64']);
});

test('ranks compound operation searches independently of query term order', async () => {
  // Given
  const catalogUrl = pathToFileURL(path.join(__dirname, '..', 'bridge', 'src', 'cyberchef-catalog.mjs'));
  const { searchOperations } = await import(catalogUrl.href);

  // When
  const names = searchOperations('encode decode Base64', 5).matches.map(({ name }) => name);

  // Then
  assert.deepEqual(names.slice(0, 2).sort(), ['From Base64', 'To Base64']);
});
