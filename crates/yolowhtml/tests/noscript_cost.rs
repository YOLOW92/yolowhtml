//! `<noscript>` 兜底的代价：正文很短、页面有 `<noscript>` 时会拆掉这层标签把整页重新解析一遍。
//! 里面没有能让结果变长的内容（GTM 的追踪 iframe、"请开启 JavaScript" 提示）时不应该重解析整页。
//!
//! 建树计数器是进程级的，这个文件只放一个用例，免得和别的用例并发干扰。

use std::sync::atomic::Ordering::Relaxed;

use yolowhtml::yolowhtml_dom::{DOM_BUILD_BYTES, DOM_BUILDS};
use yolowhtml::{Options, process};

/// 单页应用外壳：大量内联脚本，正文只有一句话，`<noscript>` 放 `extra`。
fn shell(extra: &str) -> String {
    let mut s = String::from("<!doctype html><html><head><title>App</title>");
    for i in 0..200 {
        s.push_str(&format!(
            "<script>window.__d{i}={{b:'{}'}};</script>",
            "z".repeat(500)
        ));
    }
    s.push_str("</head><body>");
    s.push_str(extra);
    s.push_str("<div id=root><p>Loading…</p></div></body></html>");
    s
}

/// 处理一页，返回 Markdown 和解析过的字节数相对页面大小的倍数。
fn parsed_share(html: &str) -> (String, f64) {
    let (builds, bytes) = (DOM_BUILDS.load(Relaxed), DOM_BUILD_BYTES.load(Relaxed));
    let md = process(html, &Options::default()).markdown;
    assert!(DOM_BUILDS.load(Relaxed) > builds);
    let share = (DOM_BUILD_BYTES.load(Relaxed) - bytes) as f64 / html.len() as f64;
    (md, share)
}

const PARA: &str = "The scholarship supports students from economically weaker backgrounds with yearly awards, \
    and applications are free and fully online with direct bank transfer of the funds to every selected student.";

#[test]
fn noscript_without_useful_content_does_not_reparse_the_page() {
    let gtm = r#"<noscript><iframe src="https://www.googletagmanager.com/ns.html?id=GTM-XXXX" height="0" width="0" style="display:none;visibility:hidden"></iframe></noscript>"#;
    let hint = "<noscript><p>You need to enable JavaScript to run this app.</p></noscript>";
    let pixel = r#"<noscript><img height="1" width="1" style="display:none" src="https://www.facebook.com/tr?id=1&ev=PageView&noscript=1"/></noscript>"#;
    let (baseline, _) = parsed_share(&shell(""));
    for extra in [gtm, hint, pixel, &format!("{gtm}{hint}{pixel}")] {
        let (md, share) = parsed_share(&shell(extra));
        assert_eq!(md, baseline, "{extra}");
        assert!(
            share < 1.1,
            "整页被重新解析（解析了 {share:.2} 倍页面大小）：{extra}"
        );
    }

    // 有真正内容时照旧拆开重抽
    let real = format!(
        "<noscript><main><h1>Asha Scholarship</h1><p>{PARA}</p><ul><li>School students</li></ul></main></noscript>"
    );
    let (md, share) = parsed_share(&shell(&real));
    assert!(md.contains("# Asha Scholarship"), "{md}");
    assert!(share >= 2.0, "{share}");
}
