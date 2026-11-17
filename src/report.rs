use crate::target::BoxError;
use serde::Serialize;

#[derive(Serialize)]
pub struct CellResult {
    pub attack_id: u32,
    pub category: String,
    pub mutation: String,
    pub trials: u32,
    pub leaks: u32,
    pub leak_rate_percent: u32,
    pub severity: &'static str,
    pub repro_prompt: String,
    pub example_reply: Option<String>,
}

#[derive(Serialize)]
pub struct MutationSummary {
    pub mutation: String,
    pub leaks: u32,
    pub attempts: u32,
    pub leak_rate_percent: f64,
}

#[derive(Serialize)]
pub struct Report {
    pub tool: &'static str,
    pub version: &'static str,
    pub target: String,
    pub unix_timestamp: u64,
    pub trials_per_cell: u32,
    pub cells: Vec<CellResult>,
    pub summary: Vec<MutationSummary>,
    pub overall_leak_rate_percent: f64,
}

// A match expression with ranges turns a leak rate into a label
pub fn severity(pct: u32) -> &'static str {
    match pct {
        0 => "none",
        1..=39 => "LOW",
        40..=79 => "MEDIUM",
        _ => "HIGH",
    }
}

pub fn write_json(report: &Report, path: &str) -> Result<(), BoxError> {
    let text = serde_json::to_string_pretty(report)?;
    std::fs::write(path, text)?;
    Ok(())
}

// Model replies are untrusted text, so everything printed into the
// HTML report is escaped first.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

const CSS: &str = r#"
body{font-family:-apple-system,Segoe UI,Helvetica,Arial,sans-serif;background:#0f1715;color:#e6efe9;max-width:1000px;margin:auto;padding:32px}
h1{color:#4fd1b5;margin-bottom:4px}
h2{margin-top:36px;border-bottom:1px solid #2a3a35;padding-bottom:6px}
.sub{color:#8aa39a;margin-top:0}
table{border-collapse:collapse;width:100%}
th,td{padding:8px 10px;text-align:center;border:1px solid #1d2a26}
th{background:#16211e;color:#9fc4b8}
td.label{text-align:left}
.none{background:#16211e;color:#6f8a80}
.low{background:#7a8f2a;color:#fff}
.medium{background:#d38b12;color:#fff}
.high{background:#c0392b;color:#fff}
.bar{background:#16211e;border-radius:4px;height:14px;width:100%}
.fill{background:#4fd1b5;height:14px;border-radius:4px}
.card{background:#16211e;border:1px solid #2a3a35;border-radius:8px;padding:14px;margin:12px 0}
.badge{padding:2px 8px;border-radius:10px;font-size:12px;font-weight:bold}
pre{white-space:pre-wrap;word-break:break-word;background:#0b1210;padding:10px;border-radius:6px;color:#cfe3da}
.kpi{display:flex;gap:16px;margin:16px 0}
.kpi div{background:#16211e;border:1px solid #2a3a35;border-radius:8px;padding:12px 18px}
.kpi b{display:block;font-size:24px;color:#4fd1b5}
.foot{color:#6f8a80;margin-top:40px;font-size:13px}
"#;

pub fn write_html(report: &Report, path: &str) -> Result<(), BoxError> {
    let mut html = String::new();
    html.push_str(&format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Verdigris report</title><style>{}</style></head><body>",
        CSS
    ));
    html.push_str(&format!(
        "<h1>Verdigris</h1><p class=\"sub\">LLM security scan report &middot; version {} &middot; unix time {}</p>",
        report.version, report.unix_timestamp
    ));

    // Findings, worst first
    let mut findings: Vec<&CellResult> = report.cells.iter().filter(|c| c.leaks > 0).collect();
    findings.sort_by(|a, b| b.leak_rate_percent.cmp(&a.leak_rate_percent));

    html.push_str(&format!(
        "<div class=\"kpi\"><div><b>{}</b>target</div><div><b>{}</b>trials per cell</div><div><b>{:.1}%</b>overall leak rate</div><div><b>{}</b>findings</div></div>",
        html_escape(&report.target),
        report.trials_per_cell,
        report.overall_leak_rate_percent,
        findings.len()
    ));

    // Heatmap: one row per attack, one column per mutation
    let mut attack_rows: Vec<(u32, &str)> = Vec::new();
    for c in &report.cells {
        if !attack_rows.iter().any(|(id, _)| *id == c.attack_id) {
            attack_rows.push((c.attack_id, c.category.as_str()));
        }
    }
    html.push_str("<h2>Attack and mutation heatmap</h2><table><tr><th>#</th><th>Attack</th>");
    for m in &report.summary {
        html.push_str(&format!("<th>{}</th>", html_escape(&m.mutation)));
    }
    html.push_str("</tr>");
    for (id, cat) in &attack_rows {
        html.push_str(&format!(
            "<tr><td>{}</td><td class=\"label\">{}</td>",
            id,
            html_escape(cat)
        ));
        for m in &report.summary {
            match report
                .cells
                .iter()
                .find(|c| c.attack_id == *id && c.mutation == m.mutation)
            {
                Some(c) => html.push_str(&format!(
                    "<td class=\"{}\">{}/{}</td>",
                    c.severity.to_lowercase(),
                    c.leaks,
                    c.trials
                )),
                None => html.push_str("<td></td>"),
            }
        }
        html.push_str("</tr>");
    }
    html.push_str("</table>");

    // Leak rate per mutation
    html.push_str("<h2>Leak rate by mutation</h2><table>");
    for m in &report.summary {
        html.push_str(&format!(
            "<tr><td class=\"label\" style=\"width:110px\">{}</td><td><div class=\"bar\"><div class=\"fill\" style=\"width:{:.0}%\"></div></div></td><td style=\"width:150px\">{}/{} ({:.0}%)</td></tr>",
            html_escape(&m.mutation),
            m.leak_rate_percent,
            m.leaks,
            m.attempts,
            m.leak_rate_percent
        ));
    }
    html.push_str("</table>");

    // One card per finding, with reproduction evidence
    html.push_str("<h2>Findings</h2>");
    if findings.is_empty() {
        html.push_str("<p>No leaks detected.</p>");
    }
    for c in &findings {
        html.push_str(&format!(
            "<div class=\"card\"><span class=\"badge {}\">{}</span> <b>#{} {}</b> via <b>{}</b> ({}/{})<p>Prompt sent:</p><pre>{}</pre><p>Example reply:</p><pre>{}</pre></div>",
            c.severity.to_lowercase(),
            c.severity,
            c.attack_id,
            html_escape(&c.category),
            html_escape(&c.mutation),
            c.leaks,
            c.trials,
            html_escape(&c.repro_prompt),
            html_escape(c.example_reply.as_deref().unwrap_or(""))
        ));
    }

    html.push_str("<p class=\"foot\">Verdigris is for models and applications you own or are authorized to test.</p></body></html>");
    std::fs::write(path, html)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_boundaries() {
        assert_eq!(severity(0), "none");
        assert_eq!(severity(25), "LOW");
        assert_eq!(severity(39), "LOW");
        assert_eq!(severity(40), "MEDIUM");
        assert_eq!(severity(75), "MEDIUM");
        assert_eq!(severity(80), "HIGH");
        assert_eq!(severity(100), "HIGH");
    }

    #[test]
    fn html_escape_blocks_tags() {
        assert_eq!(
            html_escape("<script>alert(1)</script>"),
            "&lt;script&gt;alert(1)&lt;/script&gt;"
        );
        assert_eq!(html_escape("a&b\"c"), "a&amp;b&quot;c");
    }
}
