use crate::patternlang::PlField;
use std::collections::{HashMap, HashSet};

pub enum FieldDiff<'a> {
    Same(&'a PlField),
    Changed(&'a PlField, &'a PlField),
    OnlyA(&'a PlField),
    OnlyB(&'a PlField),
}

pub fn field_bytes<'a>(data: &'a [u8], field: &PlField) -> Option<&'a [u8]> {
    let start = usize::try_from(field.offset).ok()?;
    let end = start.checked_add(usize::try_from(field.size).ok()?)?;
    data.get(start..end)
}

fn fields_equal(field_a: &PlField, field_b: &PlField, data_a: &[u8], data_b: &[u8]) -> bool {
    match (field_bytes(data_a, field_a), field_bytes(data_b, field_b)) {
        (Some(bytes_a), Some(bytes_b)) => bytes_a == bytes_b && field_a.offset == field_b.offset,
        _ => {
            field_a.value == field_b.value
                && field_a.type_name == field_b.type_name
                && field_a.offset == field_b.offset
                && field_a.size == field_b.size
        }
    }
}

pub fn diff_fields<'a>(
    fields_a: &'a [PlField],
    fields_b: &'a [PlField],
    data_a: &[u8],
    data_b: &[u8],
) -> Vec<FieldDiff<'a>> {
    let map_b: HashMap<&str, &PlField> = fields_b.iter().map(|f| (f.path.as_str(), f)).collect();
    let mut visited: HashSet<&str> = HashSet::new();
    let mut result = Vec::with_capacity(fields_a.len().max(fields_b.len()));

    for field_a in fields_a {
        visited.insert(field_a.path.as_str());
        match map_b.get(field_a.path.as_str()) {
            Some(field_b) => {
                if fields_equal(field_a, field_b, data_a, data_b) {
                    result.push(FieldDiff::Same(field_a));
                } else {
                    result.push(FieldDiff::Changed(field_a, field_b));
                }
            }
            None => result.push(FieldDiff::OnlyA(field_a)),
        }
    }

    for field_b in fields_b {
        if !visited.contains(field_b.path.as_str()) {
            result.push(FieldDiff::OnlyB(field_b));
        }
    }

    result
}

pub fn raw_byte_diff_ranges(data_a: &[u8], data_b: &[u8]) -> Vec<(usize, usize)> {
    let common_len = data_a.len().min(data_b.len());
    let mut ranges = Vec::new();
    let mut run_start: Option<usize> = None;

    for i in 0..common_len {
        if data_a[i] != data_b[i] {
            run_start.get_or_insert(i);
        } else if let Some(start) = run_start.take() {
            ranges.push((start, i));
        }
    }
    if let Some(start) = run_start {
        ranges.push((start, common_len));
    }

    ranges
}
