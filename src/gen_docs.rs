/// Generate man pages and plain HTML docs for ryth.
///
/// Outputs to `docs/man/` and `docs/html/`.
use clap::CommandFactory;
use clap_mangen::Man;
use std::{
    fs,
    io::Cursor,
    path::Path,
};

use ryth::cli::Cli;

fn main() -> anyhow::Result<()> {
    let out = Path::new("docs");
    let man_dir = out.join("man");
    let html_dir = out.join("html");
    fs::create_dir_all(&man_dir)?;
    fs::create_dir_all(&html_dir)?;

    let cmd = Cli::command();
    generate(&cmd, &man_dir, &html_dir)?;

    println!("docs written to {}", out.display());
    Ok(())
}

fn generate(cmd: &clap::Command, man_dir: &Path, html_dir: &Path) -> anyhow::Result<()> {
    // top-level page
    write_page(cmd, man_dir, html_dir)?;
    // one page per subcommand
    for sub in cmd.get_subcommands() {
        if sub.is_hide_set() { continue; }
        let mut sub = sub.clone();
        // set display name so the man page title is e.g. "ryth-status"
        let display = format!("{}-{}", cmd.get_name(), sub.get_name());
        sub = sub.display_name(display);
        write_page(&sub, man_dir, html_dir)?;
    }
    Ok(())
}

fn write_page(cmd: &clap::Command, man_dir: &Path, html_dir: &Path) -> anyhow::Result<()> {
    let name = cmd.get_display_name().unwrap_or_else(|| cmd.get_name());

    // man
    let mut roff_buf = Cursor::new(Vec::new());
    Man::new(cmd.clone()).render(&mut roff_buf)?;
    let roff = String::from_utf8(roff_buf.into_inner())?;
    fs::write(man_dir.join(format!("{name}.1")), &roff)?;

    // html
    let html = roff_to_html(&roff, name);
    fs::write(html_dir.join(format!("{name}.html")), html)?;

    Ok(())
}

/// Convert roff man page source to plain semantic HTML (no CSS, no style attributes).
fn roff_to_html(roff: &str, title: &str) -> String {
    let mut body = String::new();
    let mut in_dl = false; // inside <dl> (for .TP blocks)
    let mut in_section = false;
    let mut next_is_dt = false; // next text line is a <dt>
    let mut in_dd = false;      // a <dd> has been opened and not yet closed

    for line in roff.lines() {
        if let Some(rest) = line.strip_prefix('.') {
            let (cmd, arg) = rest.split_once(' ').unwrap_or((rest, ""));
            match cmd {
                "TH" => {} // title header — we use our own <h1>
                "SH" => {
                    close_dl(&mut body, &mut in_dl, &mut in_dd);
                    if in_section {
                        body.push_str("</section>\n");
                    }
                    let heading = strip_quotes(arg);
                    body.push_str(&format!("<section>\n<h2>{heading}</h2>\n"));
                    in_section = true;
                }
                "SS" => {
                    close_dl(&mut body, &mut in_dl, &mut in_dd);
                    body.push_str(&format!("<h3>{}</h3>\n", strip_quotes(arg)));
                }
                "TP" => {
                    if in_dd { body.push_str("</dd>\n"); in_dd = false; }
                    if !in_dl {
                        body.push_str("<dl>\n");
                        in_dl = true;
                    }
                    next_is_dt = true;
                }
                "PP" | "P" | "LP" => {
                    close_dl(&mut body, &mut in_dl, &mut in_dd);
                    body.push_str("<p></p>\n");
                }
                "RS" => body.push_str("<blockquote>\n"),
                "RE" => body.push_str("</blockquote>\n"),
                "IP" => {
                    close_dl(&mut body, &mut in_dl, &mut in_dd);
                    body.push_str("<p>");
                }
                "br" | "BR" => body.push_str("<br>\n"),
                _ => {}
            }
        } else if line.is_empty() {
            // blank line
        } else {
            let rendered = render_inline(line);
            if next_is_dt {
                body.push_str(&format!("<dt>{rendered}</dt>\n"));
                next_is_dt = false;
                body.push_str("<dd>");
                in_dd = true;
            } else if in_dl {
                body.push_str(&format!("{rendered}</dd>\n"));
                in_dd = false;
            } else {
                body.push_str(&format!("<p>{rendered}</p>\n"));
            }
        }
    }

    close_dl(&mut body, &mut in_dl, &mut in_dd);
    if in_section {
        body.push_str("</section>\n");
    }

    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n\
         <title>{title}</title>\n</head>\n<body>\n<h1>{title}</h1>\n{body}</body>\n</html>\n"
    )
}

fn close_dl(body: &mut String, in_dl: &mut bool, in_dd: &mut bool) {
    if *in_dd { body.push_str("</dd>\n"); *in_dd = false; }
    if *in_dl { body.push_str("</dl>\n"); *in_dl = false; }
}

fn strip_quotes(s: &str) -> &str {
    s.trim_matches('"')
}

/// Render roff inline markup to HTML.
/// Handles \fB (bold), \fI (italic), \fR / \fP (reset), and \(bu (bullet).
fn render_inline(s: &str) -> String {
    let s = s.replace("\\(bu", "•").replace("\\-", "-");
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut bold = false;
    let mut italic = false;

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('f') => match chars.next() {
                    Some('B') => { if !bold { out.push_str("<b>"); bold = true; } }
                    Some('I') => { if !italic { out.push_str("<em>"); italic = true; } }
                    Some('R') | Some('P') => {
                        if bold   { out.push_str("</b>");  bold = false; }
                        if italic { out.push_str("</em>"); italic = false; }
                    }
                    Some(x) => { out.push('\\'); out.push('f'); out.push(x); }
                    None => {}
                },
                Some(x) => { out.push('\\'); out.push(x); }
                None => {}
            }
        } else {
            // escape HTML special chars
            match c {
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '&' => out.push_str("&amp;"),
                c   => out.push(c),
            }
        }
    }

    if bold   { out.push_str("</b>"); }
    if italic { out.push_str("</em>"); }
    out
}
