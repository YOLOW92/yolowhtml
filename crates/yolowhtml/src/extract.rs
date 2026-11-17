//! 只取正文时的跳过规则：去掉导航、页眉页脚、侧边栏、链接密集块这类页面框架。

use std::cell::{Cell, OnceCell};

use yolowhtml_dom::{Element, Html, Tag};

use crate::dom::{NodeR, element, is_hidden, is_non_content_tag, short_text};
use crate::markdown::is_block_tag;

use crate::classes::{JUMP, NEG, POS, SKIP_ROLES, STRICT, is_main_landmark, token_class};

/// 正文被删过头时放宽规则的程度。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Relax {
    None,
    /// 软跳过规则只留链接块：菜单、页脚链接还删，按类名、role、页头删的块放回来。
    LinkBlocks,
    /// 只保留硬跳过和 `<nav>`、`<aside>`、`<footer>`。
    All,
    /// 在 `All` 的基础上连隐藏元素也保留：服务端渲染的单页应用常把整页内容包在 `display:none` 里
    /// 等脚本显示。只在前面几级的结果几乎为空时才用。
    Hidden,
}

/// 决定渲染时要整棵跳过的子树。
///
/// 两类规则：
/// - 硬跳过：脚本、样式、隐藏元素，以及只取正文时的 `<nav>`、`<aside>`、`<footer>`；
/// - 软跳过（只取正文时）：带导航的 `<header>`、导航类 `role`、类名或 id 命中噪声词、
///   链接密集的块（没打标签的菜单、相关推荐、页脚链接）。
///   软跳过的元素如果装着全页大部分文字，说明它其实是正文的外层容器，不跳过。
pub(crate) struct Skipper {
    only_main: bool,
    /// 页面里可能有公式（源码里出现过 math、MathJax、mjx-）：没有时渲染器不逐个元素检查公式。
    has_math: bool,
    /// 文章模式：文章主体里也删链接列表，评论、标签、署名这类块也算框架。
    strict: bool,
    /// 文章模式下，按严格规则删掉的块最多能有多少非链接文字（主体的一部分）：大块的不删，
    /// 免得类名凑巧命中（`meta`、`slide`）把正文删掉。
    strict_max_chars: usize,

    relaxed: Relax,
    total_chars: usize,
    protect_chars: usize,
    /// 按节点下标索引；只取正文时才计算。
    visible: Vec<Visible>,
    /// 子树里（不含自身）有没有块级元素，按节点下标索引。
    has_block: Vec<bool>,
    /// 文字节点的非空白字符数，构造时数一次，后面各遍按下标直接查（[`Self::text_chars`]）；
    /// 硬跳过子树里的文字和整页模式下不数，记为 [`UNCOUNTED_TEXT`]。
    text_counts: Vec<u32>,
    /// 跳过判断的缓存，两种模式各占两位（见 [`Verdict`]）：低两位是默认规则，0 未算、1 不跳过、
    /// 2 跳过；高两位是文章模式，另有 3 表示"只在块足够小时跳过"。文章模式的结论和主体大小
    /// （[`Self::strict_max_chars`]）无关，只有第 3 种要按当前上限比一次，所以换文章主体、
    /// 在两种模式间切换都不用清缓存。
    cache: Vec<Cell<u8>>,
    /// class/id 的分类（[`token_class`]），和模式无关，每个节点只算一次；[`UNKNOWN`] 表示还没算。
    classes: Vec<Cell<u8>>,
    /// [`crate::features`] 的特征位，每个节点只算一次；`u32::MAX` 表示还没算。
    features: Vec<Cell<u32>>,
    attachments: Vec<Cell<u8>>,
    order: Vec<yolowhtml_dom::NodeId>,
    /// 先序区间编号和标题表（[`crate::outline`]）：只有文章模式找标题时才用到，第一次查询时才建，
    /// 不走文章模式的页面不多遍历这一遍。`outline_spare` 是从工作区拿来、等着复用的分配。
    outline: OnceCell<crate::outline::Outline>,
    outline_spare: Cell<Option<crate::outline::Outline>>,
    /// 在不在正文区、子树里有没有 nav：按需计算并记住。
    landmarks: OnceCell<crate::landmarks::Landmarks>,
}

/// 一个元素的跳过结论（见 [`Skipper::verdict`]），带着原因供 [`Skipper::reason`] 诊断用。
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Verdict {
    Keep,
    Skip(&'static str),
    /// 文章模式的"文章杂项"：块的非链接文字不超过主体上限时才跳过。
    IfSmall(&'static str),
}

impl Verdict {
    const KEEP: u8 = 1;
    const SKIP: u8 = 2;
    const IF_SMALL: u8 = 3;

    fn code(self) -> u8 {
        match self {
            Verdict::Keep => Self::KEEP,
            Verdict::Skip(_) => Self::SKIP,
            Verdict::IfSmall(_) => Self::IF_SMALL,
        }
    }
}

impl From<Result<Option<&'static str>, &'static str>> for Verdict {
    fn from(r: Result<Option<&'static str>, &'static str>) -> Self {
        match r {
            Ok(None) => Verdict::Keep,
            Ok(Some(why)) => Verdict::Skip(why),
            Err(why) => Verdict::IfSmall(why),
        }
    }
}

/// Reusable per-worker vectors indexed by DOM node id.
#[derive(Default)]
pub(crate) struct Scratch {
    visible: Vec<Visible>,
    has_block: Vec<bool>,
    text_counts: Vec<u32>,
    cache: Vec<Cell<u8>>,
    classes: Vec<Cell<u8>>,
    features: Vec<Cell<u32>>,
    attachments: Vec<Cell<u8>>,
    order: Vec<yolowhtml_dom::NodeId>,
    outline: crate::outline::Outline,
}

/// 子树里可见文字的统计，只排除硬跳过的内容。构造 Skipper 时自底向上一次算好。
#[derive(Default, Clone, Copy)]
struct Visible {
    chars: usize,
    link_chars: usize,
    links: usize,
    images: u32,
    /// 包着图片的链接数（缩略图、卡片）。
    img_links: u32,
    /// 有文字的链接数，和其中文字不到 10 个字符的（"更多"、"分享"这类）。
    text_links: u32,
    short_links: u32,
}

const UNKNOWN: u8 = 0xFF;

/// 文章模式的严格规则只删非链接文字不到主体这个比例的块。
const STRICT_MAX_SHARE: f64 = 0.3;

/// 软跳过的元素文字量达到全页的这个比例时，视为正文容器。
const PROTECT_SHARE: f64 = 0.4;
/// 文字太少的页面不做保护，免得导航因为"占比高"被留下。
const PROTECT_MIN_CHARS: usize = 200;
/// `<nav>`、`<aside>`、`<footer>` 装着全页这个比例以上的文字、链接文字又不到这个比例时，
/// 是整页的外层容器，不当页面框架。
const WRAPPER_SHARE: f64 = 0.7;
const WRAPPER_MAX_LINK_SHARE: f64 = 0.5;
/// 链接密集块：至少这么多个链接，且链接文字占比不低于这个比例。
const LINK_BLOCK_MIN_LINKS: usize = 3;
const LINK_BLOCK_MIN_SHARE: f64 = 0.8;
/// 菜单的链接文字短，内容卡片的链接文字是整句标题：平均链接文字超过这个长度就不算链接块。
/// 一半以上的链接包着图片（相册、商品卡片）也不算。
const LINK_BLOCK_MAX_AVG_CHARS: f64 = 20.0;

/// "Skip to content"、"Jump to content" 这类页内跳转链接。
fn is_skip_link(node: &NodeR<'_>, el: Element<'_>) -> bool {
    if !el
        .attr("href")
        .is_some_and(|h| h.trim_start().starts_with('#'))
    {
        return false;
    }
    if token_class(el) & JUMP != 0 {
        return true;
    }
    let text: String = node
        .descendants()
        .filter_map(|d| d.as_text().map(|t| t.to_lowercase()))
        .collect();
    let text = text.trim_start();
    [
        "skip to",
        "skip navigation",
        "jump to",
        "跳到",
        "跳转到",
        "跳至",
    ]
    .iter()
    .any(|p| text.starts_with(p))
}

fn is_hard_skip(node: &NodeR<'_>) -> bool {
    node.tag().is_some_and(is_non_content_tag) && !is_text_button(node) || is_hidden(node)
}

/// 按钮文字超过这么多（非空白字符）就算正文；短的是"菜单""搜索""提交"这类控件。
const TEXT_BUTTON_MIN_CHARS: usize = 15;

/// 装着成段文字的按钮（手风琴标题、整块可点的卡片）：文字照常显示，算正文。
fn is_text_button(node: &NodeR<'_>) -> bool {
    node.tag() == Some(Tag::Button) && !short_text(node, TEXT_BUTTON_MIN_CHARS)
}

/// 手风琴、折叠块的标题（WAI-ARIA 的 disclosure 模式：`aria-expanded`、`aria-controls`
/// 指向它展开的内容），文字够长（同 [`TEXT_BUTTON_MIN_CHARS`]）：它是下面那段内容的小标题，
/// 和 `<details>` 的 `<summary>` 一样。文章模式删按钮时不删它。
pub(crate) fn is_disclosure_header(node: &NodeR<'_>) -> bool {
    is_text_button(node)
        && element(node).is_some_and(|el| {
            el.attr("aria-expanded").is_some() || el.attr("aria-controls").is_some()
        })
}

/// 自底向上一遍算出每个元素的可见文字统计和"有没有块级后代"。
fn compute(
    html: &Html<'_>,
    only_main: bool,
    mut visible: Vec<Visible>,
    mut has_block: Vec<bool>,
    mut text_counts: Vec<u32>,
    mut order: Vec<yolowhtml_dom::NodeId>,
) -> (
    Vec<Visible>,
    Vec<bool>,
    Vec<u32>,
    Vec<yolowhtml_dom::NodeId>,
) {
    let n = html.len();
    order.clear();
    order.extend(
        html.root()
            .descendants()
            .filter(|d| d.is_element())
            .map(|node| node.id()),
    );
    visible.clear();
    if only_main {
        visible.resize(n, Visible::default());
    }
    has_block.clear();
    has_block.resize(n, false);
    text_counts.clear();
    if only_main {
        text_counts.resize(n, UNCOUNTED_TEXT);
    }
    for id in order.iter().rev() {
        let node = html.get(*id);
        let id = node.id().index();
        let hard = only_main && is_hard_skip(&node);
        let mut v = Visible::default();
        let mut block = false;
        for child in node.children() {
            if let Some(t) = child.as_text() {
                if only_main && !hard {
                    let k = crate::dom::count_non_whitespace(t);
                    v.chars += k;
                    text_counts[child.id().index()] = k.min(UNCOUNTED_TEXT as usize - 1) as u32;
                }
                continue;
            }
            let Some(tag) = child.tag() else { continue };
            let c = child.id().index();
            block |= is_block_tag(tag) || has_block[c];
            if only_main && !hard {
                let cv = visible[c];
                v.chars += cv.chars;
                v.images += cv.images + u32::from(tag == Tag::Img);
                if tag == Tag::A {
                    v.link_chars += cv.chars;
                    v.links += 1;
                    v.img_links += u32::from(cv.images > 0);
                    v.text_links += u32::from(cv.chars > 0);
                    v.short_links += u32::from(cv.chars > 0 && cv.chars < 10);
                } else {
                    v.link_chars += cv.link_chars;
                    v.links += cv.links;
                    v.img_links += cv.img_links;
                    v.text_links += cv.text_links;
                    v.short_links += cv.short_links;
                }
            }
        }
        has_block[id] = block;
        if only_main {
            visible[id] = v;
        }
    }
    (visible, has_block, text_counts, order)
}

/// [`Skipper::text_counts`] 里"没有数过"的标记。
const UNCOUNTED_TEXT: u32 = u32::MAX;

impl Skipper {
    pub(crate) fn new_with_scratch(
        html: &Html<'_>,
        only_main: bool,
        scratch: &mut Scratch,
    ) -> Self {
        let body = html.body();
        let (visible, has_block, text_counts, order) = compute(
            html,
            only_main,
            std::mem::take(&mut scratch.visible),
            std::mem::take(&mut scratch.has_block),
            std::mem::take(&mut scratch.text_counts),
            std::mem::take(&mut scratch.order),
        );
        let total = visible.get(body.id().index()).map_or(0, |v| v.chars);
        let outline_spare = Cell::new(Some(std::mem::take(&mut scratch.outline)));
        let n = html.len();
        let mut cache = std::mem::take(&mut scratch.cache);
        cache.clear();
        cache.resize_with(n, || Cell::new(0));
        let mut classes = std::mem::take(&mut scratch.classes);
        classes.clear();
        classes.resize_with(n, || Cell::new(UNKNOWN));
        let mut features = std::mem::take(&mut scratch.features);
        features.clear();
        features.resize_with(n, || Cell::new(u32::MAX));
        let mut attachments = std::mem::take(&mut scratch.attachments);
        attachments.clear();
        attachments.resize_with(n, || Cell::new(0));
        Self {
            only_main,
            strict: false,
            strict_max_chars: 0,

            relaxed: Relax::None,
            has_math: true,
            total_chars: total,
            protect_chars: ((total as f64 * PROTECT_SHARE) as usize).max(PROTECT_MIN_CHARS),
            visible,
            has_block,
            text_counts,
            cache,
            classes,
            features,
            attachments,
            order,
            outline: OnceCell::new(),
            outline_spare,
            landmarks: OnceCell::new(),
        }
    }

    fn landmarks(&self, node: &NodeR<'_>) -> &crate::landmarks::Landmarks {
        self.landmarks
            .get_or_init(|| crate::landmarks::Landmarks::new(node.document().len()))
    }

    pub(crate) fn recycle(self, scratch: &mut Scratch) {
        if [
            self.visible.capacity(),
            self.has_block.capacity(),
            self.text_counts.capacity(),
            self.cache.capacity(),
            self.classes.capacity(),
            self.features.capacity(),
            self.attachments.capacity(),
            self.order.capacity(),
        ]
        .into_iter()
        .max()
        .unwrap_or(0)
            > 16 * 1024
        {
            *scratch = Scratch::default();
            return;
        }
        scratch.visible = self.visible;
        scratch.has_block = self.has_block;
        scratch.text_counts = self.text_counts;
        scratch.cache = self.cache;
        scratch.classes = self.classes;
        scratch.features = self.features;
        scratch.attachments = self.attachments;
        scratch.order = self.order;
        scratch.outline = self
            .outline
            .into_inner()
            .or_else(|| self.outline_spare.take())
            .unwrap_or_default();
    }

    /// 节点 class/id 的分类，带缓存。
    fn classes(&self, node: &NodeR<'_>, el: Element<'_>) -> u8 {
        let cell = &self.classes[node.id().index()];
        let c = cell.get();
        if c != UNKNOWN {
            return c;
        }
        // 小标题按自己文字生成的锚点 id（`id="related-work"`）不当类名看，见 is_text_slug
        let id = el.attr("id").filter(|id| {
            !(matches!(
                el.tag(),
                Tag::H1 | Tag::H2 | Tag::H3 | Tag::H4 | Tag::H5 | Tag::H6
            ) && crate::classes::is_text_slug(id, &node.text()))
        });
        let c = crate::classes::token_class_of(el.attr("class"), id);
        cell.set(c);
        c
    }

    /// 元素的 class/id 特征位（见 [`crate::features`]），带缓存；文本节点为 0。
    pub(crate) fn features(&self, node: &NodeR<'_>) -> u32 {
        let Some(el) = node.as_element() else {
            return 0;
        };
        let cell = &self.features[node.id().index()];
        let f = cell.get();
        if f != u32::MAX {
            return f;
        }
        let f = crate::features::compute(el);
        cell.set(f);
        f
    }

    /// class/id 是评论区。
    pub(crate) fn is_comments(&self, node: &NodeR<'_>) -> bool {
        self.features(node) & crate::features::COMMENTS != 0
    }

    /// 打开或关掉文章模式。`root_chars` 是文章主体的非链接文字数，用来给严格规则设上限。
    /// 判断缓存按模式分开存、并且和上限无关（见 [`Self::cache`]），这里不用清。
    pub(crate) fn set_strict(&mut self, strict: bool, root_chars: usize) {
        self.strict = strict;
        self.strict_max_chars = (root_chars as f64 * STRICT_MAX_SHARE) as usize;
    }

    /// 先序区间编号和标题表。
    pub(crate) fn outline(&self, html: &Html<'_>) -> &crate::outline::Outline {
        self.outline.get_or_init(|| {
            let mut o = self.outline_spare.take().unwrap_or_default();
            o.rebuild(html, &self.order);
            o
        })
    }

    /// 文字节点 `node`（内容是 `text`）的非空白字符数：构造时数过就直接查表，否则现数。
    pub(crate) fn text_chars(&self, node: &NodeR<'_>, text: &str) -> usize {
        match self.text_counts.get(node.id().index()) {
            Some(&k) if k != UNCOUNTED_TEXT => k as usize,
            _ => crate::dom::count_non_whitespace(text),
        }
    }

    /// 子树里不在链接里的可见文字数：正文的量，菜单和链接列表不算。
    pub(crate) fn content_chars(&self, node: &NodeR<'_>) -> usize {
        self.visible
            .get(node.id().index())
            .map_or(0, |v| v.chars.saturating_sub(v.link_chars))
    }

    /// 页面可见文字的总字符数（只排除硬跳过的内容）。
    pub(crate) fn total_chars(&self) -> usize {
        self.total_chars
    }

    /// 放宽软跳过规则，用于正文被删过头时的重试。
    pub(crate) fn relax(&mut self, level: Relax) {
        self.relaxed = level;
        self.cache.iter().for_each(|c| c.set(0));
    }

    pub(crate) fn skip(&self, node: &NodeR<'_>) -> bool {
        if !node.is_element() {
            return node.is_comment();
        }
        let cell = &self.cache[node.id().index()];
        let shift = if self.strict { 2 } else { 0 };
        let code = match (cell.get() >> shift) & 3 {
            0 => {
                let code = self.verdict(node).code();
                cell.set(cell.get() | code << shift);
                code
            }
            code => code,
        };
        match code {
            Verdict::KEEP => false,
            Verdict::SKIP => true,
            _ => self.strict_small(node),
        }
    }

    /// 文章模式下块小到可以按"文章杂项"删掉：非链接文字不超过当前主体的上限。
    fn strict_small(&self, node: &NodeR<'_>) -> bool {
        let vis = self
            .visible
            .get(node.id().index())
            .copied()
            .unwrap_or_default();
        self.strict && vis.chars.saturating_sub(vis.link_chars) <= self.strict_max_chars
    }

    /// 子树里（不含自身）有没有块级元素。
    pub(crate) fn has_block_descendant(&self, node: &NodeR<'_>) -> bool {
        self.has_block[node.id().index()]
    }

    /// 跳过这个元素的原因；不跳过时为 `None`。诊断时用它解释每个被删掉的块（平时的判断走
    /// [`Self::skip`] 的缓存，不需要原因字符串）。
    #[allow(dead_code)]
    pub(crate) fn reason(&self, node: &NodeR<'_>) -> Option<&'static str> {
        match self.verdict(node) {
            Verdict::Keep => None,
            Verdict::Skip(why) => Some(why),
            Verdict::IfSmall(why) => self.strict_small(node).then_some(why),
        }
    }

    /// 跳过判断，和文章主体的大小上限无关：最后一条"文章杂项"规则只给出 [`Verdict::IfSmall`]，
    /// 由调用方按当前上限比较。
    fn verdict(&self, node: &NodeR<'_>) -> Verdict {
        Verdict::from(self.verdict_inner(node))
    }

    fn verdict_inner(&self, node: &NodeR<'_>) -> Result<Option<&'static str>, &'static str> {
        let Some(el) = element(node) else {
            return Ok(None);
        };
        let tag = el.tag();
        if is_non_content_tag(tag) && !is_text_button(node) {
            return Ok(Some("non-content tag"));
        }
        if is_hidden(node) && self.relaxed != Relax::Hidden {
            return Ok(Some("hidden"));
        }
        if !self.only_main {
            return Ok(None);
        }
        if crate::classes::is_visually_hidden(el) {
            return Ok(Some("visually hidden"));
        }
        if matches!(tag, Tag::Nav | Tag::Aside | Tag::Footer) && !self.landmark_is_content(node) {
            return Ok(Some("nav/aside/footer"));
        }
        // 署名区是正文：作者行通常是纯链接，不能当链接墙剪掉。放在导航/侧栏/页脚判断之后：
        // 侧栏里的"本站作者"列表、页脚的作者框照常剪。
        if crate::classes::is_author_list(el) {
            return Ok(None);
        }
        if tag == Tag::A {
            return Ok(is_skip_link(node, el).then_some("skip link"));
        }
        // An embedded quotation is editorial content. Only protect compact
        // wrappers consisting of a quote and its loader, never a recommendation
        // container that merely happens to contain a quote somewhere below it.
        if matches!(tag, Tag::Div | Tag::Section)
            && el
                .attr("class")
                .is_some_and(|s| s.to_ascii_lowercase().contains("embed"))
            && node.children().any(|n| n.tag() == Some(Tag::Blockquote))
            && node.children().all(|n| {
                n.is_comment()
                    || n.as_text().is_some_and(|t| t.trim().is_empty())
                    || matches!(n.tag(), Some(Tag::Blockquote | Tag::Script))
            })
        {
            return Ok(None);
        }
        // 页面标明的正文容器（itemprop="articleBody"、article-body、post-content……）本身不按
        // 文章模式的规则删：正文本身可能就是一串链接（目录、合集）
        let declared_body = self.strict
            && (self.features(node) & crate::features::ANY_TIER != 0
                || el
                    .attr("itemprop")
                    .is_some_and(|v| v.eq_ignore_ascii_case("articleBody")));
        let attachment = self.strict && self.editorial_attachment(node);
        if self.strict && !declared_body && !attachment {
            if crate::editorial::recommendation_list(node) {
                return Ok(Some("article: attributed recommendation list"));
            }
            if crate::article::is_cleaned_tag(tag) && !is_disclosure_header(node) {
                return Ok(Some("article: cleaned tag"));
            }
            if crate::article::link_dense(node, self) {
                return Ok(Some("article: link dense"));
            }
        }
        // 页面自己标出的正文区域（`<main>`、`role="main"`）不用软规则删
        if matches!(self.relaxed, Relax::All | Relax::Hidden)
            || matches!(tag, Tag::Body | Tag::Html)
            || is_main_landmark(el)
        {
            return Ok(None);
        }
        let vis = self
            .visible
            .get(node.id().index())
            .copied()
            .unwrap_or_default();
        if vis.chars >= self.protect_chars {
            return Ok(None);
        }
        let dense = vis.links >= LINK_BLOCK_MIN_LINKS
            && vis.link_chars as f64 >= vis.chars as f64 * LINK_BLOCK_MIN_SHARE;
        // 链接文字长（整句标题）或者带图的链接列表是内容卡片，首页和列表页上要留着；
        // 文章模式里它们是指向别的文章的推荐，不是这篇文章的正文
        let link_block = dense
            && (self.strict
                || ((vis.link_chars as f64 / vis.links as f64) <= LINK_BLOCK_MAX_AVG_CHARS
                    && (vis.img_links as usize) * 2 < vis.links));
        // <main> 里的链接列表多半是内容本身（目录页、相册、分类列表），<article> 里的是文章的
        // 标签、分类、目录，都不当菜单删。`role="main"` 常标在整页的外层容器上，只在放宽规则时
        // 才这样对待（文档首页的目录）
        let in_relaxed_main =
            |e: Element<'_>| self.relaxed == Relax::LinkBlocks && e.attr("role") == Some("main");
        // 链接密集的块不可能是正文本身，文章模式里不受大小上限限制
        if link_block
            && !declared_body
            && !attachment
            && (self.strict
                || !node.ancestors().any(|a| {
                    matches!(a.tag(), Some(Tag::Main | Tag::Article))
                        || a.as_element().is_some_and(in_relaxed_main)
                }))
        {
            Ok(Some("link block"))
        } else if self.relaxed == Relax::LinkBlocks {
            Ok(None)
        } else if tag == Tag::Header
            && !self.landmarks(node).in_content_area(node)
            && self.landmarks(node).has_nav_below(node)
        {
            Ok(Some("header with nav"))
        } else if el.attr("role").is_some_and(|r| SKIP_ROLES.contains(&r)) {
            Ok(Some("skip role"))
        } else if self.classes(node, el) & (NEG | POS) == NEG {
            Ok(Some("negative class/id"))
        } else if self.strict
            && !declared_body
            && !attachment
            && (self.classes(node, el) & STRICT != 0
                || crate::article::is_furniture(node, el, self.features(node)))
        {
            // 只有这条规则看块的大小：结论交给调用方按当前上限决定
            Err("article furniture")
        } else {
            Ok(None)
        }
    }

    /// `<nav>`、`<aside>`、`<footer>` 不当页面框架的两种情况：装着全页大部分文字、链接又不多的
    /// （有的网站把整页包在 `<aside>` 里）；`<article>` 里的 `<nav>`、`<footer>`（文章目录、
    /// 署名和标签）。后者照样过软跳过规则。`<article>` 里的 `<aside>` 多是"相关文章"，仍然去掉。
    fn landmark_is_content(&self, node: &NodeR<'_>) -> bool {
        let v = self
            .visible
            .get(node.id().index())
            .copied()
            .unwrap_or_default();
        if v.chars as f64 >= self.total_chars as f64 * WRAPPER_SHARE
            && (v.link_chars as f64) < v.chars as f64 * WRAPPER_MAX_LINK_SHARE
        {
            return true;
        }
        // 正文区里的 `<aside>` 不一定都是"相关文章"：食谱的配料表、教程的提示框、文章的资料框
        // 常用 `<aside>`。名字带负面词（sidebar/related/ad……）或链接密集的照旧当框架去掉。
        if node.tag() == Some(Tag::Aside)
            && self.landmarks(node).in_content_area(node)
            && v.chars >= 20
            && (v.link_chars as f64) < v.chars as f64 * 0.5
            && node
                .as_element()
                .is_none_or(|el| self.classes(node, el) & NEG == 0)
        {
            return true;
        }
        node.tag() != Some(Tag::Aside) && node.ancestors().any(|a| a.tag() == Some(Tag::Article))
    }

    /// 元素自身的可见文字数（只算未被硬跳过的部分；只取正文时才有）。
    pub(crate) fn visible_chars(&self, node: &NodeR<'_>) -> usize {
        self.visible.get(node.id().index()).map_or(0, |v| v.chars)
    }

    /// 链接统计：（有文字的链接数，其中短链接数，子树里的图片数）。
    pub(crate) fn link_stats(&self, node: &NodeR<'_>) -> (usize, usize, usize) {
        let v = self
            .visible
            .get(node.id().index())
            .copied()
            .unwrap_or_default();
        (
            v.text_links as usize,
            v.short_links as usize,
            v.images as usize,
        )
    }

    /// 可见文字数、链接文字数、链接数。诊断用。
    pub(crate) fn stats(&self, node: &NodeR<'_>) -> (usize, usize, usize) {
        let v = self
            .visible
            .get(node.id().index())
            .copied()
            .unwrap_or_default();
        (v.chars, v.link_chars, v.links)
    }

    pub(crate) fn editorial_attachment(&self, node: &NodeR<'_>) -> bool {
        let cache = &self.attachments[node.id().index()];
        if cache.get() == 0 {
            cache.set(if crate::editorial::attachment(node) {
                1
            } else {
                2
            });
        }
        cache.get() == 1
    }

    pub(crate) fn only_main(&self) -> bool {
        self.only_main
    }

    pub(crate) fn set_has_math(&mut self, v: bool) {
        self.has_math = v;
    }

    pub(crate) fn has_math(&self) -> bool {
        self.has_math
    }

    pub(crate) fn relaxed(&self) -> Relax {
        self.relaxed
    }

    pub(crate) fn is_relaxed(&self) -> bool {
        self.relaxed != Relax::None
    }
}
