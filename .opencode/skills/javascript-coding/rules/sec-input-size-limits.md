# sec-input-size-limits

> Limit the size of incoming request bodies, file uploads, and streams

## Why It Matters

Without size limits, attackers can send enormous payloads that exhaust server memory (DoS), fill disk storage with junk uploads, or cause JSON/XML parsers to hang. Setting explicit limits on body size, file upload size, and stream length prevents resource exhaustion and is a fundamental security control.

## Bad

```js
// No size limits — attacker can send gigabytes of data
app.use(express.json());  // Default limit: 100kb (Express 4.x), but explicit is better
app.use(express.urlencoded({ extended: true }));

// No limit on file uploads
app.post('/upload', upload.single('file'), (req, res) => {
  // req.file could be 10GB
});

// Reading a stream with no cap
async function readAll(readable) {
  const chunks = [];
  for await (const chunk of readable) {
    chunks.push(chunk);  // Memory grows without bound
  }
  return Buffer.concat(chunks);
}
```

## Good

```js
// Explicit body size limits
app.use(express.json({ limit: '1mb' }));
app.use(express.urlencoded({ extended: true, limit: '1mb' }));

// File upload with size limit
import multer from 'multer';

const upload = multer({
  limits: {
    fileSize: 5 * 1024 * 1024,   // 5MB
    files: 1,                     // Max 1 file
  },
  fileFilter: (req, file, cb) => {
    // Validate MIME type
    if (file.mimetype !== 'image/png') {
      return cb(new Error('Only PNG files allowed'));
    }
    cb(null, true);
  },
});

app.post('/upload', upload.single('file'), (req, res) => {
  if (!req.file) return res.status(400).json({ error: 'No file uploaded' });
  res.json({ filename: req.file.filename });
});

// Stream with size cap
async function readWithLimit(readable, maxBytes = 10 * 1024 * 1024) {
  const chunks = [];
  let total = 0;

  for await (const chunk of readable) {
    total += chunk.length;
    if (total > maxBytes) {
      readable.destroy();
      throw new Error(`Stream exceeded ${maxBytes} byte limit`);
    }
    chunks.push(chunk);
  }

  return Buffer.concat(chunks);
}
```

## When Exceptions Apply

Internal data pipelines that process trusted data can use larger or no limits. Public-facing endpoints must always have limits.

## See Also

- [sec-rate-limit](./sec-rate-limit.md) - Rate limiting
- [node-stream-over-buffer](./node-stream-over-buffer.md) - Use streams for large data
