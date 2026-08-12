# mod-dynamic-import

> Use dynamic `import()` for lazy loading of large or conditionally-needed modules

## Why It Matters

Static imports load all dependencies at startup, increasing cold-start time and memory usage. Dynamic `import()` defers loading until the module is actually needed, reducing startup cost and enabling code splitting. This is critical for serverless functions, CLI tools, and applications with many optional features.

## Bad

```js
// Static imports — all loaded at startup, even if never used
import { pdfGenerator } from './heavy-pdf-module.js';  // 5MB, loaded always
import { imageProcessor } from './image-processor.js';   // 3MB, loaded always
import { excelExporter } from './excel-exporter.js';     // 2MB, loaded always

app.post('/export', async (req, res) => {
  if (req.body.format === 'pdf') {
    return res.send(await pdfGenerator(req.body.data));
  }
  // Excel and image modules loaded but never used in this request
});
```

## Good

```js
// Dynamic imports — loaded on demand
app.post('/export', async (req, res) => {
  switch (req.body.format) {
    case 'pdf': {
      const { pdfGenerator } = await import('./heavy-pdf-module.js');
      return res.send(await pdfGenerator(req.body.data));
    }
    case 'excel': {
      const { excelExporter } = await import('./excel-exporter.js');
      return res.send(await excelExporter(req.body.data));
    }
    case 'image': {
      const { imageProcessor } = await import('./image-processor.js');
      return res.send(await imageProcessor(req.body.data));
    }
    default:
      return res.status(400).json({ error: 'Unsupported format' });
  }
});
```

## Conditional Loading

```js
async function getParser(language) {
  switch (language) {
    case 'yaml':
      return (await import('./parsers/yaml-parser.js')).parse;
    case 'toml':
      return (await import('./parsers/toml-parser.js')).parse;
    default:
      return (await import('./parsers/json-parser.js')).parse;
  }
}
```

## When Exceptions Apply

Don't use dynamic imports for small, frequently-used modules — the overhead of the dynamic import call outweighs the benefit. Reserve for large modules (> 100KB) or rarely-used features.

## See Also

- [perf-lazy-load](./perf-lazy-load.md) - Lazy-loading patterns
- [mod-esm-over-cjs](./mod-esm-over-cjs.md) - ESM preference
