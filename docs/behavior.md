# Behavior, safety, and performance

## Inputs

alofmt accepts standard input, files, and directories.

- Directory traversal selects `.rb` and `.rbi` files.
- Standard ignore files such as `.gitignore` apply.
- Hidden paths are included unless an ignore rule excludes them.
- Symbolic links are rejected and never followed.
- Explicit files are accepted regardless of extension.

File discovery and formatting run in parallel. Results are reported in sorted
path order.

Use `--threads N` to select a fixed worker count. Directory discovery uses at
most four workers because additional walkers increase filesystem contention.
Single-threaded mode bypasses the Rayon pool.

## Modes and exit status

Without `--write` or `--check`, alofmt requires one input and prints the
formatted source to standard output.

`--write` replaces changed files. `--check` changes no files and exits with
status 1 if any file would change. Add `--diff` to print unified diffs.

| Status | Meaning |
| ---: | --- |
| 0 | Every input succeeded, and `--check` found no changes. |
| 1 | `--check` found at least one change. |
| 2 | An input, configuration, or formatter operation failed. |

## Safe writes

Before alofmt replaces a file, it:

1. Rejects symbolic links and non-regular files.
2. Writes a unique temporary file in the same directory.
3. Preserves the original permissions.
4. Re-reads the source and rejects concurrent changes.
5. Renames the completed temporary file over the source.

If cleanup also fails, alofmt reports both the original error and the cleanup
error.

## Failures

alofmt returns an error for invalid UTF-8, Ruby parse errors, invalid
configuration, and unsupported Prism nodes. It does not replace invalid text
or preserve an unsupported node silently.

An input byte-order mark is preserved.

## Performance

A release build checked 3,429 Ruby files on a 14-core Apple M4 Max with a warm
filesystem cache:

| Workers | Wall time | Typical peak RSS |
| ---: | ---: | ---: |
| 1 | 2.14 s | 86–101 MiB |
| 14 | 0.34–0.41 s | 149–152 MiB |

The command was `alofmt --check .`. The measurement includes ignore-aware
discovery, file reads, Prism parsing, layout, and comparison. Results vary with
the corpus, hardware, filesystem cache, and configured worker count.

Prism parses each file once. alofmt attaches comments in one AST traversal and
constructs a compact arena-backed document. Unchanged source remains byte
spans instead of copied strings. Width checks reuse scratch storage.
