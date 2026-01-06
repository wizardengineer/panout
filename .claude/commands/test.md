---
description: Closed loop instafix testing - runs and fixes tests
---

# Test Instafix (Closed Loop)

Run tests and fix failures automatically.

## Closed Loop Pattern

### 1. Request
Run unit testing for Instafix

### 2. Validate
After compiling, you can run:
```bash
source ~/src/instafix/init_dev_instafix_env.sh
ctest --test-dir ~/src/instafix/out/build/linux --output-on-failure 2>&1
```

### 3. Resolve

If tests fail:
- Read the failing test output
- Fix the component or test
- Return to step 2

Loop exits when all tests pass.
