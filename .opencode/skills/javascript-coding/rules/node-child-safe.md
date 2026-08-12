# node-child-safe

> Prefer `spawn()` over `exec()` — never use `exec()` with user input

## Why It Matters

`exec()` runs a command through a shell, which interprets special characters. If user input reaches the command string, attackers can inject arbitrary shell commands. `spawn()` bypasses the shell by default, accepting arguments as an array where each argument is passed literally. Even without user input, `spawn()` avoids shell interpretation overhead.

## Bad

```js
// exec with user input — shell injection
import { exec } from 'node:child_process';

app.get('/ping', (req, res) => {
  const host = req.query.host;  // "example.com; rm -rf /"
  exec(`ping -c 1 ${host}`, (err, stdout) => {
    // Attacker controls the shell command
    res.send(stdout);
  });
});
```

## Good

```js
// spawn — no shell, safe
import { spawn } from 'node:child_process';

app.get('/ping', (req, res) => {
  const host = req.query.host;

  // Validate input first
  if (!/^[a-zA-Z0-9.-]+$/.test(host)) {
    return res.status(400).json({ error: 'Invalid host' });
  }

  const child = spawn('ping', ['-c', '1', host]);

  let stdout = '';
  child.stdout.on('data', (data) => { stdout += data; });

  child.on('close', (code) => {
    res.send(code === 0 ? stdout : 'Ping failed');
  });
});
```

## execFile — Middle Ground

```js
// execFile — runs a command without a shell
import { execFile } from 'node:child_process';

execFile('convert', ['input.png', '-resize', '50%', 'output.png'], (err) => {
  if (err) console.error('Conversion failed');
});
```

## When Exceptions Apply

`exec()` is acceptable for hardcoded commands with no user input during build scripts or system administration tools. For any application code, prefer `spawn()` or `execFile()`.

## See Also

- [sec-avoid-os-command](./sec-avoid-os-command.md) - Avoid OS command injection
- [sec-input-sanitize](./sec-input-sanitize.md) - Sanitize user input
