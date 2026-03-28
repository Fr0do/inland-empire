use crate::journal::{EntryType, Genre, JournalEntry};

struct GenreTheme {
    bg: &'static str,
    bg2: &'static str,
    text: &'static str,
    accent: &'static str,
    accent2: &'static str,
    muted: &'static str,
    font_stack: &'static str,
    header_font: &'static str,
    tag_bg: &'static str,
    border: &'static str,
    extra_css: &'static str,
}

fn theme(genre: Genre) -> GenreTheme {
    match genre {
        Genre::Noir => GenreTheme {
            bg: "#1a1a2e",
            bg2: "#16213e",
            text: "#d4a574",
            accent: "#f0c070",
            accent2: "#c8956c",
            muted: "#8a7060",
            font_stack: "'Courier New', 'Lucida Console', 'Courier', monospace",
            header_font: "'Georgia', 'Times New Roman', serif",
            tag_bg: "#2a1f10",
            border: "#3a2e20",
            extra_css: r#"
  body::before {
    content: '';
    position: fixed;
    inset: 0;
    background: repeating-linear-gradient(
      0deg,
      transparent,
      transparent 2px,
      rgba(0,0,0,0.03) 2px,
      rgba(0,0,0,0.03) 4px
    );
    pointer-events: none;
    z-index: 1000;
  }
  h1 { letter-spacing: 0.15em; text-transform: uppercase; }
  .entry { border-left: 2px solid #3a2e20; }
  .entry:hover { border-left-color: #d4a574; }
  time { font-size: 0.75rem; opacity: 0.6; letter-spacing: 0.1em; }
"#,
        },
        Genre::SciFi => GenreTheme {
            bg: "#0a0a1a",
            bg2: "#080818",
            text: "#00cc77",
            accent: "#00ff88",
            accent2: "#00ddaa",
            muted: "#004433",
            font_stack: "'Share Tech Mono', 'Courier New', monospace",
            header_font: "'Share Tech Mono', 'Courier New', monospace",
            tag_bg: "#001a0d",
            border: "#003322",
            extra_css: r#"
  @keyframes scanline {
    0% { transform: translateY(-100%); }
    100% { transform: translateY(100vh); }
  }
  body::after {
    content: '';
    position: fixed;
    left: 0; right: 0;
    height: 3px;
    background: rgba(0,255,136,0.06);
    animation: scanline 8s linear infinite;
    pointer-events: none;
    z-index: 1000;
  }
  body::before {
    content: '';
    position: fixed;
    inset: 0;
    background: repeating-linear-gradient(
      0deg,
      transparent,
      transparent 1px,
      rgba(0,255,136,0.015) 1px,
      rgba(0,255,136,0.015) 2px
    );
    pointer-events: none;
    z-index: 999;
  }
  h1 { letter-spacing: 0.2em; text-transform: uppercase; font-size: 1.8rem; }
  h1::before { content: '> '; opacity: 0.5; }
  .subtitle::before { content: '// '; opacity: 0.4; }
  .entry { border: 1px solid #003322; font-size: 0.9rem; }
  .entry::before { content: attr(data-index); position: absolute; top: 0.5rem; right: 0.75rem; font-size: 0.65rem; opacity: 0.3; }
  .entry { position: relative; }
  time { font-size: 0.7rem; opacity: 0.5; text-transform: uppercase; letter-spacing: 0.15em; }
  p { line-height: 1.8; }
"#,
        },
        Genre::Horror => GenreTheme {
            bg: "#0d0d0d",
            bg2: "#080808",
            text: "#c8c8c8",
            accent: "#cc1111",
            accent2: "#991111",
            muted: "#444",
            font_stack: "'Palatino Linotype', 'Book Antiqua', 'Georgia', serif",
            header_font: "'Palatino Linotype', 'Book Antiqua', 'Georgia', serif",
            tag_bg: "#1a0000",
            border: "#2a0000",
            extra_css: r#"
  @keyframes flicker {
    0%, 95%, 100% { opacity: 1; }
    96% { opacity: 0.85; }
    97% { opacity: 1; }
    98% { opacity: 0.9; }
    99% { opacity: 1; }
  }
  @keyframes redpulse {
    0%, 100% { text-shadow: 0 0 6px #8b0000; }
    50% { text-shadow: 0 0 18px #cc1111, 0 0 30px #880000; }
  }
  body { animation: flicker 12s infinite; }
  h1 { animation: redpulse 4s ease-in-out infinite; color: #cc1111; font-style: italic; }
  .entry { border-top: 1px solid #1a0000; }
  .entry:hover { background: #110000; }
  time { font-size: 0.75rem; opacity: 0.4; font-style: italic; }
  p { font-size: 1.05rem; line-height: 2; }
  .tag-game-over { font-size: 2rem !important; text-align: center; letter-spacing: 0.3em; }
"#,
        },
        Genre::Fantasy => GenreTheme {
            bg: "#2a1f14",
            bg2: "#221a10",
            text: "#e8d5a3",
            accent: "#c9a84c",
            accent2: "#b8943c",
            muted: "#7a6040",
            font_stack: "'Palatino Linotype', 'Book Antiqua', 'Garamond', 'Georgia', serif",
            header_font: "'Palatino Linotype', 'Book Antiqua', 'Garamond', serif",
            tag_bg: "#1a1206",
            border: "#3a2a10",
            extra_css: r#"
  body {
    background-image:
      radial-gradient(ellipse at 20% 20%, rgba(201,168,76,0.04) 0%, transparent 60%),
      radial-gradient(ellipse at 80% 80%, rgba(201,168,76,0.03) 0%, transparent 60%);
  }
  h1 {
    font-size: 2.4rem;
    letter-spacing: 0.08em;
    text-shadow: 0 0 20px rgba(201,168,76,0.3), 0 2px 4px rgba(0,0,0,0.5);
    border-bottom: 1px solid #3a2a10;
    padding-bottom: 0.5rem;
  }
  .subtitle { font-style: italic; opacity: 0.75; }
  .entry { border: 1px solid #3a2a10; border-radius: 2px; }
  .entry::before {
    content: '❧';
    position: absolute;
    left: -1.2rem;
    top: 0.6rem;
    color: #c9a84c;
    opacity: 0.3;
    font-size: 1.2rem;
  }
  .entry { position: relative; }
  time { font-style: italic; opacity: 0.5; font-size: 0.8rem; }
  p { line-height: 1.9; font-size: 1.05rem; }
  footer { border-top: 1px solid #3a2a10; font-style: italic; }
"#,
        },
        Genre::Cyberpunk => GenreTheme {
            bg: "#0f0f23",
            bg2: "#0a0a1a",
            text: "#e0e0e0",
            accent: "#ff0066",
            accent2: "#00ffff",
            muted: "#404060",
            font_stack: "'Share Tech Mono', 'Courier New', 'Consolas', monospace",
            header_font: "'Share Tech Mono', 'Courier New', monospace",
            tag_bg: "#1a0011",
            border: "#2a003a",
            extra_css: r#"
  @keyframes glitch {
    0%, 90%, 100% { text-shadow: 2px 0 #ff0066, -2px 0 #00ffff; }
    91% { text-shadow: -4px 0 #ff0066, 4px 0 #00ffff; transform: translate(2px); }
    92% { text-shadow: 4px 0 #00ffff, -4px 0 #ff0066; transform: translate(-2px); }
    93% { text-shadow: 2px 0 #ff0066, -2px 0 #00ffff; transform: translate(0); }
  }
  @keyframes neonpulse {
    0%, 100% { box-shadow: 0 0 4px #ff0066, inset 0 0 4px rgba(255,0,102,0.1); }
    50% { box-shadow: 0 0 12px #ff0066, 0 0 20px rgba(255,0,102,0.3), inset 0 0 8px rgba(255,0,102,0.15); }
  }
  h1 { animation: glitch 8s infinite; color: #ff0066; letter-spacing: 0.15em; text-transform: uppercase; font-size: 2rem; }
  .subtitle { color: #00ffff; opacity: 0.8; font-size: 0.8rem; letter-spacing: 0.2em; text-transform: uppercase; }
  .entry { border: 1px solid #2a003a; animation: neonpulse 6s ease-in-out infinite; }
  .entry:hover { border-color: #ff0066; box-shadow: 0 0 16px rgba(255,0,102,0.3); }
  time { color: #00ffff; font-size: 0.7rem; opacity: 0.7; letter-spacing: 0.1em; text-transform: uppercase; }
  p { line-height: 1.8; font-size: 0.9rem; }
  .tag { border: 1px solid currentColor; }
  footer { border-top: 1px solid #2a003a; color: #00ffff; opacity: 0.5; font-size: 0.75rem; }
"#,
        },
    }
}

fn entry_type_css_class(et: EntryType) -> &'static str {
    match et {
        EntryType::CriticalSuccess => "entry-critical-success",
        EntryType::CriticalFailure => "entry-critical-failure",
        EntryType::GameOver => "entry-game-over",
        EntryType::LevelUp => "entry-level-up",
        EntryType::ThoughtInternalized => "entry-thought",
        EntryType::SubstanceUsed => "entry-substance",
        EntryType::Rest => "entry-rest",
        EntryType::Manual => "entry-manual",
    }
}

fn format_timestamp(ts: &chrono::DateTime<chrono::Utc>) -> String {
    ts.format("%Y-%m-%d %H:%M UTC").to_string()
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn generate_storybook(
    character_name: &str,
    archetype: &str,
    level: u32,
    genre: Genre,
    entries: &[JournalEntry],
) -> String {
    let t = theme(genre);
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let genre_label = genre.to_string();
    let name_escaped = escape_html(character_name);
    let archetype_escaped = escape_html(archetype);

    // Build entry HTML
    let mut entries_html = String::new();
    for (i, entry) in entries.iter().enumerate() {
        let cls = entry_type_css_class(entry.entry_type);
        let tag_label = entry.entry_type.to_string();
        let ts = format_timestamp(&entry.timestamp);
        let content = escape_html(&entry.content);

        entries_html.push_str(&format!(
            r#"    <article class="entry {cls}" data-index="{idx:04}">
      <div class="entry-meta">
        <time datetime="{ts_raw}">{ts}</time>
        <span class="tag tag-{cls}">{tag}</span>
      </div>
      <p>{content}</p>
    </article>
"#,
            cls = cls,
            idx = i + 1,
            ts_raw = entry.timestamp.to_rfc3339(),
            ts = ts,
            tag = tag_label,
            content = content,
        ));
    }

    let entry_count = entries.len();

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{name} — {genre} Journal</title>
  <style>
    *, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}

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

    /* ── Header ── */
    header {{
      text-align: center;
      padding: 3rem 0 2.5rem;
      margin-bottom: 3rem;
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

    .entry-count {{
      margin-top: 1rem;
      font-size: 0.75rem;
      color: var(--muted);
      letter-spacing: 0.08em;
    }}

    /* ── Entries ── */
    main {{
      display: flex;
      flex-direction: column;
      gap: 1.5rem;
    }}

    .entry {{
      background: var(--bg2);
      padding: 1.25rem 1.5rem;
      border-radius: 3px;
      transition: background 0.2s ease, transform 0.1s ease;
    }}

    .entry:hover {{
      transform: translateX(2px);
    }}

    .entry-meta {{
      display: flex;
      align-items: center;
      gap: 0.75rem;
      margin-bottom: 0.6rem;
      flex-wrap: wrap;
    }}

    time {{
      color: var(--muted);
      font-size: 0.8rem;
    }}

    .tag {{
      font-size: 0.7rem;
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
    }}

    /* ── Entry type variants ── */
    .entry-critical-success {{
      border-left: 3px solid #22cc66;
    }}
    .entry-critical-success .tag {{
      color: #22cc66;
    }}
    .entry-critical-success p {{
      text-shadow: 0 0 8px rgba(34,204,102,0.15);
    }}

    .entry-critical-failure {{
      border-left: 3px solid #cc2222;
    }}
    .entry-critical-failure .tag {{
      color: #cc2222;
    }}
    .entry-critical-failure p {{
      text-shadow: 0 0 8px rgba(204,34,34,0.15);
    }}

    .entry-game-over {{
      border: 2px solid #cc2222;
      text-align: center;
      padding: 2rem;
    }}
    .entry-game-over p {{
      font-size: 1.3rem;
      font-weight: 700;
      color: #cc2222;
      text-shadow: 0 0 20px rgba(204,34,34,0.4);
      letter-spacing: 0.08em;
    }}
    .entry-game-over .tag {{
      color: #cc2222;
      font-size: 0.85rem;
      letter-spacing: 0.2em;
    }}

    .entry-level-up {{
      border-left: 3px solid #ccaa00;
    }}
    .entry-level-up .tag {{
      color: #ccaa00;
    }}
    .entry-level-up p {{
      color: #e8cc66;
    }}

    .entry-thought {{
      border-left: 3px solid var(--accent2);
      padding-left: 2rem;
    }}
    .entry-thought p {{
      font-style: italic;
      opacity: 0.9;
    }}
    .entry-thought .tag {{
      color: var(--accent2);
    }}

    .entry-substance {{
      border-left: 3px solid #aa44cc;
    }}
    .entry-substance .tag {{
      color: #cc66ee;
    }}
    .entry-substance p {{
      color: #d0a0e8;
    }}

    .entry-rest {{
      border-left: 3px solid var(--muted);
      opacity: 0.75;
    }}
    .entry-rest .tag {{
      color: var(--muted);
    }}
    .entry-rest p {{
      font-size: 0.9rem;
    }}

    .entry-manual .tag {{
      color: var(--text);
      opacity: 0.6;
    }}

    /* ── Footer ── */
    footer {{
      text-align: center;
      margin-top: 4rem;
      padding-top: 1.5rem;
      font-size: 0.75rem;
      color: var(--muted);
      opacity: 0.6;
      letter-spacing: 0.08em;
    }}

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
      opacity: 0.7;
      transition: opacity 0.2s;
    }}
    .scroll-top:hover {{ opacity: 1; }}

    /* ── Responsive ── */
    @media (max-width: 600px) {{
      .container {{ padding: 1rem 1rem 3rem; }}
      header {{ padding: 2rem 0 1.5rem; }}
      h1 {{ font-size: 1.6rem; }}
      .entry {{ padding: 1rem 1.1rem; }}
      .entry-game-over p {{ font-size: 1.1rem; }}
    }}

    /* ── Genre-specific overrides ── */
{extra_css}
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>{name}</h1>
      <p class="subtitle">{archetype} &middot; Level {level} &middot; {genre}</p>
      <p class="entry-count">{count} {entries_word}</p>
    </header>
    <main>
{entries}    </main>
    <footer>
      <p>Generated by Inland Empire &middot; {today}</p>
    </footer>
  </div>
  <a class="scroll-top" href="" onclick="window.scrollTo(0,0);return false" title="Back to top">&#8593;</a>
</body>
</html>
"#,
        name = name_escaped,
        genre = genre_label,
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
        archetype = archetype_escaped,
        level = level,
        count = entry_count,
        entries_word = if entry_count == 1 { "entry" } else { "entries" },
        entries = entries_html,
        today = today,
    )
}
