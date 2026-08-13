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
