# Фаза 5 — Git-виджеты (полный набор)

**Длительность:** 3–5 дней
**Входные условия:** Фазы 0–3 (фаза 4 параллельна, не блокирует)
**Релиз:** 0.3.0

## Цель

Все 20+ git-виджетов ccstatusline. Один общий `GitInfo` lazy-init, виджеты — тонкие getter'ы. `GitPr` через GitHub API с дисковым кэшем.

## Список виджетов

| Виджет | Источник | Сложность |
|---|---|---|
| `GitBranch` | head | low (готов в Фазе 3) |
| `GitSha` | head ref | low |
| `GitRootDir` | discover | low |
| `GitAheadBehind` | upstream tracking | mid |
| `GitIsFork` | remote URL parsing | low |
| `GitChanges` | gix status | mid (готов в Фазе 3) |
| `GitStaged` | gix status | low |
| `GitUnstaged` | gix status | low |
| `GitUntracked` | gix status | low |
| `GitConflicts` | gix status | low |
| `GitInsertions` | diff stat | mid |
| `GitDeletions` | diff stat | mid |
| `GitOriginOwner` | parse origin URL | low |
| `GitOriginRepo` | parse origin URL | low |
| `GitOriginOwnerRepo` | parse origin URL | low |
| `GitUpstreamOwner` | parse upstream URL | low |
| `GitUpstreamRepo` | parse upstream URL | low |
| `GitUpstreamOwnerRepo` | parse upstream URL | low |
| `GitWorktree` | gix worktree | mid |
| `GitWorktreeBranch` | gix worktree | mid |
| `GitWorktreeMode` | gix worktree | mid |
| `GitWorktreeName` | gix worktree | low |
| `GitWorktreeOriginalBranch` | gix worktree | mid |
| `GitPr` | GitHub API | high (HTTP + cache + auth) |

## Шаги

### 5.1. Расширение `GitInfo`

`src/git/mod.rs`:
```rust
pub struct GitInfo {
    repo: gix::Repository,
    pub root_dir: PathBuf,
    pub head: Head,
    pub status: GitStatus,
    pub remotes: HashMap<String, RemoteInfo>,
    pub upstream: Option<UpstreamInfo>,
    pub worktree: Option<WorktreeInfo>,
}

pub struct Head {
    pub branch: Option<String>,
    pub sha: Option<String>,
    pub short_sha: Option<String>,
}

pub struct GitStatus {
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
    pub conflicts: u32,
    pub insertions: u32,
    pub deletions: u32,
    pub ahead: u32,
    pub behind: u32,
}

pub struct RemoteInfo {
    pub url: String,
    pub owner: Option<String>,
    pub repo: Option<String>,
}

pub struct WorktreeInfo {
    pub name: String,
    pub branch: Option<String>,
    pub mode: WorktreeMode,
    pub original_branch: Option<String>,
}
```

### 5.2. Парсинг remote URL

Порт `utils/git-remote.ts` — поддержка форматов:
```
git@github.com:owner/repo.git
https://github.com/owner/repo.git
https://github.com/owner/repo
ssh://git@gitlab.com/owner/repo.git
```

`src/git/remote.rs`:
```rust
pub fn parse_url(url: &str) -> Option<(String, String)> {
    // regex или splitn — берём owner и repo
    static SSH_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:ssh://)?(?:[\w-]+@)?([\w.-]+):([\w./-]+)").unwrap());
    static HTTP_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^https?://[\w.-]+/([\w-]+)/([\w.-]+?)(?:\.git)?/?$").unwrap());
    // ...
}
```

> Для cold-start: `regex` тяжелее чем ручной парсер; рассмотреть split-based решение без regex.

### 5.3. Diff stat (insertions/deletions)

`gix` умеет diff, но это не самый дешёвый вызов. Стратегия:
- Lazy: считать только если есть виджет `GitInsertions`/`GitDeletions` в строке
- Кэшировать в `GitInfo` — рассчитывается при первом запросе

```rust
impl GitInfo {
    pub fn diff_stat(&self) -> Option<&DiffStat> {
        self.diff_stat.get_or_init(|| compute_diff_stat(&self.repo))
    }
}
```

### 5.4. Worktree-виджеты

```rust
fn worktree_info(repo: &gix::Repository) -> Option<WorktreeInfo> {
    // gix::worktree::open для текущего worktree
    // если HEAD detached → mode = Detached
    // если main worktree → mode = Main
    // если linked worktree → mode = Linked
}
```

### 5.5. `GitPr` — самый сложный

#### Auth
```rust
fn github_token() -> Option<String> {
    // Приоритет:
    // 1. env GITHUB_TOKEN
    std::env::var("GITHUB_TOKEN").ok()
        // 2. gh CLI
        .or_else(|| Command::new("gh").args(["auth", "token"]).output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()))
}
```

#### HTTP-вызов
```rust
fn fetch_pr(owner: &str, repo: &str, branch: &str, token: &str) -> Option<PrInfo> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/pulls?head={owner}:{branch}");
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("User-Agent", "cchud")
        .timeout(Duration::from_millis(200))
        .call().ok()?;
    let prs: Vec<PrJson> = resp.into_json().ok()?;
    prs.into_iter().next().map(|p| PrInfo { number: p.number, state: p.state, ... })
}
```

#### Кэш
`~/.cache/cchud/pr-cache.bincode`:
```rust
pub struct PrCache {
    entries: HashMap<String, CachedPr>,  // key: owner/repo:branch
}

pub struct CachedPr {
    fetched_at: u64,
    pr: Option<PrInfo>,
}

const TTL_SECS: u64 = 30;

pub fn lookup_or_fetch(owner: &str, repo: &str, branch: &str) -> Option<PrInfo> {
    let key = format!("{owner}/{repo}:{branch}");
    let mut cache = read_cache();
    if let Some(entry) = cache.entries.get(&key) {
        if now() - entry.fetched_at < TTL_SECS {
            return entry.pr.clone();
        }
    }
    let token = github_token()?;
    let pr = fetch_pr(owner, repo, branch, &token);
    cache.entries.insert(key, CachedPr { fetched_at: now(), pr: pr.clone() });
    write_cache(&cache);
    pr
}
```

#### Виджет
```rust
impl Widget for GitPr {
    fn render(&self, ctx: &RenderContext) -> Option<String> {
        let git = ctx.git()?;
        let origin = git.remotes.get("origin")?;
        let owner = origin.owner.clone()?;
        let repo = origin.repo.clone()?;
        let branch = git.head.branch.clone()?;
        let pr = lookup_or_fetch(&owner, &repo, &branch)?;
        Some(format!("PR #{}", pr.number))
    }
}
```

### 5.6. Регистрация всех виджетов

Расширить `WidgetConfig` enum 24 вариантами и `build_widgets` match'ем.

### 5.7. Тесты

- Unit: парсинг URL разных форматов
- Integration: создать temp git-репо с фикстурами (`git2` или shell), проверить что виджеты видят правильное состояние
- `GitPr` mock'ать через `mockito` или wiremock-style

```rust
#[test]
fn parse_remote_urls() {
    assert_eq!(parse_url("git@github.com:foo/bar.git"), Some(("foo".into(), "bar".into())));
    assert_eq!(parse_url("https://github.com/foo/bar"), Some(("foo".into(), "bar".into())));
    assert_eq!(parse_url("ssh://git@gitlab.com/group/proj.git"), Some(("group".into(), "proj".into())));
}
```

### 5.8. Performance check

С конфигом, использующим **все** git-виджеты:
- На репе среднего размера (10k commits) — < 8 мс p95 (доп. бюджет на git stat)
- На монорепо (100k commits) — < 15 мс (документировать как edge case)

### 5.9. Релиз 0.3.0

## Exit Criteria

- [ ] Все 24 git-виджета работают
- [ ] Один lazy `GitInfo` переиспользуется всеми
- [ ] `GitPr` корректно auth через `gh auth token` и `GITHUB_TOKEN`
- [ ] Кэш PR с TTL 30 сек, не блокирует на offline (timeout 200 мс)
- [ ] Snapshot-тесты на 3+ git-сценариях (clean, dirty, conflicts, fork, worktree)
- [ ] Hyperfine: < 8 мс p95 даже с активным `GitPr` (cache hit)
- [ ] Релиз 0.3.0

## Связи

- **Фаза 6** не зависит — может идти параллельно
- **Фаза 7** добавляет последние ~30 виджетов
- **Фаза 9** README с примерами git-конфигов

## Риски

- **gix breaking changes между minor** — pin к `0.81`, smoke-тест в CI на каждом обновлении.
- **GitHub API rate limit** для `GitPr` без токена — fallback на анонимные запросы с документацией про rate limit.
- **Большой монорепо** — `gix status` тяжёлый. Документировать опцию отключить status-виджеты.
- **Worktree edge cases** (detached HEAD, bare repo, submodule) — фикстуры в тестах.
