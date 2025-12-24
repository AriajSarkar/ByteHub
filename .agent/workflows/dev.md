---
description: Run development server with hot-reload
---
// turbo-all

1. Start the development server with watch mode:
```bash
cargo watch -c -x run
```

This will:
- Clear the screen on each rebuild (`-c`)
- Automatically rebuild and restart when files change
- Show the startup banner each time

2. (Optional) In a separate terminal, start cloudflared tunnel:
```bash
cloudflared tunnel --url http://localhost:3000
```
