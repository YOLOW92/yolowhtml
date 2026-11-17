# Third-party notices

yolowhtml is MIT-licensed. It includes one piece of third-party source code (listed first below) and ideas, rules, and evaluation conventions from the other projects below. No other third-party source code is included.

## Source code

- **markup5ever 0.13** (https://github.com/servo/html5ever), MIT OR Apache-2.0 License, Copyright (c) 2014 The html5ever Project Developers.
  `crates/yolowhtml-dom/src/entities.rs` (the HTML named character reference table) was converted from markup5ever's `entities.rs`. It is used here under the MIT license, whose text follows:

  > Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
  >
  > The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
  >
  > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

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
