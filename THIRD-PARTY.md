# Third-party notices

yolowhtml is MIT-licensed. It includes ideas, rules, and evaluation conventions from the projects below. No source code from them is included unless stated.

## Algorithms and rules

- **trafilatura** (https://github.com/adbar/trafilatura), Apache-2.0 License, Copyright (c) 2019-2024 Adrien Barbaresi.
  The container-arbitration concept (`compare_extraction` between structural container roots and Readability-scored roots) and non-content class/id patterns inspire parts of `crates/yolowhtml/src/article.rs` and `crates/yolowhtml/src/classes.rs`.

- **htmldate** (https://github.com/adbar/htmldate), Apache-2.0 License, Copyright (c) 2017-2024 Adrien Barbaresi.
  The search order and priority tiers for publication dates (JSON-LD, meta tags, `<time>`, URL slugs, single-pass text date scanners) inspire `crates/yolowhtml/src/dates/`.

- **Mozilla Readability** (https://github.com/mozilla/readability), Apache-2.0 License.
  Candidate scoring and link-density ideas inspire `crates/yolowhtml/src/readability.rs`.

- **WHATWG HTML Living Standard** (https://html.spec.whatwg.org/multipage/parsing.html) and **web-platform-tests / html5lib-tests** (https://github.com/web-platform-tests/wpt), BSD-3-Clause.
  Tree-construction rules and named character references in `crates/yolowhtml-dom/`.

## Benchmark datasets (not redistributed)

- **Firecrawl `scrape-content-dataset-v1`** (597 pages)
- **Article Extraction Benchmark (AEB)** (150 pages) and **WCXB** (150 pages)
- **WebMainBench** (800 pages)
- **htmldate evaluation corpus** (1,000 pages)
