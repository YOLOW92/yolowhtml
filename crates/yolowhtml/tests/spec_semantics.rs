//! 按 WHATWG HTML 标准里元素和属性的定义渲染：标准规定了它是什么意思、浏览器怎么呈现，
//! 抽取结果就该和这个定义一致，而不是按标签名猜。
//!
//! - `<caption>`（§4.9.2 "the table caption element"）：表格的标题，属于表格内容；
//! - `hidden="until-found"`（§6.1 "the hidden attribute"）：内容默认折叠，但可以被页内查找和
//!   片段导航展开，是页面正文的一部分（手风琴式的 FAQ、长文折叠段落）；只有 `hidden`
//!   （"hidden" 状态）才是不呈现；
//! - `<menu>`（§4.4.7）："一组命令组成的工具栏"，写法和 `<ul>` 一样是无序列表；`<dir>` 是
//!   已废弃的同义写法，呈现规则（§15.3.8）和 `<ul>` 相同。整页输出时按无序列表渲染；
//!   文章模式按 trafilatura 的清理规则整块删掉，这和标准"它是命令、不是正文"的定义一致；
//! - `<q>`（§4.5.7）：行内引语，引号由呈现层（§15.3.4 的 `quotes`）加上，作者不该自己写引号，
//!   所以去掉标签时要把引号补回来。

use yolowhtml::{Options, process};

const PARA: &str = "This paragraph is long enough to count as real prose for the extractor, \
    with several clauses, commas and a clear topic that a reader would keep.";

fn md(body: &str) -> String {
    process(
        &format!("<html><body><main><p>{PARA}</p>{body}</main></body></html>"),
        &Options::default(),
    )
    .markdown
}

#[test]
fn table_caption_is_kept_with_the_table() {
    let out = md("<table><caption>Table 1. Results by region</caption>\
         <tr><th>Region</th><th>Share</th></tr><tr><td>North</td><td>42%</td></tr></table>");
    let caption = out.find("Table 1. Results by region").expect(&out);
    let table = out.find("| Region | Share |").expect(&out);
    assert!(caption < table, "标题应在表格前面：{out}");
}

#[test]
fn hidden_until_found_content_is_kept() {
    let out = md(
        "<h2>FAQ</h2><div hidden=\"until-found\"><p>The collapsed answer explains the refund window.</p></div>",
    );
    assert!(
        out.contains("The collapsed answer explains the refund window."),
        "{out}"
    );
}

#[test]
fn hidden_until_found_is_case_insensitive() {
    let out = md("<div hidden=\"Until-Found\"><p>Collapsed but findable section text.</p></div>");
    assert!(
        out.contains("Collapsed but findable section text."),
        "{out}"
    );
}

#[test]
fn plain_hidden_content_is_still_dropped() {
    for attr in [
        "hidden",
        "hidden=\"\"",
        "hidden=\"hidden\"",
        "hidden=\"bogus\"",
    ] {
        let out = md(&format!(
            "<div {attr}><p>This text is not rendered at all.</p></div>"
        ));
        assert!(!out.contains("not rendered"), "{attr}: {out}");
    }
}

#[test]
fn menu_and_dir_are_unordered_lists() {
    for tag in ["menu", "dir"] {
        let html = format!(
            "<html><body><p>{PARA}</p><{tag}><li>First option</li><li>Second option</li></{tag}></body></html>"
        );
        let opts = Options {
            only_main_content: false,
            ..Options::default()
        };
        let out = process(&html, &opts).markdown;
        assert!(
            out.contains("- First option\n- Second option"),
            "{tag}: {out}"
        );
    }
}

#[test]
fn menu_toolbar_is_not_article_content() {
    let out = md("<menu><li>Share</li><li>Print</li></menu>");
    assert!(!out.contains("Share") && !out.contains("Print"), "{out}");
}

#[test]
fn inline_quotation_gets_its_quote_marks() {
    let out = md("<p>She said <q>the release ships on Friday</q> and left.</p>");
    assert!(
        out.contains("She said “the release ships on Friday” and left."),
        "{out}"
    );
}

#[test]
fn nested_quotations_alternate_quote_marks() {
    let out = md("<p>He wrote <q>she told me <q>not yet</q> twice</q> today.</p>");
    assert!(out.contains("“she told me ‘not yet’ twice”"), "{out}");
}
