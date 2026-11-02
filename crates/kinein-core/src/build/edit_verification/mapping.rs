//! Map diagnostic points through the exact edits validated by the LSP owner.

use kinein_protocol::Diagnostic;
use std::{collections::BTreeMap, path::Path};

type Key = (Option<String>, String, Option<u64>, Option<u64>);

pub(super) struct Mapping<'a> {
    path: &'a Path,
    before: &'a str,
    after: &'a str,
    spans: Vec<(usize, usize, usize)>,
    before_lines: Vec<usize>,
    after_lines: Vec<usize>,
}

impl<'a> Mapping<'a> {
    pub(super) fn new(
        path: &'a Path,
        before: &'a str,
        after: &'a str,
        spans: &[(usize, usize, usize)],
    ) -> Result<Self, String> {
        let mut spans = spans.to_vec();
        spans.sort_unstable();
        let (mut old, mut new) = (0_usize, 0_usize);
        let mut mapped = Vec::with_capacity(spans.len());
        for &(start, end, length) in &spans {
            if start < old || end < start || end > before.len() {
                return Err("Identidade dos edits inválida.".into());
            }
            let unchanged = start - old;
            if before.as_bytes().get(old..start) != after.as_bytes().get(new..new + unchanged) {
                return Err("Conteúdo não corresponde aos edits do LSP.".into());
            }
            old = end;
            new = new
                .checked_add(unchanged)
                .and_then(|v| v.checked_add(length))
                .ok_or("Edit excede o tamanho suportado.")?;
            if new > after.len() {
                return Err("Edit excede o conteúdo editado.".into());
            }
            mapped.push((new - length, new, end));
        }
        if before.as_bytes().get(old..) != after.as_bytes().get(new..) {
            return Err("Conteúdo não corresponde aos edits do LSP.".into());
        }
        Ok(Self {
            path,
            before,
            after,
            spans: mapped,
            before_lines: std::iter::once(0)
                .chain(before.match_indices('\n').map(|(p, _)| p + 1))
                .collect(),
            after_lines: std::iter::once(0)
                .chain(after.match_indices('\n').map(|(p, _)| p + 1))
                .collect(),
        })
    }

    fn key(&self, d: &Diagnostic, edited: bool) -> Option<Key> {
        if d.file.as_deref() != self.path.to_str() {
            return Some((d.file.clone(), d.message.clone(), d.line, d.column));
        }
        let text = if edited { self.after } else { self.before };
        let line = usize::try_from(d.line?.checked_sub(1)?).ok()?;
        let column = usize::try_from(d.column?.checked_sub(1)?).ok()?;
        let starts = if edited {
            &self.after_lines
        } else {
            &self.before_lines
        };
        let start = *starts.get(line)?;
        let mut offset = start.checked_add(column)?;
        if offset > text.len() {
            return None;
        }
        if edited {
            let index = self.spans.partition_point(|(_, end, _)| *end <= offset);
            if self
                .spans
                .get(index)
                .is_some_and(|(start, _, _)| *start <= offset)
            {
                return None;
            }
            if let Some(&(_, end, old_end)) = index.checked_sub(1).and_then(|i| self.spans.get(i)) {
                offset = old_end + offset.checked_sub(end)?;
            }
        }
        Some((d.file.clone(), d.message.clone(), Some(offset as u64), None))
    }

    pub(super) fn additional(
        &self,
        before: &[Diagnostic],
        after: &[Diagnostic],
    ) -> Vec<Diagnostic> {
        let mut remaining = BTreeMap::new();
        for d in before {
            if let Some(key) = self.key(d, false) {
                *remaining.entry(key).or_insert(0_usize) += 1;
            }
        }
        after
            .iter()
            .filter(|d| {
                let Some(key) = self.key(d, true) else {
                    return true;
                };
                let count = remaining.entry(key).or_default();
                if *count > 0 {
                    *count -= 1;
                    false
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }
}
