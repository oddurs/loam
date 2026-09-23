# Where the real cases came from

The three directories under `real/` hold the `docs/` folders of three
repositories by the same author as this one, copied byte for byte from the
commit named below. They are here as test data: a reader of the format must
read them, and the corpus records what it must read. Nothing in them has been
edited to suit the format; where one of them has a broken link, the
expectation says so.

Only the pages — the `.md` files under `docs/` — are copied. Every other file
under `docs/` (an image, an HTML page) is present as an empty file, because a
link may name it and only its existence matters. A file or directory outside
`docs/` that a page links to, and that existed in the repository at that
commit, is present as an empty file or an empty directory, for the same reason.
`loam.toml` in each is written for the corpus.

Each keeps the terms it was published under. Those terms, not this
repository's licence, govern the copied pages.

| Case | Repository | Commit | Terms for `docs/` |
| --- | --- | --- | --- |
| `real/code-as-color` | [oddurs/three-numbers](https://github.com/oddurs/three-numbers) | `a8544cb995ba57dd78f6c17bf246a1d17cb05c3a` | Copyright © 2026 Oddur Sigurdsson, all rights reserved; reproduced here by the author as test data. Its `LICENSE` is copied beside it. |
| `real/poptop` | [oddurs/poptop](https://github.com/oddurs/poptop) | `d8000ec8a8ad5255c81ed920007f8300885f1279` | GNU General Public License, version 3. Its `LICENSE` is copied beside it. |
| `real/measure-of-the-world` | [oddurs/measure-of-the-world](https://github.com/oddurs/measure-of-the-world) | `e5ebe11258d1b1d6602d35ff8efdbc1ac66520ab` | Creative Commons Attribution 4.0 International. Its `LICENSE` is copied beside it. |
