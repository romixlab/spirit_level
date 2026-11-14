//! Strongly connected components (Tarjan) and a cheap cut of one: an order of its nodes with as few references
//! as possible pointing back up (Eades-Lin-Smyth, then adjacent swaps while they help).

use std::collections::BTreeMap;

/// The strongly connected components of a graph of `n` nodes, each a list of node ids
pub(crate) fn components(n: usize, edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n];
    for &(a, b) in edges {
        adj[a].push(b);
    }
    let mut index = vec![usize::MAX; n];
    let mut low = vec![0; n];
    let mut on = vec![false; n];
    let mut stack = Vec::new();
    let mut out = Vec::new();
    let mut next = 0;
    for start in 0..n {
        if index[start] != usize::MAX {
            continue;
        }
        // iterative DFS: (node, next child position)
        let mut work = vec![(start, 0usize)];
        index[start] = next;
        low[start] = next;
        next += 1;
        stack.push(start);
        on[start] = true;
        while let Some(&mut (v, ref mut pos)) = work.last_mut() {
            if *pos < adj[v].len() {
                let w = adj[v][*pos];
                *pos += 1;
                if index[w] == usize::MAX {
                    index[w] = next;
                    low[w] = next;
                    next += 1;
                    stack.push(w);
                    on[w] = true;
                    work.push((w, 0));
                } else if on[w] {
                    low[v] = low[v].min(index[w]);
                }
            } else {
                work.pop();
                if let Some(&(u, _)) = work.last() {
                    low[u] = low[u].min(low[v]);
                }
                if low[v] == index[v] {
                    let mut comp = Vec::new();
                    while let Some(w) = stack.pop() {
                        on[w] = false;
                        comp.push(w);
                        if w == v {
                            break;
                        }
                    }
                    comp.sort_unstable();
                    out.push(comp);
                }
            }
        }
    }
    out
}

/// The edges to cut so that `nodes` holds no cycle, found by ordering them top (referencing) to bottom
/// (referenced) and taking the references that point up; a heuristic, not a proven minimum
pub(crate) fn cut(nodes: &[usize], w: &BTreeMap<(usize, usize), u32>) -> Vec<(usize, usize)> {
    let local: BTreeMap<usize, usize> = nodes.iter().enumerate().map(|(i, v)| (*v, i)).collect();
    let sub: Vec<((usize, usize), u32)> = w
        .iter()
        .filter(|((a, b), _)| a != b && local.contains_key(a) && local.contains_key(b))
        .map(|((a, b), n)| ((local[a], local[b]), *n))
        .collect();
    let n = nodes.len();
    let (mut outs, mut ins) = (vec![Vec::new(); n], vec![Vec::new(); n]);
    let (mut out_w, mut in_w) = (vec![0i64; n], vec![0i64; n]);
    for &((a, b), k) in &sub {
        outs[a].push((b, k));
        ins[b].push((a, k));
        out_w[a] += i64::from(k);
        in_w[b] += i64::from(k);
    }
    // Eades-Lin-Smyth: sinks to the back, sources to the front, otherwise the node with most out minus in
    let mut alive = vec![true; n];
    let (mut front, mut back) = (Vec::new(), Vec::new());
    for _ in 0..n {
        let pick = (0..n)
            .filter(|&v| alive[v])
            .map(|v| {
                let class = if out_w[v] == 0 {
                    2
                } else {
                    i32::from(in_w[v] == 0)
                };
                (v, class)
            })
            .max_by_key(|&(v, class)| {
                (
                    class,
                    if class == 0 { out_w[v] - in_w[v] } else { 0 },
                    std::cmp::Reverse(v),
                )
            });
        let Some((v, class)) = pick else { break };
        alive[v] = false;
        for &(b, k) in &outs[v] {
            in_w[b] -= i64::from(k);
        }
        for &(a, k) in &ins[v] {
            out_w[a] -= i64::from(k);
        }
        if class == 2 {
            back.push(v);
        } else {
            front.push(v);
        }
    }
    back.reverse();
    let mut order: Vec<usize> = front.into_iter().chain(back).map(|v| nodes[v]).collect();
    // adjacent swaps while one lowers the weight pointing up
    let get = |a: usize, b: usize| w.get(&(a, b)).copied().unwrap_or(0);
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..order.len().saturating_sub(1) {
            let (u, v) = (order[i], order[i + 1]);
            if get(v, u) > get(u, v) {
                order.swap(i, i + 1);
                changed = true;
            }
        }
    }
    let pos: BTreeMap<usize, usize> = order.iter().enumerate().map(|(i, v)| (*v, i)).collect();
    sub.iter()
        .map(|((a, b), _)| (nodes[*a], nodes[*b]))
        .filter(|(a, b)| pos[a] > pos[b])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_cycles() {
        let c = components(5, &[(0, 1), (1, 2), (2, 0), (3, 4)]);
        let mut big: Vec<_> = c.into_iter().filter(|c| c.len() > 1).collect();
        big.sort();
        assert_eq!(big, vec![vec![0, 1, 2]]);
    }

    #[test]
    fn cuts_the_light_edge() {
        // a -> b heavily, b -> a once: cut b -> a
        let w = BTreeMap::from([((0, 1), 10), ((1, 0), 1), ((1, 2), 5), ((2, 1), 2)]);
        let mut c = cut(&[0, 1, 2], &w);
        c.sort_unstable();
        assert_eq!(c, vec![(1, 0), (2, 1)]);
    }
}
