# tree-rs Feature Comparison with Original `tree` Command

---

## Supported options

| Flag | Supported |
|------|-----------|
| `-a`, `--all` | ✅ |
| `-d` | ✅ |
| `-l` | ❌ |
| `-f` | ❌ |
| `-x` | ❌ |
| `-L <level>`, `--level` | ✅ |
| `-R` | ❌ |
| `-P <pattern>` | ✅ |
| `-I <pattern>` | ✅ |
| `--gitignore` | ❌ |
| `--gitfile` | ❌ |
| `--ignore-case` | ❌ |
| `--matchdirs` | ❌ |
| `--metafirst` | ❌ |
| `--prune` | ❌ |
| `--info` | ❌ |
| `--infofile` | ❌ |
| `--noreport` | ❌ |
| `--charset` | ❌ |
| `--filelimit` | ❌ |
| `--timefmt` | ❌ |
| `-o <filename>` | ❌ |
| `-q` | ❌ |
| `-N` | ❌ |
| `-Q` | ❌ |
| `-p` | ❌ |
| `-u` | ❌ |
| `-g` | ❌ |
| `-s` | ❌ |
| `-h` | ❌ |
| `--si` | ❌ |
| `--du` | ❌ |
| `-D` | ❌ |
| `-F` | ❌ |
| `--inodes` | ❌ |
| `--device` | ❌ |
| `-v` | ❌ |
| `-t` | ❌ |
| `-c` | ❌ |
| `-U` | ❌ |
| `-r` | ❌ |
| `--dirsfirst` | ❌ |
| `--filesfirst` | ❌ |
| `--sort` | ❌ |
| `-i` | ❌ |
| `-A` | ❌ |
| `-S` | ❌ |
| `-n` | ✅ |
| `-C` | ✅ |
| `-X` | ❌ |
| `-J` | ❌ |
| `-H <baseHREF>` | ❌ |
| `-T <title>` | ❌ |
| `--nolinks` | ❌ |
| `--hintro` | ❌ |
| `--houtro` | ❌ |
| `--hyperlink` | ❌ |
| `--scheme` | ❌ |
| `--authority` | ❌ |
| `--fromfile` | ❌ |
| `--fromtabfile` | ❌ |
| `--fflinks` | ❌ |
| `--opt-toggle` | ❌ |
| `--help` | ✅ |
| `--version` | ✅ |
| `--` | ❌ |

---

## Environment Variables

| Variable | Supported |
|----------|-----------|
| `LS_COLORS` | ❌ |
| `TREE_COLORS` | ❌ |
| `CLICOLOR` | ❌ |
| `CLICOLOR_FORCE` | ❌ |
| `NO_COLOR` | ❌ |
| `LC_CTYPE` | ❌ |
| `LC_TIME` | ❌ |
| `TZ` | ❌ |

---

## Special Files

| Feature | Supported |
|---------|-----------|
| `.gitignore` | ❌ |
| `.info` files | ❌ |
| `/etc/DIR_COLORS` | ❌ |
| `~/.dircolors` | ❌ |

---

*Comparison based on tree v2.2.1 and tree-rs v0.6.4 (2025-10-04)*
