use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn aggregate_render_prof_counters(log_path: &Path) -> BTreeMap<String, u64> {
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    let Ok(contents) = fs::read_to_string(log_path) else {
        return totals;
    };
    for line in contents.lines().filter(|line| line.contains("render.prof")) {
        let Some(counters) = field_slice(line, "counters=", " durations=") else {
            continue;
        };
        for pair in counters.split(',') {
            let Some((name, value)) = pair.rsplit_once('=') else {
                continue;
            };
            let Ok(value) = value.trim().parse::<u64>() else {
                continue;
            };
            *totals.entry(name.trim().to_string()).or_default() += value;
        }
    }
    totals
}

pub fn aggregate_render_prof_durations(log_path: &Path, name: &str) -> (u64, f64) {
    let Ok(contents) = fs::read_to_string(log_path) else {
        return (0, 0.0);
    };
    let needle = format!("{name}=count:");
    let mut total_count = 0u64;
    let mut weighted_microseconds = 0f64;
    for line in contents.lines().filter(|line| line.contains("render.prof")) {
        let mut cursor = 0usize;
        while let Some(found) = line[cursor..].find(&needle) {
            let start = cursor + found + needle.len();
            let rest = &line[start..];
            let count = leading_number(rest);
            let average = rest
                .find("avg_us:")
                .map(|offset| leading_number(&rest[offset + "avg_us:".len()..]))
                .unwrap_or(0.0);
            total_count += count as u64;
            weighted_microseconds += count * average;
            cursor = start;
        }
    }
    let average = if total_count == 0 {
        0.0
    } else {
        weighted_microseconds / total_count as f64
    };
    (total_count, average)
}

fn leading_number(text: &str) -> f64 {
    text.chars()
        .take_while(|character| character.is_ascii_digit() || *character == '.')
        .collect::<String>()
        .parse::<f64>()
        .unwrap_or(0.0)
}

fn field_slice(line: &str, start_marker: &str, end_marker: &str) -> Option<String> {
    let start = line.find(start_marker)? + start_marker.len();
    let remainder = &line[start..];
    let end = remainder.find(end_marker).unwrap_or(remainder.len());
    Some(
        remainder[..end]
            .trim()
            .trim_matches('"')
            .trim_end_matches(',')
            .to_string(),
    )
}
