# Phase 8 — bench results

## cchud configure cold-start (no-TTY guard)

| Run                                       | Mean (ms) | p95 (ms) |
|-------------------------------------------|-----------|----------|
| `printf '' \| cchud configure`            | 3.5       | 5.0      |

NFR: < 50 ms p95 — config load + sample::payload + tempfile + IsTerminal check.

Запуск:
```
hyperfine --warmup 3 --runs 50 --ignore-failure 'printf "" | ./target/release/cchud configure'
```
