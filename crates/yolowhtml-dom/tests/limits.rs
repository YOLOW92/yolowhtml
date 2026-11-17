//! 超大输入：源字符串和解码缓冲区都用 u32 记位置（最高位是“指向解码缓冲区”的标记），
//! 超过上限的部分必须丢弃并报告，绝不能静默读成别的文字。
//!
//! 这两个用例要分配 1～2 GiB，默认不跑：
//! `cargo test --profile quick -p yolowhtml-dom --test limits -- --ignored`

use yolowhtml_dom::{Html, MAX_INPUT_LEN, Selector};

/// 开头一段 `<p>` 里放 1000 个字符引用（解码进缓冲区），用注释垫到 `target`，再放一段 `<p>`。
fn padded(target: usize, tail: &str) -> String {
    let mut s = String::from("<p>");
    for _ in 0..1000 {
        s.push_str("&amp;");
    }
    s.push_str("</p><!--");
    let pad = target - s.len() - "--><p>".len();
    s.reserve(pad + tail.len() + 16);
    s.extend(std::iter::repeat_n('x', pad));
    s.push_str("--><p>");
    s.push_str(tail);
    s
}

fn paragraphs(dom: &Html<'_>) -> Vec<String> {
    let sel = Selector::parse("p").unwrap();
    dom.select(&sel).map(|n| n.text()).collect()
}

#[test]
#[ignore = "分配约 2 GiB"]
fn text_past_two_gib_is_dropped_not_misread() {
    // 第二段文字的源偏移是 2^31 + 100：修复前它的最高位被当成标记，读出 11 个 '&'
    let html = padded((1 << 31) + 100, "hello world</p>");
    let dom = Html::parse(&html);
    assert!(dom.is_truncated());
    let texts = paragraphs(&dom);
    assert_eq!(texts.len(), 1, "超过上限的段落应当被丢弃");
    assert_eq!(texts[0], "&".repeat(1000));
}

#[test]
#[ignore = "分配约 1 GiB"]
fn input_at_the_limit_is_kept_whole() {
    let tail = "hello world</p>";
    let html = padded(MAX_INPUT_LEN - tail.len(), tail);
    assert_eq!(html.len(), MAX_INPUT_LEN);
    let dom = Html::parse(&html);
    assert!(!dom.is_truncated());
    assert_eq!(paragraphs(&dom)[1], "hello world");
}
