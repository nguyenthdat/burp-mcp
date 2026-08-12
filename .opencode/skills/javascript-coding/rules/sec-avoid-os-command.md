# sec-avoid-os-command

> Avoid child_process.exec() — use execFile() or spawn() with explicit arguments

## Why It Matters

`child_process.exec()` runs a command through a shell, which interprets special characters. If user input is interpolated into the command string, attackers can inject arbitrary shell commands (command injection). `execFile()` and `spawn()` bypass the shell, accepting arguments as an array where each argument is passed literally.

## Bad

```js
// Shell injection risk — user controls filename
import { exec } from 'node:child_process';

app.post('/convert', (req, res) => {
  const filename = req.body.filename;  // "file; rm -rf /"
  exec(`convert ${filename} output.pdf`, (err) => {
    // Attacker can inject arbitrary shell commands
  });
});

// String interpolation in exec
const user = req.query.user;
exec(`useradd ${user}`, (err) => { /* ... */ });
```

## Good

```js
// execFile — arguments are passed as an array, no shell interpretation
import { execFile } from 'node:child_process';

app.post('/convert', (req, res) => {
  const filename = req.body.filename;

  // Validate filename before use
  if (!/^[\w.-]+$/.test(filename)) {
    return res.status(400).json({ error: 'Invalid filename' });
  }

  execFile('convert', [filename, 'output.pdf'], (err, stdout) => {
    if (err) return res.status(500).json({ error: 'Conversion failed' });
    res.json({ result: stdout });
  });
});

// spawn for long-running processes with streaming output
import { spawn } from 'node:child_process';

const child = spawn('ffmpeg', [
  '-i', inputPath,
  '-c:v', 'libx264',
  outputPath,
]);

child.on('error', (err) => {
  console.error('FFmpeg failed:', err);
});
```

## When Exceptions Apply

`exec()` is acceptable when the command is entirely hardcoded with no user input — but `execFile()` is still the safer default.

## See Also

- [node-child-safe](./node-child-safe.md) - Prefer spawn over exec
- [sec-input-sanitize](./sec-input-sanitize.md) - Sanitize all user input
