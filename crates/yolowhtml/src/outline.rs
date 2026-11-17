//! 元素的先序区间编号和标题表：把"子树里有没有 `<h1>`""A 是不是 B 的祖先"这类问题从
//! 遍历子树变成区间比较。
//!
//! 按先序（文档顺序）给每个元素编号 `pre`，再记下它子树里最大的编号 `end`，那么 `d` 在 `a` 的
//! 子树里当且仅当 `pre[a] <= pre[d] <= end[a]`（树的 Euler 序 / 区间表示）。标题元素按先序存成表，
//! 一个子树里的标题就是表里编号落在这个区间的连续一段，用二分查找取出；顺序和
//! `descendants()` 的访问顺序一致，所以"子树里第一个满足条件的标题"结果不变。

use yolowhtml_dom::{Html, NodeId, Tag};

use crate::dom::NodeR;

const NOT_ELEMENT: u32 = u32::MAX;

#[derive(Default)]
pub(crate) struct Outline {
    pre: Vec<u32>,
    end: Vec<u32>,
    /// `<h1>`、`<h2>` 元素，按先序编号排好：(编号, 节点)。
    h1: Vec<(u32, NodeId)>,
    h2: Vec<(u32, NodeId)>,
}

impl Outline {
    /// `order` 是全部元素的先序列表（[`crate::extract`] 构造时已经算好）。沿用 `self` 的分配。
    pub(crate) fn rebuild(&mut self, html: &Html<'_>, order: &[NodeId]) {
        let n = html.len();
        self.pre.clear();
        self.pre.resize(n, NOT_ELEMENT);
        self.end.clear();
        self.end.resize(n, NOT_ELEMENT);
        self.h1.clear();
        self.h2.clear();
        for (i, id) in order.iter().enumerate() {
            let i = i as u32;
            self.pre[id.index()] = i;
            match html.get(*id).tag() {
                Some(Tag::H1) => self.h1.push((i, *id)),
                Some(Tag::H2) => self.h2.push((i, *id)),
                _ => {}
            }
        }
        // 逆先序：子元素都在父元素之前处理完，子树的最大编号自底向上取
        for id in order.iter().rev() {
            let node = html.get(*id);
            let mut end = self.pre[id.index()];
            for c in node.children() {
                if c.is_element() {
                    end = end.max(self.end[c.id().index()]);
                }
            }
            self.end[id.index()] = end;
        }
    }

    /// `node` 子树（含自身）的先序编号区间；文档根节点是全部元素。
    fn range(&self, node: &NodeR<'_>) -> (u32, u32) {
        if node.is_document() {
            return (0, u32::MAX - 1);
        }
        match self.pre.get(node.id().index()) {
            Some(&p) if p != NOT_ELEMENT => (p, self.end[node.id().index()]),
            _ => (1, 0), // 非元素：空区间
        }
    }

    fn slice(list: &[(u32, NodeId)], (lo, hi): (u32, u32)) -> &[(u32, NodeId)] {
        let a = list.partition_point(|&(p, _)| p < lo);
        let b = list.partition_point(|&(p, _)| p <= hi);
        &list[a..b.max(a)]
    }

    /// `node` 子树里（含自身）的 `<h1>`，按文档顺序。
    pub(crate) fn h1_in(&self, node: &NodeR<'_>) -> impl Iterator<Item = NodeId> + '_ {
        Self::slice(&self.h1, self.range(node))
            .iter()
            .map(|&(_, id)| id)
    }

    /// `node` 子树里（含自身）的 `<h1>` 和 `<h2>`，按文档顺序合并。
    pub(crate) fn h1_h2_in(&self, node: &NodeR<'_>) -> Vec<NodeId> {
        let r = self.range(node);
        let (a, b) = (Self::slice(&self.h1, r), Self::slice(&self.h2, r));
        let mut out = Vec::with_capacity(a.len() + b.len());
        let (mut i, mut j) = (0, 0);
        while i < a.len() || j < b.len() {
            if j == b.len() || (i < a.len() && a[i].0 < b[j].0) {
                out.push(a[i].1);
                i += 1;
            } else {
                out.push(b[j].1);
                j += 1;
            }
        }
        out
    }

    pub(crate) fn has_h1(&self, node: &NodeR<'_>) -> bool {
        self.h1_in(node).next().is_some()
    }
}
