//! 回归测试：严格文章提取、正文评分、完整度与多格式输出中的 `<figure>` / `<figcaption>` 处理。

use url::Url;
use yolowhtml::{Options, process};

fn long_prose() -> String {
    "<p>The research team evaluated three architectural variants across twelve real-world \
     workloads, recording latency, peak resident memory, and structural fidelity under \
     concurrent execution. Every measurement was repeated fifty times under identical thermal \
     and frequency limits to eliminate scheduling noise from the comparison.</p>\
     <p>Across all twelve benchmarks, eliminating duplicate tree construction reduced \
     allocator contention and kept per-worker memory flat even when documents contained deeply \
     nested tables, inline formatting, and embedded mathematical expressions.</p>\
     <p>Additional ablation runs confirmed that retaining contextual captions and tabular \
     summaries alongside the main narrative prevents downstream retrieval stages from losing \
     units, axis definitions, and experimental parameters.</p>"
        .repeat(2)
}

fn opts(base: &str) -> Options {
    Options {
        base_url: Some(Url::parse(base).unwrap()),
        ..Options::default()
    }
}

fn all_opts(base: &str) -> Options {
    Options {
        base_url: Some(Url::parse(base).unwrap()),
        plain_text: true,
        clean_html: true,
        images: true,
        ..Options::default()
    }
}

const ARTICLE_META: &str = r#"<meta property="og:type" content="article"><meta property="article:published_time" content="2026-10-20T08:00:00Z">"#;

#[test]
fn figure_with_img_and_figcaption_before_and_after_in_strict_article() {
    let prose = long_prose();
    let html = format!(
        r#"<!doctype html><html><head><title>Evaluation Report</title>{ARTICLE_META}</head><body>
        <header><nav><a href="/">Home</a> <a href="/archive">Archive</a></nav></header>
        <main><article>
          <h1>Evaluation Report</h1>
          <div class="byline">By Alice Researcher</div>
          <div class="article-body">
            {prose}
            <figure>
              <img src="/img/loss.png" alt="Training loss curve">
              <figcaption>Figure 1: Training loss over 100 epochs.</figcaption>
            </figure>
            <figure>
              <figcaption class="figure-caption">Figure 2: Memory footprint across workers.</figcaption>
              <img src="/img/mem.png" alt="Memory bar chart">
            </figure>
            <figure class="wp-caption aligncenter">
              <div class="image-wrapper"><img src="/img/wp.png" alt="WordPress diagram"></div>
              <figcaption class="wp-caption-text">Figure 3: WordPress captioned figure.</figcaption>
            </figure>
            <figure class="wp-block-image">
              <div class="img-outer"><div class="img-inner"><img src="/img/gutenberg.png" alt="Gutenberg block"></div></div>
              <figcaption class="wp-element-caption"><div>Figure 4: Div-wrapped caption text.</div></figcaption>
            </figure>
          </div>
        </article>
        <aside class="sidebar"><h3>Popular</h3><ul><li><a href="/x">Other post</a></li></ul></aside>
        <div class="related-stories"><h3>Related</h3><ul>
          <li><a href="/r1">First unrelated story in feed</a></li>
          <li><a href="/r2">Second unrelated story in feed</a></li>
          <li><a href="/r3">Third unrelated story in feed</a></li>
        </ul></div></main></body></html>"#
    );
    let md = process(&html, &opts("https://example.org/papers/eval")).markdown;
    for expected in [
        "![Training loss curve](https://example.org/img/loss.png)",
        "Figure 1: Training loss over 100 epochs.",
        "Figure 2: Memory footprint across workers.",
        "![Memory bar chart](https://example.org/img/mem.png)",
        "![WordPress diagram](https://example.org/img/wp.png)",
        "Figure 3: WordPress captioned figure.",
        "![Gutenberg block](https://example.org/img/gutenberg.png)",
        "Figure 4: Div-wrapped caption text.",
    ] {
        assert!(md.contains(expected), "missing {expected:?}:\n{md}");
    }
    for noise in [
        "Archive",
        "By Alice Researcher",
        "Other post",
        "First unrelated story",
    ] {
        assert!(!md.contains(noise), "noise {noise:?} leaked:\n{md}");
    }
}

#[test]
fn figcaption_rich_inline_block_links_and_math() {
    let prose = long_prose();
    let html = format!(
        r#"<!doctype html><html><head><title>Math Paper</title>{ARTICLE_META}</head><body>
        <article>
          <h1>Math Paper</h1>
          {prose}
          <figure>
            <img src="/img/surface.png" alt="Objective surface">
            <figcaption>
              <strong>Figure 5.</strong> Direct prefix with <em>italic</em>, <code>loss_fn</code>,
              data from <a href="/refs/smith">Smith et al.</a>, and inline formula
              <span class="katex"><span class="katex-mathml"><math><semantics><mrow><mi>L</mi></mrow><annotation encoding="application/x-tex">\mathcal{{L}}(\theta)</annotation></semantics></math></span><span class="katex-html" aria-hidden="true">L(theta)</span></span>.
              <p>Second paragraph inside figcaption explaining units in ms/page.</p>
              <math display="block"><mfrac><mi>a</mi><mi>b</mi></mfrac></math>
            </figcaption>
          </figure>
        </article></body></html>"#
    );
    let md = process(&html, &opts("https://example.org/paper")).markdown;
    for expected in [
        "![Objective surface](https://example.org/img/surface.png)",
        "**Figure 5.** Direct prefix with *italic*, `loss_fn`",
        "[Smith et al.](https://example.org/refs/smith)",
        r"$\mathcal{L}(\theta)$",
        "Second paragraph inside figcaption explaining units in ms/page.",
        r"\frac{a}{b}",
    ] {
        assert!(md.contains(expected), "missing {expected:?}:\n{md}");
    }
    assert!(
        !md.contains("L(theta)"),
        "katex visual duplicate leaked:\n{md}"
    );
}

#[test]
fn figure_non_image_content_code_tables_quotes_lists_and_attachments() {
    let prose = long_prose();
    let html = format!(
        r##"<!doctype html><html><head><title>Rust Patterns</title>{ARTICLE_META}</head><body>
        <article>
          <h1>Rust Patterns</h1>
          {prose}
          <figure class="listing" id="listing-1">
            <span class="file-name">Filename: src/lib.rs</span>
            <pre><code class="language-rust">pub fn answer() -&gt; u32 {{ 42 }}</code></pre>
            <figcaption><a href="#listing-1">Listing 1</a>: Returning <code>42</code>.</figcaption>
          </figure>
          <figure class="wp-block-table">
            <table>
              <thead><tr><th>Engine</th><th>F1</th></tr></thead>
              <tbody><tr><td>yolowhtml</td><td>0.95</td></tr><tr><td>baseline</td><td>0.84</td></tr></tbody>
            </table>
            <figcaption>Table 1: Benchmark F1 scores.</figcaption>
          </figure>
          <figure>
            <blockquote><p>Keep semantic containers intact.</p></blockquote>
            <figcaption>Design principle</figcaption>
          </figure>
          <figure>
            <ul><li>First invariant</li><li>Second invariant</li></ul>
            <figcaption>Summary list</figcaption>
          </figure>
          <figure class="attachment attachment--preview">
            <img src="/img/attach.png" alt="Attached chart">
            <figcaption>Attachment figure from ActionText.</figcaption>
          </figure>
        </article></body></html>"##
    );
    let md = process(&html, &opts("https://example.org/rust")).markdown;
    for expected in [
        "Filename: src/lib.rs",
        "```rust\npub fn answer() -> u32 { 42 }\n```",
        "Listing 1: Returning `42`.",
        "| Engine | F1 |\n| --- | --- |\n| yolowhtml | 0.95 |\n| baseline | 0.84 |",
        "Table 1: Benchmark F1 scores.",
        "> Keep semantic containers intact.",
        "Design principle",
        "- First invariant\n- Second invariant",
        "Summary list",
        "![Attached chart](https://example.org/img/attach.png)",
        "Attachment figure from ActionText.",
    ] {
        assert!(md.contains(expected), "missing {expected:?}:\n{md}");
    }
}

#[test]
fn figure_multiple_images_captionless_and_deeply_nested() {
    let prose = long_prose();
    let open = "<div>".repeat(14);
    let close = "</div>".repeat(14);
    let html = format!(
        r#"<!doctype html><html><head><title>Deep Figures</title>{ARTICLE_META}</head><body>
        <article>
          <h1>Deep Figures</h1>
          {prose}
          <figure>
            <img src="/img/left.png" alt="Left view">
            <img src="/img/right.png" alt="Right view">
            <figcaption>Comparison of left and right views.</figcaption>
          </figure>
          <figure>
            <div class="panel"><img src="/img/p1.png" alt="Panel 1"></div>
            <div class="panel"><img src="/img/p2.png" alt="Panel 2"></div>
            <figcaption>Two div-wrapped panels sharing one caption.</figcaption>
          </figure>
          <figure><img src="/img/solo.png" alt="Captionless architecture"></figure>
          {open}
          <figure>
            <img src="/img/deep.png" alt="Deeply nested plot">
            <figcaption>Deeply nested caption beyond ten ancestors.</figcaption>
          </figure>
          {close}
        </article></body></html>"#
    );
    let md = process(&html, &opts("https://example.org/deep")).markdown;
    for expected in [
        "![Left view](https://example.org/img/left.png)",
        "![Right view](https://example.org/img/right.png)",
        "Comparison of left and right views.",
        "![Panel 1](https://example.org/img/p1.png)",
        "![Panel 2](https://example.org/img/p2.png)",
        "Two div-wrapped panels sharing one caption.",
        "![Captionless architecture](https://example.org/img/solo.png)",
        "![Deeply nested plot](https://example.org/img/deep.png)",
        "Deeply nested caption beyond ten ancestors.",
    ] {
        assert!(md.contains(expected), "missing {expected:?}:\n{md}");
    }
}

#[test]
fn figure_in_legitimate_container_without_article_tag() {
    let prose = long_prose();
    let html = format!(
        r#"<!doctype html><html><head><title>Technical Guide Without Article Tag</title></head><body>
        <header><nav><a href="/">Home</a> <a href="/guides">Guides</a> <a href="/api-ref">API Reference</a></nav></header>
        <main>
          <div class="post-content">
            <h1>Technical Guide Without Article Tag</h1>
            {prose}
            <figure>
              <a href="/img/arch-full.png"><img src="/img/arch.png" alt="Pipeline architecture"></a>
              <figcaption>Figure 1: Single-pass extraction architecture without an article tag.</figcaption>
            </figure>
          </div>
        </main>
        <aside class="sidebar"><p>Newsletter signup and archive links.</p></aside>
        </body></html>"#
    );
    let md = process(&html, &opts("https://example.org/docs/arch")).markdown;
    assert!(
        md.contains("[![Pipeline architecture](https://example.org/img/arch.png)](https://example.org/img/arch-full.png)"),
        "{md}"
    );
    assert!(
        md.contains("Figure 1: Single-pass extraction architecture without an article tag."),
        "{md}"
    );
    assert!(!md.contains("Newsletter signup"), "{md}");
    assert!(!md.contains("API Reference"), "{md}");

    // 连 <main> 都没有，仅靠 <div id="content"> 承载正文
    let no_main = format!(
        r#"<!doctype html><html><head><title>Plain Div Container</title></head><body>
        <div class="nav-links"><a href="/a">Menu A</a> <a href="/b">Menu B</a> <a href="/c">Menu C</a></div>
        <div id="content">
          <h1>Plain Div Container</h1>
          {prose}
          <figure>
            <img src="/img/plain.png" alt="Plain container plot">
            <figcaption>Figure 2: Inside div#content without article or main.</figcaption>
          </figure>
        </div>
        </body></html>"#
    );
    let md2 = process(&no_main, &opts("https://example.org/plain")).markdown;
    assert!(
        md2.contains("![Plain container plot](https://example.org/img/plain.png)"),
        "{md2}"
    );
    assert!(
        md2.contains("Figure 2: Inside div#content without article or main."),
        "{md2}"
    );
    assert!(!md2.contains("Menu A"), "{md2}");
}

#[test]
fn noise_figures_and_hidden_figures_stay_filtered() {
    let prose = long_prose();
    let html = format!(
        r#"<!doctype html><html><head><title>Noise vs Content</title>{ARTICLE_META}</head><body>
        <nav><figure><img src="/nav.png" alt="Nav"><figcaption>NAV_NOISE</figcaption></figure></nav>
        <main><article>
          <h1>Noise vs Content</h1>
          {prose}
          <figure>
            <img src="/valid.png" alt="Valid chart">
            <figcaption>VALID_CAPTION</figcaption>
            <figcaption hidden>HIDDEN_CAPTION_ATTR</figcaption>
            <div style="display: none">HIDDEN_STYLE_CHILD</div>
            <span aria-hidden="true">HIDDEN_ARIA_ICON</span>
            <button type="button">COPY_BUTTON_NOISE</button>
          </figure>
          <figure hidden><img src="/h1.png" alt="H1"><figcaption>HIDDEN_FIGURE_ATTR</figcaption></figure>
          <figure style="display:none"><img src="/h2.png" alt="H2"><figcaption>HIDDEN_FIGURE_STYLE</figcaption></figure>
          <figure class="sr-only"><img src="/h3.png" alt="H3"><figcaption>HIDDEN_FIGURE_SR</figcaption></figure>
          <figure class="ad-banner"><img src="/ad.png" alt="Ad"><figcaption>AD_FIGURE_NOISE</figcaption></figure>
          <figure class="related-stories"><img src="/rel.png" alt="Rel"><figcaption>RELATED_FIGURE_NOISE</figcaption></figure>
          <figure class="sponsored-card"><img src="/sp.png" alt="Sp"><figcaption>SPONSORED_FIGURE_NOISE</figcaption></figure>
          <div class="recommended"><figure><img src="/rec.png" alt="Rec"><figcaption>RECOMMENDED_CONTAINER_NOISE</figcaption></figure></div>
          <div class="cta_caption"><p>OUTSIDE_CTA_CAPTION_NOISE</p></div>
        </article>
        <aside class="sidebar"><figure><img src="/side.png" alt="Side"><figcaption>SIDEBAR_NOISE</figcaption></figure></aside>
        </main></body></html>"#
    );
    let doc = process(&html, &all_opts("https://example.org/post"));
    assert!(
        doc.markdown
            .contains("![Valid chart](https://example.org/valid.png)")
    );
    assert!(doc.markdown.contains("VALID_CAPTION"));
    for gone in [
        "NAV_NOISE",
        "HIDDEN_CAPTION_ATTR",
        "HIDDEN_STYLE_CHILD",
        "HIDDEN_ARIA_ICON",
        "COPY_BUTTON_NOISE",
        "HIDDEN_FIGURE_ATTR",
        "HIDDEN_FIGURE_STYLE",
        "HIDDEN_FIGURE_SR",
        "AD_FIGURE_NOISE",
        "RELATED_FIGURE_NOISE",
        "SPONSORED_FIGURE_NOISE",
        "RECOMMENDED_CONTAINER_NOISE",
        "OUTSIDE_CTA_CAPTION_NOISE",
        "SIDEBAR_NOISE",
    ] {
        assert!(
            !doc.markdown.contains(gone),
            "{gone} leaked into markdown:\n{}",
            doc.markdown
        );
    }
    let plain = doc.plain_text.as_deref().unwrap();
    let clean = doc.html.as_deref().unwrap();
    for out in [plain, clean] {
        assert!(
            out.contains("VALID_CAPTION"),
            "missing VALID_CAPTION:\n{out}"
        );
        for gone in [
            "NAV_NOISE",
            "HIDDEN_CAPTION_ATTR",
            "HIDDEN_STYLE_CHILD",
            "HIDDEN_FIGURE_ATTR",
            "HIDDEN_FIGURE_STYLE",
            "AD_FIGURE_NOISE",
            "RELATED_FIGURE_NOISE",
            "SPONSORED_FIGURE_NOISE",
            "RECOMMENDED_CONTAINER_NOISE",
            "SIDEBAR_NOISE",
        ] {
            assert!(!out.contains(gone), "{gone} leaked into output:\n{out}");
        }
    }
}

#[test]
fn cross_format_full_mode_and_relaxed_fallback_consistency() {
    let prose = long_prose();
    let html = format!(
        r#"<!doctype html><html><head><title>Cross Format</title>{ARTICLE_META}</head><body>
        <article>
          <h1>Cross Format</h1>
          {prose}
          <figure>
            <img src="/img/fig.png" alt="Measured throughput">
            <figcaption>Figure 1: Throughput scales linearly across workers.</figcaption>
          </figure>
        </article></body></html>"#
    );
    let doc = process(&html, &all_opts("https://example.org/cross"));
    assert!(
        doc.markdown
            .contains("![Measured throughput](https://example.org/img/fig.png)"),
        "{}",
        doc.markdown
    );
    assert!(
        doc.markdown
            .contains("Figure 1: Throughput scales linearly across workers."),
        "{}",
        doc.markdown
    );
    let plain = doc.plain_text.as_deref().unwrap();
    assert!(plain.contains("Figure 1: Throughput scales linearly across workers."));
    let clean = doc.html.as_deref().unwrap();
    assert!(clean.contains("<figure>") && clean.contains("<figcaption>"));
    assert!(clean.contains("https://example.org/img/fig.png"));
    let images = doc.images.as_ref().unwrap();
    assert!(
        images
            .iter()
            .any(|i| i.src == "https://example.org/img/fig.png"
                && i.alt.as_deref() == Some("Measured throughput")),
        "{images:?}"
    );

    let full = process(
        &html,
        &Options {
            only_main_content: false,
            ..all_opts("https://example.org/cross")
        },
    );
    assert!(
        full.markdown
            .contains("Figure 1: Throughput scales linearly across workers.")
    );
    assert!(
        full.markdown
            .contains("![Measured throughput](https://example.org/img/fig.png)")
    );

    // 宽松回退路径（列表为主的文章主体触发 render_relaxed，依然保留合法 figure/figcaption）
    let links: String = (0..40)
        .map(|i| format!("<li><a href=\"/item/{i}\">Index entry number {i} in collection</a></li>"))
        .collect();
    let list_article = format!(
        r#"<!doctype html><html><head><title>Index Article</title>{ARTICLE_META}</head><body>
        <header><nav><a href="/">Home</a></nav></header>
        <main><article>
          <h1>Index Article</h1>
          <p>Short introduction to the index.</p>
          <figure><img src="/img/index.png" alt="Index overview"><figcaption>Figure 1: Index overview.</figcaption></figure>
          <ul>{links}</ul>
        </article></main></body></html>"#
    );
    let relaxed_doc = process(&list_article, &opts("https://example.org/index"));
    assert!(
        relaxed_doc.markdown.contains("Figure 1: Index overview."),
        "{}",
        relaxed_doc.markdown
    );
    assert!(
        relaxed_doc
            .markdown
            .contains("![Index overview](https://example.org/img/index.png)"),
        "{}",
        relaxed_doc.markdown
    );
    assert!(
        !relaxed_doc.markdown.contains("Home"),
        "{}",
        relaxed_doc.markdown
    );
}

#[test]
fn caption_heavy_article_keeps_strict_mode_and_has_no_partial_content_warning() {
    // 正文约 850 字符，figure/table/figcaption 合计超过 1100 字符：
    // 修复后 prose() 须准确计入 figure/figcaption/table，既不会因严格占比跌破 50% 而错误退回
    // 非严格整页输出（漏出 related-stories），也不会误报 partial_content。
    let short_prose = "<p>The study compares three retrieval pipelines under fixed memory and \
        latency budgets across academic PDFs, technical documentation, and news reports.</p>"
        .repeat(6);
    let long_caption = "Detailed experimental breakdown across all tested datasets, showing \
        per-category F1, p50 latency in milliseconds, p95 tail latency, and memory ceiling \
        compliance across single-thread and multi-worker configurations. "
        .repeat(5);
    let html = format!(
        r#"<!doctype html><html><head><title>Caption Heavy Study</title></head><body>
        <main><article>
          <h1>Caption Heavy Study</h1>
          {short_prose}
          <figure>
            <img src="/img/study.png" alt="Study results">
            <figcaption>{long_caption}</figcaption>
          </figure>
        </article>
        <div class="related-stories"><h3>More studies</h3><ul>
          <li><a href="/a">Unrelated article one</a></li>
          <li><a href="/b">Unrelated article two</a></li>
          <li><a href="/c">Unrelated article three</a></li>
        </ul></div></main></body></html>"#
    );
    let doc = process(&html, &opts("https://example.org/study"));
    assert!(
        doc.markdown.contains("per-category F1"),
        "caption missing:\n{}",
        doc.markdown
    );
    assert!(
        !doc.markdown.contains("Unrelated article one"),
        "fell back to full page and leaked related stories:\n{}",
        doc.markdown
    );
    assert!(
        doc.coverage.warnings().is_empty(),
        "unexpected coverage warning: {:?}",
        doc.coverage.warnings()
    );

    // 确实只选中了带图注的导语、丢失了外部大段正文时，partial_content 警告仍能正常触发
    let lead =
        "A short introductory summary that presents the initial setup and figure. ".repeat(6);
    let external_body = "<p>Full experimental methodology and remaining discussion in a separate container outside the article tag, providing detailed ablation tables and statistical error bounds.</p>".repeat(16);
    let partial_html = format!(
        r#"<!doctype html><html><head><title>Split Lead</title>{ARTICLE_META}</head><body>
        <article>
          <h1>Split Lead</h1>
          <p>{lead}</p>
          <figure><img src="/img/lead.png" alt="Lead"><figcaption>Figure 1: Lead overview.</figcaption></figure>
        </article>
        <div class="entry">{external_body}</div>
        </body></html>"#
    );
    let partial_doc = process(&partial_html, &opts("https://example.org/split"));
    assert!(partial_doc.markdown.contains("Figure 1: Lead overview."));
    assert!(
        partial_doc
            .coverage
            .warnings()
            .iter()
            .any(|w| w.starts_with("partial_content:")),
        "expected partial_content warning, got {:?}",
        partial_doc.coverage.warnings()
    );
}
