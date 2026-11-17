//! 分词和建树合在一起：分词器用 memchr 在源字符串里跳着找 `<`，直接把标签、文本交给建树器，
//! 中间不产生 token 对象。建树器实现 HTML5 树构建规则里影响文本结构的部分：
//! 隐式的 html/head/body、`<p>` `<li>` `<dd>` `<option>` 等的自动闭合、作用域边界、
//! 表格的隐式 tbody/tr 和 foster parenting、SVG/MathML 外部内容和跳出规则、
//! raw text 和 RCDATA 元素、`<pre>` 开头的换行、脚本的转义状态、quirks 模式（只影响
//! `<table>` 是否关掉 `<p>`）、select 和列组的内容规则。
//! 和规范有意不同的：`<template>` 不建独立的文档片段（内容作为普通子节点）；插入模式按当前节点
//! 推断，不单独记录（frameset 文档的三种模式除外）。和 html5lib-tests 的逐条对照见 `tests/html5lib`。

use std::borrow::Cow;

use crate::Tag;
use crate::dom::{
    Attr, Data, Html, MAX_BUF_LEN, MAX_INPUT_LEN, NONE, Namespace, Node, ParseOptions,
    ParseScratch, Span,
};

/// 打开元素栈的最大深度。更深的元素照样插入但不入栈，树的深度因此有上限，
/// 递归遍历这棵树的代码不会因为恶意的深层嵌套栈溢出。
const MAX_DEPTH: usize = 512;

pub(crate) fn parse(html: &str) -> Html<'_> {
    parse_with_scratch(html, &mut ParseScratch::default())
}

pub(crate) fn parse_with_scratch<'s>(html: &'s str, scratch: &mut ParseScratch) -> Html<'s> {
    parse_with_options(html, scratch, &ParseOptions::default())
}

/// 进程内建树的总次数（`count-builds` feature）。
#[cfg(feature = "count-builds")]
pub static BUILDS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
/// 进程内建树累计解析的输入字节数（`count-builds` feature）。
#[cfg(feature = "count-builds")]
pub static BUILD_BYTES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub(crate) fn parse_with_options<'s>(
    html: &'s str,
    scratch: &mut ParseScratch,
    opts: &ParseOptions<'_>,
) -> Html<'s> {
    parse_inner(html, scratch, opts, Limits::default())
}

/// 源字符串和解码缓冲区的长度上限（单元测试里调小）。
#[derive(Clone, Copy)]
struct Limits {
    input: usize,
    buf: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            input: MAX_INPUT_LEN,
            buf: MAX_BUF_LEN,
        }
    }
}

fn parse_inner<'s>(
    html: &'s str,
    scratch: &mut ParseScratch,
    opts: &ParseOptions<'_>,
    limits: Limits,
) -> Html<'s> {
    #[cfg(feature = "count-builds")]
    {
        BUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        BUILD_BYTES.fetch_add(html.len(), std::sync::atomic::Ordering::Relaxed);
    }
    // 先按上限截一次，免得为超大输入做完整的规范化拷贝；规范化可能变长（NUL 换成 3 字节的
    // U+FFFD），之后再截一次
    let (html, cut_raw) = clamp_str(html, limits.input);
    let (src, mut nuls) = normalize(html);
    let (src, cut_src) = clamp(src, limits.input);
    nuls.retain(|&at| (at as usize) < src.len());
    let (buf, nodes, attrs, cut_buf) = {
        let mut tb = Builder::with_buffers(
            &src,
            std::mem::take(&mut scratch.buf),
            std::mem::take(&mut scratch.nodes),
            std::mem::take(&mut scratch.attrs),
        );
        tb.nuls = nuls;
        tb.buf_limit = limits.buf;
        tb.scripting = opts.scripting;
        match opts.context {
            Some((ns, name)) => fragment::run(&src, &mut tb, ns, name),
            None => tokenize(&src, &mut tb),
        }
        (tb.buf, tb.nodes, tb.attrs, tb.truncated)
    };
    Html {
        src,
        buf,
        nodes,
        attrs,
        truncated: cut_raw || cut_src || cut_buf,
    }
}

/// 截到不超过 `limit` 字节的最近字符边界。返回是否截掉了内容。
fn clamp_str(s: &str, limit: usize) -> (&str, bool) {
    if s.len() <= limit {
        return (s, false);
    }
    let mut cut = limit;
    while !s.is_char_boundary(cut) {
        cut -= 1;
    }
    (&s[..cut], true)
}

fn clamp(src: Cow<'_, str>, limit: usize) -> (Cow<'_, str>, bool) {
    match src {
        Cow::Borrowed(s) => {
            let (s, cut) = clamp_str(s, limit);
            (Cow::Borrowed(s), cut)
        }
        Cow::Owned(mut s) => {
            let keep = clamp_str(&s, limit).0.len();
            let cut = keep < s.len();
            s.truncate(keep);
            (Cow::Owned(s), cut)
        }
    }
}

/// NUL 占全部字节的比例不低于 1/`UTF16_NUL_SHARE` 时，当作被误当成 UTF-8 的 UTF-16 文本，
/// NUL 全部删掉（每个 ASCII 字符旁边一个 NUL，删掉后能恢复出文字）。
const UTF16_NUL_SHARE: usize = 4;
/// 太短的输入不按 UTF-16 猜（几个字节里一个 NUL 就超过比例）。
const UTF16_MIN_LEN: usize = 64;

/// 去掉 BOM，`\r\n` 和单独的 `\r` 变成 `\n`。NUL 按规范换成 U+FFFD（分词器各状态里都是这样），
/// 换掉的位置记下来：正文（data 状态的 HTML 内容）里规范要求忽略 NUL，建树时按位置去掉。
/// 看起来是误判成 UTF-8 的 UTF-16 文本时 NUL 直接删掉。
fn normalize(html: &str) -> (Cow<'_, str>, Vec<u32>) {
    let html = html.strip_prefix('\u{feff}').unwrap_or(html);
    let bytes = html.as_bytes();
    let Some(first) = memchr::memchr2(b'\r', b'\0', bytes) else {
        return (Cow::Borrowed(html), Vec::new());
    };
    let drop_nul = bytes.len() >= UTF16_MIN_LEN
        && memchr::memchr(b'\0', &bytes[first..]).is_some()
        && memchr::memchr_iter(b'\0', bytes).count() * UTF16_NUL_SHARE >= bytes.len();
    let mut out = String::with_capacity(html.len());
    let mut nuls = Vec::new();
    let mut last = 0;
    let mut i = first;
    loop {
        out.push_str(&html[last..i]);
        if bytes[i] == b'\r' {
            out.push('\n');
            if bytes.get(i + 1) == Some(&b'\n') {
                i += 1;
            }
        } else if !drop_nul {
            nuls.push(out.len() as u32);
            out.push('\u{fffd}');
        }
        last = i + 1;
        match memchr::memchr2(b'\r', b'\0', &bytes[last..]) {
            Some(off) => i = last + off,
            None => break,
        }
    }
    out.push_str(&html[last..]);
    (Cow::Owned(out), nuls)
}

mod end;
mod format;
mod fragment;
mod frameset;
mod noscript;
mod quirks;
mod script;
mod tags;
mod tokenize;
mod tree;

use tokenize::{Next, RawAttr, is_ws, tokenize};

// ---------------------------------------------------------------- 建树

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Mode {
    Initial,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    /// frameset 文档的三种模式，见 `frameset` 模块。
    InFrameset,
    AfterFrameset,
    AfterAfterFrameset,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ns {
    Html,
    /// SVG / MathML 元素。
    Foreign,
    /// SVG 的 foreignObject、desc、title，MathML 的 mi、mo、mn、ms、mtext，`encoding` 是 HTML 的
    /// annotation-xml：外部内容里重新按 HTML 解析的地方。
    Integration,
    /// 其他 annotation-xml：内容仍是 MathML，但和集成点一样是"特殊"元素、作用域边界。
    Annotation,
}

impl Ns {
    /// 按外部内容的规则分词、建树（SVG/MathML 元素，集成点除外）。
    fn foreign(self) -> bool {
        matches!(self, Ns::Foreign | Ns::Annotation)
    }
}

#[derive(Clone, Copy)]
struct Open {
    id: u32,
    tag: Tag,
    ns: Ns,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    Default,
    ListItem,
    Button,
    Table,
}

struct Builder<'a> {
    src: &'a str,
    buf: String,
    nodes: Vec<Node>,
    attrs: Vec<Attr>,
    stack: Vec<Open>,
    mode: Mode,
    html: u32,
    head: u32,
    body: u32,
    /// `<pre>`、`<listing>`、`<textarea>` 之后紧跟的一个换行不算内容。
    skip_lf: bool,
    /// 分词器放属性位置的地方，每个标签复用。
    scratch: Vec<RawAttr>,
    lower: String,
    /// 活动格式元素列表（`<b>`、`<a>` 等），[`NONE`] 是标记（单元格、`<object>` 等的边界）。
    afe: Vec<u32>,
    /// 当前打开的 `<form>`；打开时再出现的 `<form>` 被忽略。
    form: u32,
    /// quirks 模式（没有 DOCTYPE 或老式 DOCTYPE），见 `quirks` 模块。
    quirks: bool,
    /// 规范的 frameset-ok 标志：body 里出现正文或某些元素之后，`<frameset>` 不再取代 body。
    frameset_ok: bool,
    /// 源字符串里由 NUL 换成的 U+FFFD 的位置（升序），见 [`normalize`]。
    nuls: Vec<u32>,
    /// 规范的 scripting flag，见 `noscript` 模块。
    scripting: bool,
    /// 片段解析的上下文元素（`id` 不用）和它的命名空间，见 `fragment` 模块。
    context: Option<Open>,
    context_ns: Namespace,
    /// 出栈的下限：片段解析时根元素不出栈。
    floor: usize,
    /// 解码缓冲区的长度上限（[`MAX_BUF_LEN`]，单元测试里调小）。
    buf_limit: usize,
    /// 有内容因为超出缓冲区上限被丢弃，见 [`Builder::owned_span`]。
    truncated: bool,
}

fn formatting(tag: Tag) -> bool {
    use Tag::*;
    matches!(
        tag,
        A | B | Big | Code | Em | Font | I | Nobr | S | Small | Strike | Strong | Tt | U
    )
}

/// 入栈时在活动格式元素列表里放标记的元素。
fn marker(tag: Tag) -> bool {
    use Tag::*;
    matches!(
        tag,
        Applet | Caption | Marquee | Object | Td | Th | Template
    )
}

fn void(tag: Tag) -> bool {
    use Tag::*;
    matches!(
        tag,
        Area | Base
            | Basefont
            | Bgsound
            | Br
            | Col
            | Embed
            | Frame
            | Hr
            | Img
            | Input
            | Keygen
            | Link
            | Meta
            | Param
            | Source
            | Track
            | Wbr
    )
}

fn special(tag: Tag) -> bool {
    use Tag::*;
    matches!(
        tag,
        Address
            | Applet
            | Area
            | Article
            | Aside
            | Base
            | Basefont
            | Bgsound
            | Blockquote
            | Body
            | Br
            | Button
            | Caption
            | Center
            | Col
            | Colgroup
            | Dd
            | Details
            | Dir
            | Div
            | Dl
            | Dt
            | Embed
            | Fieldset
            | Figcaption
            | Figure
            | Footer
            | Form
            | Frame
            | Frameset
            | H1
            | H2
            | H3
            | H4
            | H5
            | H6
            | Head
            | Header
            | Hgroup
            | Hr
            | Html
            | Iframe
            | Img
            | Input
            | Keygen
            | Li
            | Link
            | Listing
            | Main
            | Marquee
            | Menu
            | Meta
            | Nav
            | Noembed
            | Noframes
            | Noscript
            | Object
            | Ol
            | P
            | Param
            | Plaintext
            | Pre
            | Script
            | Search
            | Section
            | Select
            | Source
            | Style
            | Summary
            | Table
            | Tbody
            | Td
            | Template
            | Textarea
            | Tfoot
            | Th
            | Thead
            | Title
            | Tr
            | Track
            | Ul
            | Wbr
            | Xmp
    )
}

fn heading(tag: Tag) -> bool {
    matches!(
        tag,
        Tag::H1 | Tag::H2 | Tag::H3 | Tag::H4 | Tag::H5 | Tag::H6
    )
}

/// 当前节点是这些元素时，非表格内容要 foster parenting 到表格前面。
fn table_context(tag: Tag) -> bool {
    matches!(
        tag,
        Tag::Table | Tag::Tbody | Tag::Thead | Tag::Tfoot | Tag::Tr
    )
}

/// 外部内容里遇到这些 HTML 标签时跳出 SVG/MathML。
fn breaks_out_of_foreign(tag: Tag) -> bool {
    use Tag::*;
    matches!(
        tag,
        B | Big
            | Blockquote
            | Body
            | Br
            | Center
            | Code
            | Dd
            | Div
            | Dl
            | Dt
            | Em
            | Embed
            | H1
            | H2
            | H3
            | H4
            | H5
            | H6
            | Head
            | Hr
            | I
            | Img
            | Li
            | Listing
            | Menu
            | Meta
            | Nobr
            | Ol
            | P
            | Pre
            | Ruby
            | S
            | Small
            | Span
            | Strong
            | Strike
            | Sub
            | Sup
            | Table
            | Tt
            | U
            | Ul
            | Var
    )
}

/// 外部元素在栈上的类别：HTML 集成点、MathML 文本集成点、annotation-xml 或普通外部元素。
/// `encoding_html` 是元素的 `encoding` 属性为 `text/html` 或 `application/xhtml+xml`。
fn foreign_kind(ns: Namespace, name: &str, encoding_html: bool) -> Ns {
    let is = |n: &str| name.eq_ignore_ascii_case(n);
    match ns {
        Namespace::Svg if is("foreignobject") || is("desc") || is("title") => Ns::Integration,
        Namespace::MathMl if is("mi") || is("mo") || is("mn") || is("ms") || is("mtext") => {
            Ns::Integration
        }
        Namespace::MathMl if is("annotation-xml") => {
            if encoding_html {
                Ns::Integration
            } else {
                Ns::Annotation
            }
        }
        _ => Ns::Foreign,
    }
}

impl<'a> Builder<'a> {
    fn with_buffers(
        src: &'a str,
        mut buf: String,
        mut nodes: Vec<Node>,
        mut attrs: Vec<Attr>,
    ) -> Self {
        buf.clear();
        nodes.clear();
        attrs.clear();
        let node_capacity = src.len() / 40 + 16;
        if nodes.capacity() < node_capacity {
            nodes.reserve(node_capacity - nodes.capacity());
        }
        let attr_capacity = src.len() / 80 + 16;
        if attrs.capacity() < attr_capacity {
            attrs.reserve(attr_capacity - attrs.capacity());
        }
        nodes.push(Node::new(Data::Document));
        Builder {
            src,
            buf,
            nodes,
            attrs,
            stack: Vec::with_capacity(64),
            mode: Mode::Initial,
            html: NONE,
            head: NONE,
            body: NONE,
            skip_lf: false,
            scratch: Vec::new(),
            lower: String::new(),
            afe: Vec::new(),
            form: NONE,
            quirks: true,
            frameset_ok: true,
            nuls: Vec::new(),
            scripting: true,
            context: None,
            context_ns: Namespace::Html,
            floor: 0,
            buf_limit: MAX_BUF_LEN,
            truncated: false,
        }
    }

    /// 把这次写进缓冲区的 `buf[start..]` 收成一段 [`Span`]。超出缓冲区上限时撤回这次写入、
    /// 返回空段并记下丢了内容：Span 的起点只有 31 位，越界的位置会被读成别的文字。
    pub(super) fn owned_span(&mut self, start: usize) -> Span {
        if self.buf.len() > self.buf_limit {
            self.buf.truncate(start);
            self.truncated = true;
            return Span::owned(start, start);
        }
        Span::owned(start, self.buf.len())
    }

    pub(super) fn str(&self, s: Span) -> &str {
        if s.is_owned() {
            &self.buf[s.offset()..s.end()]
        } else {
            &self.src[s.offset()..s.end()]
        }
    }

    /// 规范的 "adjusted current node" 是外部元素（片段解析时栈里只剩根元素就看上下文元素）。
    pub(super) fn in_foreign(&self) -> bool {
        self.current().is_some_and(|o| o.ns.foreign())
    }

    /// 栈顶真的是外部元素（跳出外部内容时按它逐个出栈）。
    pub(super) fn top_foreign(&self) -> bool {
        self.stack.last().is_some_and(|o| o.ns.foreign())
    }

    /// 当前节点。片段解析时栈里只剩根元素，标签和类别取上下文元素的（规范的 "adjusted current
    /// node"，也用来推断插入模式），插入位置仍是根元素。
    pub(super) fn current(&self) -> Option<Open> {
        let top = self.stack.last().copied();
        match (top, self.context) {
            (Some(o), Some(c)) if self.stack.len() == 1 => Some(Open {
                id: o.id,
                tag: c.tag,
                ns: c.ns,
            }),
            _ => top,
        }
    }

    pub(super) fn current_is(&self, tag: Tag) -> bool {
        self.current()
            .is_some_and(|o| o.tag == tag && !o.ns.foreign())
    }
}

#[cfg(test)]
mod limit_tests {
    use super::*;

    fn parse_limited(html: &str, input: usize, buf: usize) -> Html<'_> {
        let limits = Limits { input, buf };
        parse_inner(
            html,
            &mut ParseScratch::default(),
            &ParseOptions::default(),
            limits,
        )
    }

    fn texts(dom: &Html<'_>) -> Vec<String> {
        dom.root()
            .descendants()
            .filter(|n| n.tag() == Some(Tag::P))
            .map(|n| n.text())
            .collect()
    }

    #[test]
    fn normal_input_is_not_truncated() {
        let dom = Html::parse("<p>a &amp; b</p>");
        assert!(!dom.is_truncated());
        assert_eq!(texts(&dom), ["a & b"]);
    }

    #[test]
    fn input_past_the_limit_is_cut() {
        let dom = parse_limited("<p>hello world</p>", 10, MAX_BUF_LEN);
        assert!(dom.is_truncated());
        assert_eq!(texts(&dom), ["hello w"]);
    }

    #[test]
    fn input_cut_lands_on_a_char_boundary() {
        // "<p>" 3 字节，"日" 3 字节：上限 5 落在 "日" 中间，退到 3
        let dom = parse_limited("<p>日本</p>", 5, MAX_BUF_LEN);
        assert!(dom.is_truncated());
        assert!(texts(&dom).iter().all(|t| t.is_empty()));
    }

    #[test]
    fn input_limit_applies_after_normalization() {
        // NUL 换成 3 字节的 U+FFFD，规范化后变长；上限按规范化后的长度算
        let dom = parse_limited("<p>\0\0\0\0</p>", 8, MAX_BUF_LEN);
        assert!(dom.is_truncated());
        assert!(dom.src.len() <= 8);
    }

    #[test]
    fn decoded_text_past_the_buffer_limit_is_dropped() {
        // 第一段解码后 3 字节正好装满，第二段装不下：丢弃并报告，第一段不受影响
        let dom = parse_limited("<p>a&amp;b</p><p>c&amp;d</p>", usize::MAX, 3);
        assert!(dom.is_truncated());
        assert_eq!(texts(&dom), ["a&b", ""]);
    }

    #[test]
    fn merged_text_past_the_buffer_limit_keeps_the_first_part() {
        // 多余的结束标签被忽略，前后两段文字合并成一个节点，后半段装不下时保留前半段
        let dom = parse_limited("<p>a&amp;b</span>cd</p>", usize::MAX, 3);
        assert!(dom.is_truncated());
        assert_eq!(texts(&dom), ["a&b"]);
    }

    #[test]
    fn attribute_values_past_the_buffer_limit_are_emptied() {
        let dom = parse_limited(r#"<a href="?a=1&amp;b=2">x</a>"#, usize::MAX, 3);
        assert!(dom.is_truncated());
        let a = dom
            .root()
            .descendants()
            .find(|n| n.tag() == Some(Tag::A))
            .unwrap();
        assert_eq!(a.as_element().unwrap().attr("href"), Some(""));
    }
}
