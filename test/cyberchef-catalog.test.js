'use strict';

const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { test } = require('node:test');

async function loadCatalog() {
  const sourceRoot = path.join(__dirname, '..', 'bridge', 'src');
  await import(pathToFileURL(path.join(sourceRoot, 'cyberchef-bun-compat.mjs')).href);
  return import(pathToFileURL(path.join(sourceRoot, 'cyberchef-catalog.mjs')).href);
}

test('maps every advertised CyberChef operation to a public operation wrapper', async () => {
  // Given
  const { getOperationDescriptor, listOperations } = await loadCatalog();

  // When / Then
  for (const operation of listOperations()) {
    const descriptor = getOperationDescriptor(operation.name);
    assert.notEqual(descriptor, null, operation.name);
    assert.equal(typeof descriptor.execute, 'function', operation.name);
  }
});

test('ranks both directions for a compound Base64 encode decode query', async () => {
  // Given
  const { searchOperations } = await loadCatalog();

  // When
  const names = searchOperations('Base64 encode decode', 5).matches.map(({ name }) => name);

  // Then
  assert.deepEqual(names.slice(0, 2).sort(), ['From Base64', 'To Base64']);
});

test('ranks compound operation searches independently of query term order', async () => {
  // Given
  const { searchOperations } = await loadCatalog();

  // When
  const names = searchOperations('encode decode Base64', 5).matches.map(({ name }) => name);

  // Then
  assert.deepEqual(names.slice(0, 2).sort(), ['From Base64', 'To Base64']);
});
