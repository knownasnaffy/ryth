/// Generate a single man page and a single HTML page for ryth.
///
/// Outputs to `docs/man/ryth.1` and `docs/html/ryth.html`.
use clap::CommandFactory;
use std::{fs, path::Path};

use ryth::cli::Cli;

fn main() -> anyhow::Result<()> {
    let out = Path::new("docs");
    fs::create_dir_all(out.join("man"))?;
    fs::create_dir_all(out.join("html"))?;

    let cmd = Cli::command();
    let roff = build_roff(&cmd);

    fs::write(out.join("man/ryth.1"), &roff)?;
    fs::write(out.join("html/ryth.html"), roff_to_html(&roff))?;

    println!("docs written to {}", out.display());
    Ok(())
}

/// Build a single roff man page covering all subcommands inline.
fn build_roff(cmd: &clap::Command) -> String {
    use std::fmt::Write;
    let mut cmd = cmd.clone();
    cmd.build();
    let mut r = String::new();

    // Header
    writeln!(r, ".TH \"RYTH\" \"1\"").unwrap();
    writeln!(r, ".SH NAME").unwrap();
    writeln!(r, "ryth \\- Scriptable iwd interface").unwrap();

    // Synopsis
    writeln!(r, ".SH SYNOPSIS").unwrap();
    writeln!(r, ".B ryth").unwrap();
    writeln!(r, "<command> [options]").unwrap();

    // Description
    writeln!(r, ".SH DESCRIPTION").unwrap();
    writeln!(r, "ryth is a scriptable interface to the iwd wifi backend.").unwrap();
    writeln!(r, "All output is JSON. Designed for use in scripts and automation widgets.").unwrap();
    writeln!(r, "Device and station are selected automatically.").unwrap();

    // One section per subcommand
    for sub in cmd.get_subcommands() {
        if sub.is_hide_set() || sub.get_name() == "help" { continue; }

        let name = sub.get_name();
        let about = sub.get_about().map(|s| s.to_string()).unwrap_or_default();

        writeln!(r, ".SH \"{}\"", name.to_uppercase()).unwrap();
        writeln!(r, ".B ryth {name}").unwrap();

        // positionals and flags in synopsis style
        for arg in sub.get_arguments() {
            if arg.is_hide_set() { continue; }
            match (arg.get_short(), arg.get_long()) {
                (_, Some(long)) => {
                    if arg.get_num_args().map(|n| n.takes_values()).unwrap_or(false) {
                        let val = arg.get_value_names()
                            .and_then(|v| v.first())
                            .map(|s| s.as_str())
                            .unwrap_or("VALUE");
                        write!(r, " [\\-\\-{long} <{val}>]").unwrap();
                    } else {
                        write!(r, " [\\-\\-{long}]").unwrap();
                    }
                }
                _ => {
                    if arg.is_positional() {
                        let val = arg.get_value_names()
                            .and_then(|v| v.first())
                            .map(|s| s.as_str())
                            .unwrap_or(arg.get_id().as_str());
                        let (l, r_) = if arg.is_required_set() { ("<", ">") } else { ("[", "]") };
                        write!(r, " {l}{val}{r_}").unwrap();
                    }
                }
            }
        }
        writeln!(r).unwrap();

        writeln!(r, ".PP").unwrap();
        writeln!(r, "{about}").unwrap();

        // Positional arguments
        let positionals: Vec<_> = sub.get_arguments()
            .filter(|a| !a.is_hide_set() && a.is_positional())
            .collect();
        if !positionals.is_empty() {
            writeln!(r, ".PP").unwrap();
            writeln!(r, "\\fBArguments:\\fR").unwrap();
            for arg in positionals {
                let val = arg.get_value_names()
                    .and_then(|v| v.first())
                    .map(|s| s.as_str())
                    .unwrap_or(arg.get_id().as_str());
                let desc = match val {
                    "SSID"     => "The network name (SSID) as a UTF-8 string.",
                    "STATE"    => "Either \\fBon\\fR or \\fBoff\\fR.",
                    _          => "",
                };
                writeln!(r, ".TP").unwrap();
                writeln!(r, "\\fB<{val}>\\fR").unwrap();
                writeln!(r, "{desc}").unwrap();
            }
        }

        // Options
        let args: Vec<_> = sub.get_arguments()
            .filter(|a| !a.is_hide_set() && !a.is_positional())
            .collect();
        if !args.is_empty() {
            writeln!(r, ".PP").unwrap();
            writeln!(r, "\\fBOptions:\\fR").unwrap();
            for arg in args {
                let flag = match (arg.get_short(), arg.get_long()) {
                    (Some(s), Some(l)) => format!("\\-{s}, \\-\\-{l}"),
                    (Some(s), None)    => format!("\\-{s}"),
                    (None, Some(l))    => format!("\\-\\-{l}"),
                    _ => continue,
                };
                let val_suffix = if arg.get_num_args().map(|n| n.takes_values()).unwrap_or(false) {
                    let v = arg.get_value_names()
                        .and_then(|v| v.first())
                        .map(|s| s.as_str())
                        .unwrap_or("VALUE");
                    format!(" <{v}>")
                } else {
                    String::new()
                };
                let help = arg.get_help().map(|s| s.to_string()).unwrap_or_else(|| {
                    match arg.get_long() {
                        Some("password") => "Passphrase for the network. Omit for open networks.".into(),
                        _ => String::new(),
                    }
                });
                writeln!(r, ".TP").unwrap();
                writeln!(r, "\\fB{flag}\\fR{val_suffix}").unwrap();
                writeln!(r, "{help}").unwrap();
            }
        }
    }

    // Output section
    writeln!(r, ".SH OUTPUT").unwrap();
    writeln!(r, "All commands output JSON to stdout. Errors are printed to stderr as:").unwrap();
    writeln!(r, ".PP").unwrap();
    writeln!(r, "{{\"error\": \"message\"}}").unwrap();

    r
}

/// Convert roff man page source to plain semantic HTML (no CSS, no style attributes).
fn roff_to_html(roff: &str) -> String {
    let mut body = String::new();
    let mut in_dl = false;
    let mut in_section = false;
    let mut next_is_dt = false;
    let mut in_dd = false;

    for line in roff.lines() {
        if let Some(rest) = line.strip_prefix('.') {
            let (cmd, arg) = rest.split_once(' ').unwrap_or((rest, ""));
            match cmd {
                "TH" => {}
                "SH" => {
                    close_dl(&mut body, &mut in_dl, &mut in_dd);
                    if in_section { body.push_str("</section>\n"); }
                    body.push_str(&format!("<section>\n<h2>{}</h2>\n", strip_quotes(arg)));
                    in_section = true;
                }
                "SS" => {
                    close_dl(&mut body, &mut in_dl, &mut in_dd);
                    body.push_str(&format!("<h3>{}</h3>\n", strip_quotes(arg)));
                }
                "TP" => {
                    if in_dd { body.push_str("</dd>\n"); in_dd = false; }
                    if !in_dl { body.push_str("<dl>\n"); in_dl = true; }
                    next_is_dt = true;
                }
                "PP" | "P" | "LP" => {
                    close_dl(&mut body, &mut in_dl, &mut in_dd);
                }
                "RS" => body.push_str("<blockquote>\n"),
                "RE" => body.push_str("</blockquote>\n"),
                "br" | "BR" => body.push_str("<br>\n"),
                _ => {}
            }
        } else if !line.is_empty() {
            let rendered = render_inline(line);
            if next_is_dt {
                body.push_str(&format!("<dt>{rendered}</dt>\n<dd>"));
                next_is_dt = false;
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
    if in_section { body.push_str("</section>\n"); }

    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n\
         <title>ryth</title>\n</head>\n<body>\n<h1>ryth</h1>\n{body}</body>\n</html>\n"
    )
}

fn close_dl(body: &mut String, in_dl: &mut bool, in_dd: &mut bool) {
    if *in_dd { body.push_str("</dd>\n"); *in_dd = false; }
    if *in_dl { body.push_str("</dl>\n"); *in_dl = false; }
}

fn strip_quotes(s: &str) -> &str { s.trim_matches('"') }

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
                    Some('B') => { if !bold   { out.push_str("<b>");   bold   = true; } }
                    Some('I') => { if !italic { out.push_str("<em>");  italic = true; } }
                    Some('R') | Some('P') => {
                        if bold   { out.push_str("</b>");  bold   = false; }
                        if italic { out.push_str("</em>"); italic = false; }
                    }
                    Some(x) => { out.push('\\'); out.push('f'); out.push(x); }
                    None => {}
                },
                Some(x) => { out.push('\\'); out.push(x); }
                None => {}
            }
        } else {
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
