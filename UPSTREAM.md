# Upstream

This repository extracts the HTML5 parser (`pagesift-html`) and main-content / Markdown / metadata / publication-date engine (`pagesift-content`) from PageSift 1.2.6 (https://github.com/YOLOW92/pagesift).

Copied and renamed:

| Upstream crate | Here |
|---|---|
| `pagesift-html` | `crates/yolowhtml-dom` |
| `pagesift-content` | `crates/yolowhtml` |
| `pagesift-testkit` | `crates/yolowhtml-testkit` |
| `pagesift-sys` | `crates/yolowhtml-sys` |

The code is the same implementation, with package names, repository metadata, and the license (MIT here; PageSift itself remains AGPL-3.0-only) updated. The copyright owner relicensed this extracted code under MIT. PageSift `1.2.7+` depends directly on `yolowhtml` and `yolowhtml-dom`.

Public API entry points: `yolowhtml::process`, `yolowhtml::process_with_workspace`, `yolowhtml::extract_metadata`, `yolowhtml::Options`, and `yolowhtml_dom::Html::parse` / `yolowhtml_dom::Selector`.
