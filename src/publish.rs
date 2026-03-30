use crate::journal::{CheckDetails, EntryType, JournalEntry};
use crate::storybook::{entry_type_css_class, escape_html, format_timestamp, theme};
use std::path::PathBuf;

const ENTRIES_PER_PAGE: usize = 20;
const FEED_ENTRY_LIMIT: usize = 50;

// ── Public API ────────────────────────────────────────────────────────────────

/// Generate a multi-page static site from a character's journal.
/// Returns a vec of (relative_path, content) pairs for the caller to write to disk.
pub fn generate_site(ch: &crate::character::Character, base_url: &str) -> Vec<(PathBuf, String)> {
    let stats = crate::stats::compute_stats(ch);
    let mut files: Vec<(PathBuf, String)> = Vec::new();

    // Entries in reverse-chronological order for display
    let entries: Vec<&JournalEntry> = ch.journal.iter().rev().collect();
    let total_entries = entries.len();
    let total_pages = total_pages(total_entries);

    let base_url = base_url.trim_end_matches('/');

    // style.css
    files.push((PathBuf::from("style.css"), generate_css(ch)));

    // feed.xml
    files.push((PathBuf::from("feed.xml"), generate_feed(ch, base_url)));

    // index.html (page 1) and page-N.html
    for page in 1..=total_pages.max(1) {
        let page_entries: Vec<&JournalEntry> = entries
            .iter()
            .copied()
            .skip((page - 1) * ENTRIES_PER_PAGE)
            .take(ENTRIES_PER_PAGE)
            .collect();

        let html = generate_index_page(ch, &page_entries, page, total_pages, &stats, base_url);
        let path = if page == 1 {
            PathBuf::from("index.html")
        } else {
            PathBuf::from(format!("page-{}.html", page))
        };
        files.push((path, html));
    }

    // entries/NNNN.html — chronological order (oldest = 0001)
    let chrono_entries: Vec<&JournalEntry> = ch.journal.iter().collect();
    let entry_total = chrono_entries.len();
    for (idx, entry) in chrono_entries.iter().enumerate() {
        let n = idx + 1; // 1-indexed
        let prev = if n > 1 { Some(n - 1) } else { None };
        let next = if n < entry_total { Some(n + 1) } else { None };
        let html = generate_entry_page(ch, entry, n, entry_total, prev, next, base_url);
        files.push((PathBuf::from(format!("entries/{:04}.html", n)), html));
    }

    files
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn total_pages(count: usize) -> usize {
    if count == 0 {
        1
    } else {
        count.div_ceil(ENTRIES_PER_PAGE)
    }
}

fn pass_rate(stats: &crate::stats::Stats) -> f64 {
    if stats.total_checks == 0 {
        0.0
    } else {
        (stats.successes as f64 / stats.total_checks as f64) * 100.0
    }
}

fn truncate(s: &str, max: usize) -> String {
    let escaped = escape_html(s);
    if s.chars().count() > max {
        let t: String = s.chars().take(max).collect();
        format!("{}…", escape_html(&t))
    } else {
        escaped
    }
}

fn page_href(page: usize) -> String {
    if page == 1 {
        "index.html".to_string()
    } else {
        format!("page-{}.html", page)
    }
}

fn entry_href(n: usize) -> String {
    format!("entries/{:04}.html", n)
}

// ── CSS generation ────────────────────────────────────────────────────────────

fn generate_css(ch: &crate::character::Character) -> String {
    let t = theme(ch.genre);
    // Google Fonts import for monospace genres
    let font_import = match ch.genre {
        crate::journal::Genre::SciFi | crate::journal::Genre::Cyberpunk => {
            "@import url('https://fonts.googleapis.com/css2?family=Share+Tech+Mono&display=swap');\n\n"
        }
        _ => "",
    };

    format!(
        r#"{font_import}*, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}

:root {{
  --bg: {bg};
  --bg2: {bg2};
  --text: {text};
  --accent: {accent};
  --accent2: {accent2};
  --muted: {muted};
  --border: {border};
  --tag-bg: {tag_bg};
}}

html {{ scroll-behavior: smooth; }}

body {{
  background-color: var(--bg);
  color: var(--text);
  font-family: {font_stack};
  font-size: 16px;
  line-height: 1.7;
  min-height: 100vh;
}}

/* ── Layout ── */
.container {{
  max-width: 780px;
  margin: 0 auto;
  padding: 2rem 1.5rem 4rem;
}}

/* ── Site nav ── */
.site-nav {{
  display: flex;
  gap: 1.25rem;
  align-items: center;
  padding: 0.75rem 0;
  margin-bottom: 0.5rem;
  border-bottom: 1px solid var(--border);
  font-size: 0.8rem;
  letter-spacing: 0.06em;
}}

.site-nav a {{
  color: var(--accent2);
  text-decoration: none;
  opacity: 0.75;
  transition: opacity 0.15s;
}}

.site-nav a:hover {{ opacity: 1; color: var(--accent); }}

.site-nav .feed-link {{
  margin-left: auto;
  font-size: 0.72rem;
  opacity: 0.5;
}}

/* ── Header ── */
header {{
  text-align: center;
  padding: 2.5rem 0 2rem;
  margin-bottom: 2.5rem;
}}

h1 {{
  font-family: {header_font};
  font-size: 2.2rem;
  color: var(--accent);
  font-weight: 700;
  line-height: 1.2;
}}

.subtitle {{
  margin-top: 0.6rem;
  font-size: 0.9rem;
  color: var(--accent2);
  opacity: 0.8;
  letter-spacing: 0.05em;
}}

/* ── Profile card ── */
.profile-card {{
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 1.25rem 1.5rem;
  margin-bottom: 2rem;
  display: flex;
  gap: 2rem;
  flex-wrap: wrap;
  align-items: center;
}}

.profile-stat {{
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}}

.profile-stat-value {{
  font-size: 1.4rem;
  font-weight: 700;
  color: var(--accent);
  line-height: 1;
}}

.profile-stat-label {{
  font-size: 0.68rem;
  color: var(--muted);
  letter-spacing: 0.1em;
  text-transform: uppercase;
}}

.profile-stat-sep {{
  width: 1px;
  background: var(--border);
  align-self: stretch;
}}

/* ── Entries list ── */
main {{
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}}

.entry {{
  background: var(--bg2);
  padding: 1.1rem 1.4rem;
  border-radius: 3px;
  transition: background 0.2s ease, transform 0.1s ease;
}}

.entry:hover {{ transform: translateX(2px); }}

.entry-meta {{
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.5rem;
  flex-wrap: wrap;
}}

time {{
  color: var(--muted);
  font-size: 0.8rem;
}}

.tag {{
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  padding: 0.15em 0.5em;
  border-radius: 2px;
  background: var(--tag-bg);
}}

.entry p {{
  color: var(--text);
  margin: 0;
  font-size: 0.95rem;
}}

.entry-link {{
  display: inline-block;
  margin-top: 0.5rem;
  font-size: 0.72rem;
  color: var(--accent2);
  text-decoration: none;
  opacity: 0.6;
  letter-spacing: 0.05em;
  transition: opacity 0.15s;
}}

.entry-link:hover {{ opacity: 1; }}

/* ── Entry type variants ── */
.entry-critical-success {{ border-left: 3px solid #22cc66; }}
.entry-critical-success .tag {{ color: #22cc66; }}
.entry-critical-success p {{ text-shadow: 0 0 8px rgba(34,204,102,0.15); }}

.entry-critical-failure {{ border-left: 3px solid #cc2222; }}
.entry-critical-failure .tag {{ color: #cc2222; }}
.entry-critical-failure p {{ text-shadow: 0 0 8px rgba(204,34,34,0.15); }}

.entry-check-pass {{ border-left: 3px solid #22cc66; }}
.entry-check-pass .tag {{ color: #22cc66; }}

.entry-check-fail {{ border-left: 3px solid #cc4444; }}
.entry-check-fail .tag {{ color: #cc4444; }}

.entry-game-over {{
  border: 2px solid #cc2222;
  text-align: center;
  padding: 2rem;
}}
.entry-game-over p {{
  font-size: 1.2rem;
  font-weight: 700;
  color: #cc2222;
  text-shadow: 0 0 20px rgba(204,34,34,0.4);
  letter-spacing: 0.08em;
}}
.entry-game-over .tag {{ color: #cc2222; font-size: 0.85rem; letter-spacing: 0.2em; }}

.entry-level-up {{ border-left: 3px solid #ccaa00; }}
.entry-level-up .tag {{ color: #ccaa00; }}
.entry-level-up p {{ color: #e8cc66; }}

.entry-thought {{ border-left: 3px solid var(--accent2); padding-left: 2rem; }}
.entry-thought p {{ font-style: italic; opacity: 0.9; }}
.entry-thought .tag {{ color: var(--accent2); }}

.entry-substance {{ border-left: 3px solid #aa44cc; }}
.entry-substance .tag {{ color: #cc66ee; }}
.entry-substance p {{ color: #d0a0e8; }}

.entry-rest {{ border-left: 3px solid var(--muted); opacity: 0.75; }}
.entry-rest .tag {{ color: var(--muted); }}
.entry-rest p {{ font-size: 0.9rem; }}

.entry-manual .tag {{ color: var(--text); opacity: 0.6; }}

/* ── Pagination ── */
.pagination {{
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  margin-top: 2.5rem;
  padding-top: 1.5rem;
  border-top: 1px solid var(--border);
  font-size: 0.82rem;
  color: var(--muted);
}}

.pagination a {{
  color: var(--accent2);
  text-decoration: none;
  padding: 0.3em 0.7em;
  border: 1px solid var(--border);
  border-radius: 3px;
  transition: border-color 0.15s, color 0.15s;
}}

.pagination a:hover {{
  color: var(--accent);
  border-color: var(--accent2);
}}

.pagination .current-page {{
  color: var(--text);
  opacity: 0.5;
}}

/* ── Entry page ── */
.breadcrumb {{
  font-size: 0.8rem;
  margin-bottom: 2rem;
  opacity: 0.65;
}}

.breadcrumb a {{
  color: var(--accent2);
  text-decoration: none;
}}

.breadcrumb a:hover {{ color: var(--accent); opacity: 1; }}

.entry-full {{
  background: var(--bg2);
  padding: 1.75rem 2rem;
  border-radius: 4px;
}}

.entry-full p {{
  margin-top: 0.75rem;
  font-size: 1rem;
  line-height: 1.85;
  color: var(--text);
}}

/* ── Check details block ── */
.check-details {{
  margin-top: 1.25rem;
  padding: 0.9rem 1.1rem;
  background: var(--tag-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  font-family: 'Courier New', 'Consolas', monospace;
  font-size: 0.82rem;
  line-height: 1.7;
}}

.check-details-title {{
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--muted);
  margin-bottom: 0.5rem;
}}

.check-details-row {{
  display: flex;
  gap: 0.5rem;
  align-items: baseline;
}}

.check-details-label {{
  color: var(--muted);
  min-width: 7rem;
  font-size: 0.75rem;
}}

.check-details-value {{ color: var(--text); }}

.check-result-pass {{ color: #22cc66; font-weight: 700; }}
.check-result-fail {{ color: #cc2222; font-weight: 700; }}
.check-color-white {{ color: #cccccc; }}
.check-color-red {{ color: #cc2222; }}

/* ── Entry nav ── */
.entry-nav {{
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 2.5rem;
  padding-top: 1.25rem;
  border-top: 1px solid var(--border);
  font-size: 0.8rem;
  gap: 1rem;
}}

.entry-nav a {{
  color: var(--accent2);
  text-decoration: none;
  padding: 0.3em 0.7em;
  border: 1px solid var(--border);
  border-radius: 3px;
  transition: border-color 0.15s, color 0.15s;
}}

.entry-nav a:hover {{
  color: var(--accent);
  border-color: var(--accent2);
}}

.entry-nav .entry-nav-center {{
  color: var(--muted);
  font-size: 0.75rem;
  text-align: center;
  flex: 1;
}}

.entry-nav .entry-nav-placeholder {{
  width: 4.5rem;
}}

/* ── Footer ── */
footer {{
  text-align: center;
  margin-top: 4rem;
  padding-top: 1.5rem;
  font-size: 0.72rem;
  color: var(--muted);
  opacity: 0.55;
  letter-spacing: 0.08em;
}}

footer a {{
  color: var(--muted);
  text-decoration: none;
  opacity: 0.8;
}}

footer a:hover {{ opacity: 1; }}

/* ── Scroll to top ── */
.scroll-top {{
  position: fixed;
  bottom: 2rem;
  right: 2rem;
  background: var(--tag-bg);
  border: 1px solid var(--border);
  color: var(--accent);
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 1rem;
  text-decoration: none;
  opacity: 0.6;
  transition: opacity 0.2s;
}}

.scroll-top:hover {{ opacity: 1; }}

/* ── Responsive ── */
@media (max-width: 600px) {{
  .container {{ padding: 1rem 1rem 3rem; }}
  header {{ padding: 1.75rem 0 1.25rem; }}
  h1 {{ font-size: 1.6rem; }}
  .entry {{ padding: 0.9rem 1rem; }}
  .profile-card {{ gap: 1rem; }}
  .entry-full {{ padding: 1.25rem 1rem; }}
  .entry-nav {{ flex-wrap: wrap; justify-content: center; }}
}}

/* ── Genre-specific overrides ── */
{extra_css}
"#,
        font_import = font_import,
        bg = t.bg,
        bg2 = t.bg2,
        text = t.text,
        accent = t.accent,
        accent2 = t.accent2,
        muted = t.muted,
        border = t.border,
        tag_bg = t.tag_bg,
        font_stack = t.font_stack,
        header_font = t.header_font,
        extra_css = t.extra_css,
    )
}

// ── Index / pagination pages ──────────────────────────────────────────────────

fn generate_index_page(
    ch: &crate::character::Character,
    entries: &[&JournalEntry],
    page: usize,
    total_pages: usize,
    stats: &crate::stats::Stats,
    base_url: &str,
) -> String {
    let name = escape_html(&ch.name);
    let archetype = escape_html(&ch.archetype);
    let genre_label = ch.genre.to_string();
    let total_entries = ch.journal.len();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let title = if page == 1 {
        format!("{}'s Journal", name)
    } else {
        format!("{}'s Journal — Page {}", name, page)
    };

    // Profile card stats
    let pr = pass_rate(stats);
    let profile_card = format!(
        r#"    <div class="profile-card">
      <div class="profile-stat">
        <span class="profile-stat-value">{level}</span>
        <span class="profile-stat-label">Level</span>
      </div>
      <div class="profile-stat-sep"></div>
      <div class="profile-stat">
        <span class="profile-stat-value">{total}</span>
        <span class="profile-stat-label">{word}</span>
      </div>
      <div class="profile-stat-sep"></div>
      <div class="profile-stat">
        <span class="profile-stat-value">{checks}</span>
        <span class="profile-stat-label">Checks</span>
      </div>
      <div class="profile-stat-sep"></div>
      <div class="profile-stat">
        <span class="profile-stat-value">{pass_rate:.0}%</span>
        <span class="profile-stat-label">Pass Rate</span>
      </div>
      <div class="profile-stat-sep"></div>
      <div class="profile-stat">
        <span class="profile-stat-value">{crits}</span>
        <span class="profile-stat-label">Critical Wins</span>
      </div>
    </div>
"#,
        level = ch.level,
        total = total_entries,
        word = if total_entries == 1 {
            "Entry"
        } else {
            "Entries"
        },
        checks = stats.total_checks,
        pass_rate = pr,
        crits = stats.critical_successes,
    );

    // Entry list
    // We need the global chronological index for entry links.
    // entries here are in reverse-chrono, and we know which page we're on.
    // The chronological index of the i-th entry in rev-order is (total_entries - global_rev_idx).
    let global_offset = (page - 1) * ENTRIES_PER_PAGE; // offset in the rev-chrono slice
    let mut entries_html = String::new();
    for (local_i, entry) in entries.iter().enumerate() {
        let rev_idx = global_offset + local_i; // index in full rev-chrono list
        let chrono_n = total_entries - rev_idx; // 1-indexed chrono number
        let cls = entry_type_css_class(entry.entry_type);
        let tag_label = entry_type_display(entry.entry_type);
        let ts = format_timestamp(&entry.timestamp);
        let content = truncate(&entry.content, 200);

        entries_html.push_str(&format!(
            r#"    <article class="entry {cls}" data-index="{n:04}">
      <div class="entry-meta">
        <time datetime="{ts_raw}">{ts}</time>
        <span class="tag tag-{cls}">{tag}</span>
      </div>
      <p>{content}</p>
      <a class="entry-link" href="{href}">Read entry #{n} &rarr;</a>
    </article>
"#,
            cls = cls,
            n = chrono_n,
            ts_raw = entry.timestamp.to_rfc3339(),
            ts = ts,
            tag = tag_label,
            content = content,
            href = entry_href(chrono_n),
        ));
    }

    // Pagination
    let pagination = generate_pagination(page, total_pages);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <meta name="description" content="{archetype} · Level {level} · {genre} · {total_entries} entries · {pass_rate_int}% pass rate">
  <meta property="og:title" content="{og_title}">
  <meta property="og:description" content="{archetype} · Level {level} · {genre} · {total_entries} entries · {pass_rate_int}% pass rate">
  <meta property="og:type" content="website">
  <meta property="og:url" content="{base_url}/{page_href}">
  <meta name="twitter:card" content="summary">
  <meta name="twitter:title" content="{og_title}">
  <meta name="twitter:description" content="{archetype} · Level {level} · {genre} · {total_entries} entries">
  <link rel="stylesheet" href="style.css">
  <link rel="alternate" type="application/atom+xml" title="{name}'s Journal" href="feed.xml">
</head>
<body>
  <div class="container">
    <nav class="site-nav">
      <a href="index.html">Journal</a>
      <a class="feed-link" href="feed.xml">&#128229; Atom Feed</a>
    </nav>
    <header>
      <h1>{name}</h1>
      <p class="subtitle">{archetype} &middot; Level {level} &middot; {genre}</p>
    </header>
{profile_card}
    <main>
{entries}    </main>
{pagination}
    <footer>
      <p>Generated by <a href="https://github.com/inland-empire">Inland Empire</a> &middot; {today}</p>
    </footer>
  </div>
  <a class="scroll-top" href="" onclick="window.scrollTo(0,0);return false" title="Back to top">&#8593;</a>
</body>
</html>
"#,
        title = title,
        og_title = title,
        name = name,
        archetype = archetype,
        level = ch.level,
        genre = genre_label,
        total_entries = total_entries,
        pass_rate_int = pr as u64,
        base_url = base_url,
        page_href = page_href(page),
        profile_card = profile_card,
        entries = entries_html,
        pagination = pagination,
        today = today,
    )
}

fn generate_pagination(page: usize, total_pages: usize) -> String {
    if total_pages <= 1 {
        return String::new();
    }

    let prev = if page > 1 {
        format!(r#"<a href="{}">&laquo; Prev</a>"#, page_href(page - 1))
    } else {
        r#"<span class="entry-nav-placeholder"></span>"#.to_string()
    };

    let next = if page < total_pages {
        format!(r#"<a href="{}">Next &raquo;</a>"#, page_href(page + 1))
    } else {
        r#"<span class="entry-nav-placeholder"></span>"#.to_string()
    };

    format!(
        r#"    <nav class="pagination" aria-label="Page navigation">
      {prev}
      <span class="current-page">Page {page} of {total}</span>
      {next}
    </nav>
"#,
        prev = prev,
        page = page,
        total = total_pages,
        next = next,
    )
}

// ── Individual entry page ─────────────────────────────────────────────────────

fn generate_entry_page(
    ch: &crate::character::Character,
    entry: &JournalEntry,
    n: usize,
    total: usize,
    prev: Option<usize>,
    next: Option<usize>,
    base_url: &str,
) -> String {
    let name = escape_html(&ch.name);
    let cls = entry_type_css_class(entry.entry_type);
    let tag_label = entry_type_display(entry.entry_type);
    let ts = format_timestamp(&entry.timestamp);
    let content = escape_html(&entry.content);
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let check_details_html = match &entry.details {
        Some(d) => render_check_details(d),
        None => String::new(),
    };

    let prev_link = match prev {
        Some(p) => format!(
            r#"<a href="{}" title="Previous entry">&larr; Previous</a>"#,
            entry_href(p)
        ),
        None => r#"<span class="entry-nav-placeholder"></span>"#.to_string(),
    };

    let next_link = match next {
        Some(nx) => format!(
            r#"<a href="{}" title="Next entry">Next &rarr;</a>"#,
            entry_href(nx)
        ),
        None => r#"<span class="entry-nav-placeholder"></span>"#.to_string(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Entry #{n} &mdash; {name}</title>
  <meta name="description" content="{desc}">
  <meta property="og:title" content="Entry #{n} — {name}">
  <meta property="og:description" content="{desc}">
  <meta property="og:type" content="article">
  <meta property="og:url" content="{base_url}/entries/{n:04}.html">
  <meta property="article:published_time" content="{ts_raw}">
  <meta name="twitter:card" content="summary">
  <meta name="twitter:title" content="Entry #{n} — {name}">
  <meta name="twitter:description" content="{desc}">
  <link rel="stylesheet" href="../style.css">
  <link rel="alternate" type="application/atom+xml" title="{name}'s Journal" href="../feed.xml">
</head>
<body>
  <div class="container">
    <nav class="site-nav">
      <a href="../index.html">&#8592; Journal</a>
      <a class="feed-link" href="../feed.xml">&#128229; Atom Feed</a>
    </nav>
    <div class="breadcrumb">
      <a href="../index.html">{name}</a> / Entry #{n}
    </div>
    <main>
      <article class="entry entry-full {cls}" data-index="{n:04}">
        <div class="entry-meta">
          <time datetime="{ts_raw}">{ts}</time>
          <span class="tag tag-{cls}">{tag}</span>
        </div>
        <p>{content}</p>
{check_details}      </article>
    </main>
    <nav class="entry-nav" aria-label="Entry navigation">
      {prev_link}
      <span class="entry-nav-center">Entry {n} of {total}</span>
      {next_link}
    </nav>
    <footer>
      <p>Generated by <a href="https://github.com/inland-empire">Inland Empire</a> &middot; {today}</p>
    </footer>
  </div>
</body>
</html>
"#,
        n = n,
        name = name,
        desc = truncate(&entry.content, 160),
        base_url = base_url,
        cls = cls,
        ts_raw = entry.timestamp.to_rfc3339(),
        ts = ts,
        tag = tag_label,
        content = content,
        check_details = check_details_html,
        prev_link = prev_link,
        next_link = next_link,
        total = total,
        today = today,
    )
}

fn render_check_details(d: &CheckDetails) -> String {
    let result_class = if d.success {
        "check-result-pass"
    } else {
        "check-result-fail"
    };
    let result_label = if d.success { "PASS" } else { "FAIL" };
    let color_class = if d.check_color.to_lowercase() == "red" {
        "check-color-red"
    } else {
        "check-color-white"
    };
    let modifier_str = if d.modifier >= 0 {
        format!("+{}", d.modifier)
    } else {
        d.modifier.to_string()
    };
    let skill = escape_html(&d.skill);
    let tool = escape_html(&d.tool);
    let context = escape_html(&d.context);

    format!(
        r#"        <div class="check-details">
          <div class="check-details-title">Check Details</div>
          <div class="check-details-row">
            <span class="check-details-label">Skill</span>
            <span class="check-details-value">{skill}</span>
          </div>
          <div class="check-details-row">
            <span class="check-details-label">Roll</span>
            <span class="check-details-value">{d1} + {d2} {modifier} = {total} vs DC {dc}</span>
          </div>
          <div class="check-details-row">
            <span class="check-details-label">Result</span>
            <span class="check-details-value {result_class}">{result_label}</span>
          </div>
          <div class="check-details-row">
            <span class="check-details-label">Check Color</span>
            <span class="check-details-value {color_class}">{color}</span>
          </div>
          <div class="check-details-row">
            <span class="check-details-label">Tool</span>
            <span class="check-details-value">{tool}</span>
          </div>
          <div class="check-details-row">
            <span class="check-details-label">Context</span>
            <span class="check-details-value">{context}</span>
          </div>
        </div>
"#,
        skill = skill,
        d1 = d.roll.0,
        d2 = d.roll.1,
        modifier = modifier_str,
        total = d.total,
        dc = d.threshold,
        result_class = result_class,
        result_label = result_label,
        color_class = color_class,
        color = escape_html(&d.check_color),
        tool = tool,
        context = context,
    )
}

// ── Atom feed ─────────────────────────────────────────────────────────────────

fn generate_feed(ch: &crate::character::Character, base_url: &str) -> String {
    let name = escape_html(&ch.name);
    let archetype = escape_html(&ch.archetype);
    let genre_label = escape_html(&ch.genre.to_string());
    let base_url = base_url.trim_end_matches('/');

    // Most recent entries first, up to FEED_ENTRY_LIMIT
    let entries: Vec<(usize, &JournalEntry)> = ch
        .journal
        .iter()
        .enumerate()
        .rev()
        .take(FEED_ENTRY_LIMIT)
        .collect();

    let updated = entries
        .first()
        .map(|(_, e)| e.timestamp.to_rfc3339())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let mut feed_entries = String::new();
    for (chrono_idx, entry) in &entries {
        let n = chrono_idx + 1; // 1-indexed
        let tag_label = escape_html(&entry_type_display(entry.entry_type));
        let ts = entry.timestamp.to_rfc3339();
        let content_escaped = escape_html(&entry.content);
        let entry_url = format!("{}/entries/{:04}.html", base_url, n);
        let entry_id = format!(
            "tag:{},{}/entries/{:04}",
            base_url
                .trim_start_matches("https://")
                .trim_start_matches("http://"),
            entry.timestamp.format("%Y-%m-%d"),
            n,
        );

        // Optional check details as part of HTML content
        let details_html = match &entry.details {
            Some(d) => {
                let result = if d.success { "PASS" } else { "FAIL" };
                let modifier_str = if d.modifier >= 0 {
                    format!("+{}", d.modifier)
                } else {
                    d.modifier.to_string()
                };
                escape_html(&format!(
                    "\n[{skill} | {d1}+{d2}{mod}={total} vs DC{dc} | {result} | {color}]",
                    skill = d.skill,
                    d1 = d.roll.0,
                    d2 = d.roll.1,
                    mod = modifier_str,
                    total = d.total,
                    dc = d.threshold,
                    result = result,
                    color = d.check_color,
                ))
            }
            None => String::new(),
        };

        feed_entries.push_str(&format!(
            r#"  <entry>
    <title>{tag}: {name}</title>
    <id>{id}</id>
    <published>{ts}</published>
    <updated>{ts}</updated>
    <link href="{url}"/>
    <content type="html">{content}{details}</content>
  </entry>
"#,
            tag = tag_label,
            name = name,
            id = entry_id,
            ts = ts,
            url = entry_url,
            content = content_escaped,
            details = details_html,
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>{name}&apos;s Journal</title>
  <subtitle>{archetype} &middot; Level {level} &middot; {genre}</subtitle>
  <id>{base_url}/</id>
  <link rel="self" href="{base_url}/feed.xml"/>
  <link rel="alternate" href="{base_url}/"/>
  <updated>{updated}</updated>
  <author>
    <name>{name}</name>
  </author>
{entries}</feed>
"#,
        name = name,
        archetype = archetype,
        level = ch.level,
        genre = genre_label,
        base_url = base_url,
        updated = updated,
        entries = feed_entries,
    )
}

// ── Timeline aggregator ───────────────────────────────────────────────────────

/// Generate a standalone timeline aggregator (index.html + feeds.json).
/// Returns a vec of (relative_path, content) pairs for the caller to write to disk.
pub fn generate_timeline(own_feed_url: &str) -> Vec<(std::path::PathBuf, String)> {
    let own_feed_url = own_feed_url.trim_end_matches('/');

    let feeds_json = if own_feed_url.is_empty() {
        "[]".to_string()
    } else {
        format!(
            r#"[
  {{ "url": "{own_feed_url}/feed.xml", "name": "My Journal" }}
]
"#,
            own_feed_url = own_feed_url
        )
    };

    let index_html = generate_timeline_html();

    vec![
        (std::path::PathBuf::from("index.html"), index_html),
        (std::path::PathBuf::from("feeds.json"), feeds_json),
    ]
}

fn generate_timeline_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>INLAND EMPIRE — Timeline</title>
  <meta name="description" content="A merged chronological feed of Inland Empire player journals">
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    :root {
      --bg: #12121f;
      --bg2: #1a1a2e;
      --bg3: #0e0e1a;
      --text: #c8b898;
      --accent: #f0c070;
      --accent2: #a08858;
      --muted: #6a5a40;
      --border: #2a2a3f;
      --tag-bg: #1e1e30;
      --scanline: rgba(255,255,255,0.02);
    }

    html { scroll-behavior: smooth; }

    body {
      background-color: var(--bg);
      color: var(--text);
      font-family: 'Georgia', 'Times New Roman', serif;
      font-size: 16px;
      line-height: 1.7;
      min-height: 100vh;
    }

    /* ── Scanline effect ── */
    body::before {
      content: '';
      position: fixed;
      top: 0; left: 0; right: 0; bottom: 0;
      background: repeating-linear-gradient(
        0deg,
        transparent,
        transparent 2px,
        var(--scanline) 2px,
        var(--scanline) 4px
      );
      pointer-events: none;
      z-index: 1000;
      opacity: 0.4;
    }

    .container {
      max-width: 820px;
      margin: 0 auto;
      padding: 2rem 1.5rem 5rem;
    }

    /* ── Header ── */
    header {
      text-align: center;
      padding: 2.5rem 0 2rem;
      margin-bottom: 2rem;
      position: relative;
      overflow: hidden;
    }

    header::after {
      content: '';
      position: absolute;
      bottom: 0; left: 10%; right: 10%;
      height: 1px;
      background: linear-gradient(90deg, transparent, var(--accent2), transparent);
    }

    h1 {
      font-family: 'Georgia', serif;
      font-size: 2rem;
      color: var(--accent);
      font-weight: 700;
      letter-spacing: 0.1em;
      text-transform: uppercase;
      line-height: 1.2;
      text-shadow: 0 0 30px rgba(240,192,112,0.3);
    }

    .subtitle {
      margin-top: 0.6rem;
      font-size: 0.82rem;
      color: var(--accent2);
      opacity: 0.8;
      letter-spacing: 0.15em;
      text-transform: uppercase;
    }

    /* ── Feed management ── */
    .feeds-section {
      background: var(--bg2);
      border: 1px solid var(--border);
      border-radius: 4px;
      padding: 1.25rem 1.5rem;
      margin-bottom: 2rem;
    }

    .feeds-section h2 {
      font-size: 0.72rem;
      font-weight: 700;
      letter-spacing: 0.14em;
      text-transform: uppercase;
      color: var(--muted);
      margin-bottom: 1rem;
    }

    .feeds-list {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      margin-bottom: 1rem;
    }

    .feed-item {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 0.4rem 0.6rem;
      background: var(--tag-bg);
      border: 1px solid var(--border);
      border-radius: 3px;
      font-size: 0.85rem;
    }

    .feed-item-name {
      color: var(--accent2);
      font-weight: 600;
      min-width: 8rem;
    }

    .feed-item-url {
      color: var(--muted);
      font-size: 0.75rem;
      flex: 1;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .feed-item-status {
      font-size: 0.68rem;
      letter-spacing: 0.06em;
    }

    .feed-status-ok { color: #22cc66; }
    .feed-status-err { color: #cc4444; }
    .feed-status-loading { color: var(--muted); }

    .btn-remove {
      background: none;
      border: 1px solid var(--border);
      color: var(--muted);
      font-size: 0.7rem;
      padding: 0.15em 0.5em;
      border-radius: 2px;
      cursor: pointer;
      transition: border-color 0.15s, color 0.15s;
      flex-shrink: 0;
    }

    .btn-remove:hover { border-color: #cc4444; color: #cc4444; }

    .add-feed-row {
      display: flex;
      gap: 0.5rem;
    }

    .add-feed-input {
      flex: 1;
      background: var(--bg3);
      border: 1px solid var(--border);
      color: var(--text);
      padding: 0.45rem 0.75rem;
      border-radius: 3px;
      font-size: 0.85rem;
      font-family: inherit;
      transition: border-color 0.15s;
    }

    .add-feed-input:focus {
      outline: none;
      border-color: var(--accent2);
    }

    .add-feed-input::placeholder { color: var(--muted); }

    .btn-add {
      background: var(--tag-bg);
      border: 1px solid var(--accent2);
      color: var(--accent);
      font-size: 0.82rem;
      padding: 0.45rem 1rem;
      border-radius: 3px;
      cursor: pointer;
      transition: background 0.15s, color 0.15s;
      white-space: nowrap;
    }

    .btn-add:hover { background: var(--accent2); color: var(--bg); }

    /* ── Status bar ── */
    .status-bar {
      display: flex;
      align-items: center;
      gap: 1rem;
      margin-bottom: 1.5rem;
      font-size: 0.78rem;
      color: var(--muted);
      letter-spacing: 0.04em;
    }

    .status-count { color: var(--accent2); }

    .btn-refresh {
      margin-left: auto;
      background: none;
      border: 1px solid var(--border);
      color: var(--muted);
      font-size: 0.72rem;
      padding: 0.25em 0.7em;
      border-radius: 2px;
      cursor: pointer;
      transition: border-color 0.15s, color 0.15s;
    }

    .btn-refresh:hover { border-color: var(--accent2); color: var(--accent2); }

    /* ── Timeline entries ── */
    #timeline {
      display: flex;
      flex-direction: column;
      gap: 1.1rem;
    }

    @keyframes fadeInUp {
      from { opacity: 0; transform: translateY(8px); }
      to { opacity: 1; transform: translateY(0); }
    }

    .entry {
      background: var(--bg2);
      padding: 1.1rem 1.4rem;
      border-radius: 4px;
      border-left: 3px solid var(--border);
      border: 1px solid var(--border);
      border-left: 3px solid var(--border);
      animation: fadeInUp 0.2s ease forwards;
      transition: background 0.2s ease, transform 0.1s ease;
    }

    .entry:hover { transform: translateX(2px); background: #1d1d32; }

    .entry-meta {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      margin-bottom: 0.5rem;
      flex-wrap: wrap;
    }

    .entry-author {
      font-size: 0.8rem;
      font-weight: 600;
      color: var(--accent2);
      letter-spacing: 0.04em;
    }

    .entry-time {
      color: var(--muted);
      font-size: 0.78rem;
    }

    .tag {
      font-size: 0.65rem;
      font-weight: 700;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      padding: 0.15em 0.5em;
      border-radius: 2px;
      background: var(--tag-bg);
    }

    .entry-content {
      color: var(--text);
      font-size: 0.95rem;
      margin: 0;
    }

    .entry-link {
      display: inline-block;
      margin-top: 0.5rem;
      font-size: 0.72rem;
      color: var(--accent2);
      text-decoration: none;
      opacity: 0.6;
      letter-spacing: 0.05em;
      transition: opacity 0.15s;
    }

    .entry-link:hover { opacity: 1; }

    /* ── Entry type colors ── */
    .tag-critical-success { color: #22cc66; }
    .entry-critical-success { border-left-color: #22cc66; }

    .tag-critical-failure { color: #cc2222; }
    .entry-critical-failure { border-left-color: #cc2222; }

    .tag-check-pass { color: #22cc66; }
    .entry-check-pass { border-left-color: #22cc66; }

    .tag-check-fail { color: #cc4444; }
    .entry-check-fail { border-left-color: #cc4444; }

    .tag-game-over { color: #cc2222; }
    .entry-game-over { border-left-color: #cc2222; border-color: #cc2222; }

    .tag-level-up { color: #ccaa00; }
    .entry-level-up { border-left-color: #ccaa00; }

    .tag-thought { color: var(--accent2); }
    .entry-thought { border-left-color: var(--accent2); }

    .tag-substance { color: #cc66ee; }
    .entry-substance { border-left-color: #aa44cc; }

    .tag-rest { color: var(--muted); }
    .entry-rest { border-left-color: var(--muted); opacity: 0.75; }

    .tag-manual { color: var(--text); opacity: 0.6; }

    /* ── Loading / empty states ── */
    .state-loading {
      text-align: center;
      padding: 3rem 1rem;
      color: var(--muted);
      font-size: 0.9rem;
      letter-spacing: 0.1em;
    }

    .state-loading::before {
      content: '';
      display: block;
      width: 2rem;
      height: 2rem;
      border: 2px solid var(--border);
      border-top-color: var(--accent2);
      border-radius: 50%;
      margin: 0 auto 1rem;
      animation: spin 1s linear infinite;
    }

    @keyframes spin { to { transform: rotate(360deg); } }

    .state-empty {
      text-align: center;
      padding: 3rem 1rem;
      color: var(--muted);
      font-size: 0.9rem;
    }

    .feed-error {
      padding: 0.6rem 1rem;
      background: rgba(204,34,34,0.08);
      border: 1px solid rgba(204,34,34,0.3);
      border-radius: 3px;
      font-size: 0.78rem;
      color: #cc6666;
      margin-bottom: 0.75rem;
    }

    .feed-error strong { color: #cc4444; }

    .load-more {
      text-align: center;
      padding: 1.5rem 0;
    }

    .btn-load-more {
      background: var(--tag-bg);
      border: 1px solid var(--border);
      color: var(--accent2);
      font-size: 0.82rem;
      padding: 0.55rem 1.5rem;
      border-radius: 3px;
      cursor: pointer;
      transition: border-color 0.15s, color 0.15s;
    }

    .btn-load-more:hover { border-color: var(--accent2); color: var(--accent); }

    /* ── Footer ── */
    footer {
      text-align: center;
      margin-top: 4rem;
      padding-top: 1.5rem;
      font-size: 0.72rem;
      color: var(--muted);
      opacity: 0.55;
      letter-spacing: 0.08em;
      border-top: 1px solid var(--border);
    }

    footer a { color: var(--muted); text-decoration: none; }
    footer a:hover { opacity: 1; color: var(--accent2); }

    /* ── Responsive ── */
    @media (max-width: 600px) {
      .container { padding: 1rem 1rem 3rem; }
      h1 { font-size: 1.4rem; }
      .add-feed-row { flex-direction: column; }
      .entry { padding: 0.9rem 1rem; }
      .feed-item-url { display: none; }
    }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>Inland Empire</h1>
      <p class="subtitle">Timeline &mdash; Merged Field Reports</p>
    </header>

    <section class="feeds-section" aria-label="Feed management">
      <h2>Connected Feeds</h2>
      <div id="feeds-list" class="feeds-list"></div>
      <div class="add-feed-row">
        <input
          type="url"
          id="add-feed-input"
          class="add-feed-input"
          placeholder="https://player.example.com"
          aria-label="Feed URL to add"
        >
        <button class="btn-add" id="btn-add-feed">+ Add Feed</button>
      </div>
    </section>

    <div class="status-bar">
      <span id="status-text">Loading...</span>
      <button class="btn-refresh" id="btn-refresh" title="Refresh all feeds">&#8635; Refresh</button>
    </div>

    <div id="error-area"></div>

    <div id="timeline">
      <div class="state-loading">Tuning in...</div>
    </div>

    <div id="load-more-area"></div>

    <footer>
      <p>Powered by <a href="https://github.com/inland-empire">Inland Empire</a></p>
    </footer>
  </div>

  <script>
  (function () {
    'use strict';

    const STORAGE_KEY = 'ie-timeline-feeds';
    const PAGE_SIZE = 30;
    const REFRESH_MS = 5 * 60 * 1000;

    // ── State ──────────────────────────────────────────────────────────────────
    let feeds = [];        // {{ url, name }}
    let allEntries = [];   // parsed, merged, sorted
    let rendered = 0;      // how many entries currently shown
    let feedStatuses = {}; // url → 'loading' | 'ok' | 'error'
    let cachedEntries = {}; // url → entry array (for offline-first)
    let refreshTimer = null;

    // ── Boot ──────────────────────────────────────────────────────────────────
    async function boot() {
      loadCachedEntries();
      await loadFeeds();
      renderFeedsList();
      renderCachedTimeline();
      await fetchAllFeeds();
      scheduledRefresh();
    }

    function scheduledRefresh() {
      clearTimeout(refreshTimer);
      refreshTimer = setTimeout(async function () {
        await fetchAllFeeds();
        scheduledRefresh();
      }, REFRESH_MS);
    }

    // ── Feed config ───────────────────────────────────────────────────────────
    async function loadFeeds() {
      let defaultFeeds = [];
      try {
        const r = await fetch('feeds.json', { cache: 'no-cache' });
        if (r.ok) defaultFeeds = await r.json();
      } catch (_) {}

      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        try {
          // localStorage takes priority; merge so defaults appear if not already present
          const storedFeeds = JSON.parse(stored);
          const storedUrls = new Set(storedFeeds.map(function (f) { return f.url; }));
          const merged = storedFeeds.slice();
          defaultFeeds.forEach(function (f) {
            if (!storedUrls.has(f.url)) merged.push(f);
          });
          feeds = merged;
        } catch (_) {
          feeds = defaultFeeds;
        }
      } else {
        feeds = defaultFeeds;
      }
      saveFeeds();
    }

    function saveFeeds() {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(feeds));
    }

    function addFeed(url, name) {
      url = url.trim().replace(/\/$/, '');
      if (!url) return;
      if (feeds.some(function (f) { return f.url === url; })) return;
      const feedName = name || hostnameFromUrl(url);
      feeds.push({ url: url, name: feedName });
      saveFeeds();
      renderFeedsList();
      fetchSingleFeed(url).then(function () {
        rebuildTimeline();
        renderTimeline();
      });
    }

    function removeFeed(url) {
      feeds = feeds.filter(function (f) { return f.url !== url; });
      delete cachedEntries[url];
      delete feedStatuses[url];
      saveFeeds();
      saveCachedEntries();
      renderFeedsList();
      rebuildTimeline();
      renderTimeline();
    }

    function hostnameFromUrl(url) {
      try { return new URL(url).hostname; } catch (_) { return url; }
    }

    // ── Fetch ─────────────────────────────────────────────────────────────────
    async function fetchAllFeeds() {
      if (feeds.length === 0) {
        updateStatus();
        renderTimeline();
        return;
      }
      feeds.forEach(function (f) {
        feedStatuses[f.url] = feedStatuses[f.url] || 'loading';
      });
      renderFeedsList();
      await Promise.all(feeds.map(function (f) { return fetchSingleFeed(f.url); }));
      rebuildTimeline();
      renderTimeline();
    }

    async function fetchSingleFeed(url) {
      feedStatuses[url] = 'loading';
      renderFeedStatus(url);
      try {
        const r = await fetch(url, { mode: 'cors', cache: 'no-cache' });
        if (!r.ok) throw new Error('HTTP ' + r.status);
        const text = await r.text();
        const entries = parseAtom(text, url);
        cachedEntries[url] = entries;
        feedStatuses[url] = 'ok';
        saveCachedEntries();
        clearFeedError(url);
      } catch (err) {
        feedStatuses[url] = 'error';
        showFeedError(url, err.message);
      }
      renderFeedStatus(url);
    }

    // ── Atom XML parsing ──────────────────────────────────────────────────────
    function parseAtom(xmlText, feedUrl) {
      const parser = new DOMParser();
      const doc = parser.parseFromString(xmlText, 'application/xml');
      if (doc.querySelector('parsererror')) return [];

      const feedTitle = text(doc, 'feed > title') ||
                        text(doc, 'channel > title') ||
                        hostnameFromUrl(feedUrl);

      const itemEls = Array.from(
        doc.querySelectorAll('feed > entry, channel > item')
      );

      return itemEls.map(function (el) {
        const title   = text(el, 'title') || '';
        const content = text(el, 'content') || text(el, 'summary') ||
                        text(el, 'description') || '';
        const published = text(el, 'published') || text(el, 'updated') ||
                          text(el, 'pubDate') || '';
        const linkEl  = el.querySelector('link');
        const link    = (linkEl && linkEl.getAttribute('href')) ||
                        (linkEl && linkEl.textContent) || feedUrl;

        return {
          feedUrl:   feedUrl,
          feedTitle: feedTitle,
          title:     title,
          content:   content,
          published: published ? new Date(published) : new Date(0),
          link:      link,
          tagInfo:   parseTag(title),
        };
      });
    }

    function text(el, selector) {
      const node = el.querySelector(selector);
      if (!node) return '';
      // For CDATA / HTML content nodes
      return node.textContent.trim();
    }

    function parseTag(title) {
      // Titles from the feed look like "CRITICAL SUCCESS: CharacterName"
      // or "PASS: CharacterName"
      const known = [
        'CRITICAL SUCCESS', 'CRITICAL FAILURE', 'PASS', 'FAIL',
        'GAME OVER', 'LEVEL UP', 'THOUGHT INTERNALIZED',
        'SUBSTANCE USED', 'REST', 'MANUAL',
      ];
      for (let i = 0; i < known.length; i++) {
        if (title.toUpperCase().startsWith(known[i] + ':') ||
            title.toUpperCase().startsWith(known[i] + ' ')) {
          return { label: known[i], cls: tagClass(known[i]) };
        }
      }
      return { label: '', cls: '' };
    }

    function tagClass(label) {
      const map = {
        'CRITICAL SUCCESS':   'critical-success',
        'CRITICAL FAILURE':   'critical-failure',
        'PASS':               'check-pass',
        'FAIL':               'check-fail',
        'GAME OVER':          'game-over',
        'LEVEL UP':           'level-up',
        'THOUGHT INTERNALIZED': 'thought',
        'SUBSTANCE USED':     'substance',
        'REST':               'rest',
        'MANUAL':             'manual',
      };
      return map[label] || '';
    }

    // ── Timeline build ────────────────────────────────────────────────────────
    function rebuildTimeline() {
      const all = [];
      Object.values(cachedEntries).forEach(function (entries) {
        entries.forEach(function (e) { all.push(e); });
      });
      all.sort(function (a, b) { return b.published - a.published; });
      allEntries = all;
      rendered = 0;
    }

    // ── Render: timeline ──────────────────────────────────────────────────────
    function renderTimeline() {
      const container = document.getElementById('timeline');
      const moreArea  = document.getElementById('load-more-area');

      if (allEntries.length === 0 && feeds.length === 0) {
        container.innerHTML = '<div class="state-empty">No feeds added yet.<br>Add a player\'s site URL above to get started.</div>';
        moreArea.innerHTML = '';
        updateStatus();
        return;
      }

      if (allEntries.length === 0) {
        const anyLoading = Object.values(feedStatuses).some(function (s) { return s === 'loading'; });
        if (anyLoading) {
          container.innerHTML = '<div class="state-loading">Tuning in...</div>';
        } else {
          container.innerHTML = '<div class="state-empty">No entries found in connected feeds.</div>';
        }
        moreArea.innerHTML = '';
        updateStatus();
        return;
      }

      const batch = allEntries.slice(0, rendered + PAGE_SIZE);
      rendered = batch.length;

      const html = batch.map(function (e) { return renderEntry(e); }).join('');
      container.innerHTML = html;

      if (rendered < allEntries.length) {
        moreArea.innerHTML = '<div class="load-more"><button class="btn-load-more" id="btn-load-more">Load more entries</button></div>';
        document.getElementById('btn-load-more').addEventListener('click', function () {
          rendered += PAGE_SIZE;
          renderTimeline();
          window.scrollBy(0, 100);
        });
      } else {
        moreArea.innerHTML = '';
      }

      updateStatus();
    }

    function renderCachedTimeline() {
      if (Object.keys(cachedEntries).length === 0) return;
      rebuildTimeline();
      renderTimeline();
    }

    function renderEntry(e) {
      const cls   = e.tagInfo.cls ? 'entry entry-' + e.tagInfo.cls : 'entry';
      const tagHtml = e.tagInfo.label
        ? '<span class="tag tag-' + e.tagInfo.cls + '">' + esc(e.tagInfo.label) + '</span>'
        : '';
      const ts = formatDate(e.published);
      const content = esc(e.content.replace(/<[^>]+>/g, '').slice(0, 300)) +
                      (e.content.length > 300 ? '…' : '');
      return '<article class="' + cls + '">' +
        '<div class="entry-meta">' +
          '<span class="entry-author">' + esc(e.feedTitle) + '</span>' +
          '<span class="entry-time">' + ts + '</span>' +
          tagHtml +
        '</div>' +
        '<p class="entry-content">' + content + '</p>' +
        '<a class="entry-link" href="' + esc(e.link) + '" target="_blank" rel="noopener">View entry &rarr;</a>' +
      '</article>';
    }

    function formatDate(d) {
      if (!d || d.getTime() === 0) return '';
      return d.toLocaleDateString(undefined, {
        year: 'numeric', month: 'short', day: 'numeric',
        hour: '2-digit', minute: '2-digit',
      });
    }

    function esc(s) {
      return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
    }

    // ── Render: feeds list ────────────────────────────────────────────────────
    function renderFeedsList() {
      const container = document.getElementById('feeds-list');
      if (feeds.length === 0) {
        container.innerHTML = '<p style="font-size:0.8rem;color:var(--muted);padding:0.25rem 0">No feeds connected.</p>';
        return;
      }
      container.innerHTML = feeds.map(function (f) {
        const status = feedStatuses[f.url] || 'loading';
        const statusLabel = status === 'ok' ? '&#10003; ok' :
                            status === 'error' ? '&#10007; error' : '&#8230;';
        const statusCls = 'feed-status-' + status;
        return '<div class="feed-item" data-url="' + esc(f.url) + '">' +
          '<span class="feed-item-name">' + esc(f.name) + '</span>' +
          '<span class="feed-item-url">' + esc(f.url) + '</span>' +
          '<span class="feed-item-status ' + statusCls + '">' + statusLabel + '</span>' +
          '<button class="btn-remove" data-url="' + esc(f.url) + '">Remove</button>' +
        '</div>';
      }).join('');

      container.querySelectorAll('.btn-remove').forEach(function (btn) {
        btn.addEventListener('click', function () {
          removeFeed(btn.dataset.url);
        });
      });
    }

    function renderFeedStatus(url) {
      const el = document.querySelector('.feed-item[data-url="' + CSS.escape(url) + '"] .feed-item-status');
      if (!el) return;
      const status = feedStatuses[url] || 'loading';
      el.className = 'feed-item-status feed-status-' + status;
      el.innerHTML = status === 'ok' ? '&#10003; ok' :
                     status === 'error' ? '&#10007; error' : '&#8230;';
    }

    // ── Errors ────────────────────────────────────────────────────────────────
    function showFeedError(url, msg) {
      const area = document.getElementById('error-area');
      const id = 'err-' + btoa(url).replace(/[^a-z0-9]/gi, '');
      let el = document.getElementById(id);
      if (!el) {
        el = document.createElement('div');
        el.id = id;
        el.className = 'feed-error';
        area.appendChild(el);
      }
      const isCors = msg.toLowerCase().includes('failed') || msg === 'Failed to fetch';
      el.innerHTML = '<strong>' + esc(url) + '</strong> &mdash; ' + esc(msg) +
        (isCors ? '<br><span style="opacity:0.7">CORS issue? The feed server must allow cross-origin requests, or host the timeline on the same origin.</span>' : '');
    }

    function clearFeedError(url) {
      const id = 'err-' + btoa(url).replace(/[^a-z0-9]/gi, '');
      const el = document.getElementById(id);
      if (el) el.remove();
    }

    // ── Status bar ────────────────────────────────────────────────────────────
    function updateStatus() {
      const el = document.getElementById('status-text');
      const total = allEntries.length;
      const feedCount = feeds.length;
      if (feedCount === 0) {
        el.textContent = 'No feeds connected';
        return;
      }
      const loading = Object.values(feedStatuses).filter(function (s) { return s === 'loading'; }).length;
      const errors  = Object.values(feedStatuses).filter(function (s) { return s === 'error'; }).length;
      let msg = total + ' ' + (total === 1 ? 'entry' : 'entries');
      msg += ' from ' + feedCount + ' ' + (feedCount === 1 ? 'feed' : 'feeds');
      if (loading > 0) msg += ' &middot; <em>fetching ' + loading + '…</em>';
      if (errors > 0)  msg += ' &middot; <span style="color:#cc6666">' + errors + ' failed</span>';
      el.innerHTML = '<span class="status-count">' + msg + '</span>';
    }

    // ── localStorage cache ────────────────────────────────────────────────────
    function saveCachedEntries() {
      const data = {};
      Object.entries(cachedEntries).forEach(function (kv) {
        data[kv[0]] = kv[1].map(function (e) {
          return Object.assign({}, e, { published: e.published.toISOString() });
        });
      });
      try { localStorage.setItem('ie-timeline-cache', JSON.stringify(data)); } catch (_) {}
    }

    function loadCachedEntries() {
      try {
        const raw = localStorage.getItem('ie-timeline-cache');
        if (!raw) return;
        const data = JSON.parse(raw);
        Object.entries(data).forEach(function (kv) {
          cachedEntries[kv[0]] = kv[1].map(function (e) {
            return Object.assign({}, e, { published: new Date(e.published) });
          });
        });
      } catch (_) {}
    }

    // ── Add feed button ───────────────────────────────────────────────────────
    document.getElementById('btn-add-feed').addEventListener('click', function () {
      const input = document.getElementById('add-feed-input');
      const val = input.value.trim();
      if (!val || !val.startsWith('http')) return;
      // Accept either a site URL or a direct feed URL
      const url = val.endsWith('.xml') ? val : val.replace(/\/$/, '') + '/feed.xml';
      addFeed(url, hostnameFromUrl(val));
      input.value = '';
    });

    document.getElementById('add-feed-input').addEventListener('keydown', function (e) {
      if (e.key === 'Enter') document.getElementById('btn-add-feed').click();
    });

    document.getElementById('btn-refresh').addEventListener('click', function () {
      clearTimeout(refreshTimer);
      fetchAllFeeds().then(scheduledRefresh);
    });

    // ── Init ──────────────────────────────────────────────────────────────────
    boot();
  })();
  </script>
</body>
</html>
"#.to_string()
}

// ── Entry type label ──────────────────────────────────────────────────────────

fn entry_type_display(et: EntryType) -> String {
    match et {
        EntryType::CriticalSuccess => "CRITICAL SUCCESS".to_string(),
        EntryType::CriticalFailure => "CRITICAL FAILURE".to_string(),
        EntryType::CheckPass => "PASS".to_string(),
        EntryType::CheckFail => "FAIL".to_string(),
        EntryType::GameOver => "GAME OVER".to_string(),
        EntryType::LevelUp => "LEVEL UP".to_string(),
        EntryType::ThoughtInternalized => "THOUGHT INTERNALIZED".to_string(),
        EntryType::SubstanceUsed => "SUBSTANCE USED".to_string(),
        EntryType::Rest => "REST".to_string(),
        EntryType::Manual => "MANUAL".to_string(),
    }
}
