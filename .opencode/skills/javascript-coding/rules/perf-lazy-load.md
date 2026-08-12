# perf-lazy-load

> Lazy-load heavy modules with dynamic `import()` to reduce startup time and memory

## Why It Matters

Every static `import` is loaded and evaluated at startup, regardless of whether the code path is ever reached. Large libraries (PDF generators, image processors, template engines) can add seconds to cold start. Dynamic `import()` defers loading until the module is actually needed, reducing startup time and memory footprint.

## Bad

```js
// All loaded at startup — even if never used
import { generatePDF } from './pdf-generator.js';  // 5MB
import { processImage } from './image-tools.js';    // 3MB
import { exportExcel } from './excel-exporter.js';  // 2MB

app.post('/export', async (req, res) => {
  if (req.body.format === 'pdf') {
    return res.send(await generatePDF(req.body.data));
  }
  // image-tools and excel-exporter loaded for nothing
});
```

## Good

```js
app.post('/export', async (req, res) => {
  switch (req.body.format) {
    case 'pdf': {
      const { generatePDF } = await import('./pdf-generator.js');
      return res.send(await generatePDF(req.body.data));
    }
    case 'image': {
      const { processImage } = await import('./image-tools.js');
      return res.send(await processImage(req.body.data));
    }
    default:
      return res.status(400).json({ error: 'Unsupported format' });
  }
});
```

## When Exceptions Apply

Don't lazy-load modules that are used on every request — the dynamic import overhead would hurt latency. Lazy-load modules that are large (>100KB) AND rarely used or conditionally needed.

## See Also

- [mod-dynamic-import](./mod-dynamic-import.md) - Dynamic import() for lazy loading
- [async-worker-threads](./async-worker-threads.md) - Worker threads for CPU work
