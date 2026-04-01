use crate::character::Character;
use crate::equipment::EquipSlot;
use crate::skills::{Attribute, Skill};
use crate::stats::compute_stats;
use crate::storybook::{self, escape_html, entry_type_css_class, format_timestamp};
use crate::types::CheckColor;
use tiny_http::{Header, Response, Server};

pub fn serve(initial_ch: &Character, port: u16, no_open: bool) {
    let addr = format!("0.0.0.0:{port}");
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind to {addr}: {e}");
            return;
        }
    };

    let url = format!("http://localhost:{port}");
    println!("Dashboard running at {url}");
    println!("Character: {}", initial_ch.name);
    println!("Press Ctrl+C to stop");

    if !no_open {
        let _ = open::that(&url);
    }

    for request in server.incoming_requests() {
        let ch = match Character::load_active() {
            Ok(c) => c,
            Err(e) => {
                let body = format!(
                    "<html><body><h1>Error</h1><p>{}</p></body></html>",
                    escape_html(&e)
                );
                let resp = Response::from_string(body)
                    .with_header(html_header())
                    .with_status_code(500);
                let _ = request.respond(resp);
                continue;
            }
        };

        let path = request.url().split('?').next().unwrap_or("/").to_string();

        let (body, content_type, status) = match path.as_str() {
            "/" => (page_character(&ch), "text/html; charset=utf-8", 200),
            "/checks" => (page_checks(&ch), "text/html; charset=utf-8", 200),
            "/journal" => (page_journal(&ch), "text/html; charset=utf-8", 200),
            "/thoughts" => (page_thoughts(&ch), "text/html; charset=utf-8", 200),
            "/equipment" => (page_equipment(&ch), "text/html; charset=utf-8", 200),
            "/leaderboard" => (page_leaderboard(), "text/html; charset=utf-8", 200),
            "/style.css" => (dashboard_css(&ch), "text/css", 200),
            "/api/character" => {
                let json = serde_json::to_string_pretty(&ch).unwrap_or_else(|e| {
                    format!("{{\"error\":\"{e}\"}}")
                });
                (json, "application/json", 200)
            }
            "/api/stats" => {
                let stats = compute_stats(&ch);
                let json = serde_json::to_string_pretty(&stats).unwrap_or_else(|e| {
                    format!("{{\"error\":\"{e}\"}}")
                });
                (json, "application/json", 200)
            }
            _ => (page_not_found(), "text/html; charset=utf-8", 404),
        };

        let ct_header = Header::from_bytes("Content-Type", content_type).unwrap();
        let resp = Response::from_string(body)
            .with_header(ct_header)
            .with_status_code(status);
        let _ = request.respond(resp);
    }
}

fn html_header() -> Header {
    Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap()
}

// ── Layout ────────────────────────────────────────────────────────────────────

fn render_layout(ch: &Character, title: &str, active_nav: &str, body: &str) -> String {
    let _t = storybook::theme(ch.genre);
    let name = escape_html(&ch.name);
    let archetype = escape_html(&ch.archetype);
    let level = ch.level;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let nav_items = [
        ("/", "Character"),
        ("/checks", "Checks"),
        ("/journal", "Journal"),
        ("/thoughts", "Thoughts"),
        ("/equipment", "Equipment"),
        ("/leaderboard", "Leaderboard"),
    ];

    let nav_html: String = nav_items
        .iter()
        .map(|(href, label)| {
            let active = if *label == active_nav { " class=\"nav-active\"" } else { "" };
            format!("<a href=\"{href}\"{active}>{label}</a>")
        })
        .collect::<Vec<_>>()
        .join("\n        ");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta http-equiv="refresh" content="5">
  <title>{title} — Inland Empire</title>
  <link rel="stylesheet" href="/style.css">
</head>
<body>
  <nav class="navbar">
    <div class="nav-brand">INLAND EMPIRE</div>
    <div class="nav-links">
        {nav_html}
    </div>
  </nav>
  <header class="site-header">
    <h1>{name}</h1>
    <div class="header-meta">
      <span class="archetype">{archetype}</span>
      <span class="level-badge">LVL {level}</span>
    </div>
  </header>
  <main>
{body}
  </main>
  <footer>Inland Empire Dashboard &bull; {today}</footer>
</body>
</html>"#,
        title = escape_html(title),
    )
}

// ── CSS ───────────────────────────────────────────────────────────────────────

fn dashboard_css(ch: &Character) -> String {
    let t = storybook::theme(ch.genre);
    format!(
        r#":root {{
  --bg:      {bg};
  --bg2:     {bg2};
  --text:    {text};
  --accent:  {accent};
  --accent2: {accent2};
  --muted:   {muted};
  --border:  {border};
  --tag-bg:  {tag_bg};
  --intellect: #4488cc;
  --psyche:    #cc44cc;
  --physique:  #cc4444;
  --motorics:  #ccaa44;
}}

*, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}

body {{
  background: var(--bg);
  color: var(--text);
  font-family: {font_stack};
  min-height: 100vh;
  line-height: 1.6;
}}

a {{ color: var(--accent); text-decoration: none; }}
a:hover {{ text-decoration: underline; }}

/* ── Navbar ── */
.navbar {{
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6rem 2rem;
  background: var(--bg2);
  border-bottom: 1px solid var(--border);
  position: sticky;
  top: 0;
  z-index: 100;
}}
.nav-brand {{
  font-family: {header_font};
  font-size: 0.85rem;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--muted);
}}
.nav-links {{
  display: flex;
  gap: 0.25rem;
}}
.nav-links a {{
  padding: 0.35rem 0.9rem;
  border-radius: 3px;
  font-size: 0.85rem;
  letter-spacing: 0.06em;
  color: var(--text);
  transition: background 0.15s, color 0.15s;
}}
.nav-links a:hover {{
  background: rgba(255,255,255,0.07);
  text-decoration: none;
  color: var(--accent);
}}
.nav-links a.nav-active {{
  background: var(--accent);
  color: var(--bg);
  font-weight: bold;
}}

/* ── Header ── */
.site-header {{
  padding: 1.5rem 2rem 1rem;
  border-bottom: 1px solid var(--border);
}}
.site-header h1 {{
  font-family: {header_font};
  font-size: 2rem;
  color: var(--accent);
  margin-bottom: 0.25rem;
}}
.header-meta {{
  display: flex;
  gap: 1rem;
  align-items: center;
}}
.archetype {{
  font-size: 0.85rem;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.1em;
}}
.level-badge {{
  background: var(--accent);
  color: var(--bg);
  font-size: 0.75rem;
  font-weight: bold;
  padding: 0.15rem 0.5rem;
  border-radius: 2px;
  letter-spacing: 0.08em;
}}

/* ── Main layout ── */
main {{
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}}

/* ── Section headings ── */
.section-title {{
  font-family: {header_font};
  font-size: 1.1rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--accent);
  border-bottom: 1px solid var(--border);
  padding-bottom: 0.4rem;
  margin-bottom: 1rem;
}}

/* ── Vitals ── */
.vitals-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}}
.vital-card {{
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 1rem;
}}
.vital-label {{
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--muted);
  margin-bottom: 0.4rem;
}}
.vital-value {{
  font-size: 1.4rem;
  font-weight: bold;
  color: var(--accent);
  margin-bottom: 0.4rem;
}}
.bar-track {{
  height: 10px;
  background: rgba(255,255,255,0.08);
  border-radius: 2px;
  overflow: hidden;
}}
.bar-fill {{
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}}
.bar-health {{ background: #cc4444; }}
.bar-morale {{ background: #cc44cc; }}
.bar-xp     {{ background: var(--accent); }}

/* ── Stat cards ── */
.stat-cards {{
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 0.75rem;
  margin-bottom: 2rem;
}}
.stat-card {{
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0.9rem 1rem;
  text-align: center;
}}
.stat-card .stat-num {{
  font-size: 2rem;
  font-weight: bold;
  color: var(--accent);
  line-height: 1;
}}
.stat-card .stat-label {{
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--muted);
  margin-top: 0.3rem;
}}

/* ── Attributes / Skills ── */
.attributes-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}}
.attribute-block {{
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 1rem;
}}
.attribute-header {{
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 0.75rem;
  padding-bottom: 0.4rem;
  border-bottom: 1px solid var(--border);
}}
.attribute-name {{
  font-size: 0.8rem;
  font-weight: bold;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}}
.attr-intellect {{ color: var(--intellect); }}
.attr-psyche    {{ color: var(--psyche); }}
.attr-physique  {{ color: var(--physique); }}
.attr-motorics  {{ color: var(--motorics); }}
.attribute-value {{
  font-size: 1.5rem;
  font-weight: bold;
}}
.skill-row {{
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}}
.skill-name {{
  width: 140px;
  font-size: 0.8rem;
  flex-shrink: 0;
}}
.skill-level {{
  width: 24px;
  text-align: right;
  font-size: 0.8rem;
  font-weight: bold;
  color: var(--accent);
  flex-shrink: 0;
}}
.skill-bar-wrap {{
  flex: 1;
  height: 6px;
  background: rgba(255,255,255,0.08);
  border-radius: 3px;
  overflow: hidden;
}}
.skill-bar {{
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease;
}}
.skill-modifier {{
  font-size: 0.7rem;
  width: 30px;
  text-align: right;
  flex-shrink: 0;
}}
.mod-pos {{ color: #44cc88; }}
.mod-neg {{ color: #cc4444; }}
.skill-domain {{
  font-size: 0.68rem;
  color: var(--muted);
  margin-left: 164px;
  margin-top: -0.3rem;
  margin-bottom: 0.4rem;
  line-height: 1.3;
}}

/* ── Tables ── */
.data-table {{
  width: 100%;
  border-collapse: collapse;
  font-size: 0.82rem;
  margin-bottom: 1.5rem;
}}
.data-table th {{
  text-align: left;
  padding: 0.4rem 0.6rem;
  font-size: 0.72rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
  border-bottom: 1px solid var(--border);
}}
.data-table td {{
  padding: 0.35rem 0.6rem;
  border-bottom: 1px solid rgba(255,255,255,0.05);
  vertical-align: middle;
}}
.data-table tr:hover td {{
  background: rgba(255,255,255,0.03);
}}
.result-pass {{ color: #44cc88; font-weight: bold; }}
.result-fail {{ color: #cc4444; font-weight: bold; }}
.badge {{
  display: inline-block;
  font-size: 0.65rem;
  padding: 0.1rem 0.35rem;
  border-radius: 2px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: bold;
}}
.badge-white {{ background: rgba(200,200,200,0.15); color: #ccc; }}
.badge-red   {{ background: rgba(200,50,50,0.3);   color: #f88; }}

/* ── Journal entries ── */
.journal-list {{
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}}
.journal-entry {{
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0.9rem 1rem;
}}
.journal-entry-meta {{
  display: flex;
  gap: 0.75rem;
  align-items: center;
  margin-bottom: 0.5rem;
}}
.journal-entry-content {{
  font-size: 0.88rem;
  line-height: 1.7;
  color: var(--text);
  opacity: 0.9;
}}
.entry-tag {{
  font-size: 0.65rem;
  padding: 0.15rem 0.45rem;
  border-radius: 2px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  font-weight: bold;
  background: var(--tag-bg);
  color: var(--accent);
  border: 1px solid var(--border);
}}
.entry-time {{
  font-size: 0.72rem;
  color: var(--muted);
}}
.entry-critical-success .entry-tag {{ background: rgba(50,180,100,0.2);  color: #44cc88; border-color: #225533; }}
.entry-critical-failure .entry-tag  {{ background: rgba(180,50,50,0.2);   color: #cc4444; border-color: #552222; }}
.entry-game-over .entry-tag         {{ background: rgba(180,50,50,0.4);   color: #ff6666; border-color: #cc1111; }}
.entry-level-up .entry-tag          {{ background: rgba(240,192,112,0.2); color: #f0c070; border-color: #554422; }}
.entry-thought .entry-tag           {{ background: rgba(100,100,220,0.2); color: #8888ff; border-color: #333388; }}
.entry-substance .entry-tag         {{ background: rgba(180,80,180,0.2);  color: #cc88cc; border-color: #442244; }}
.entry-rest .entry-tag              {{ background: rgba(50,100,180,0.2);  color: #6699dd; border-color: #223355; }}
.entry-manual .entry-tag            {{ background: rgba(150,150,150,0.1); color: var(--muted); border-color: var(--border); }}
.entry-check-pass .entry-tag        {{ background: rgba(50,180,100,0.15); color: #44cc88; border-color: #224433; }}
.entry-check-fail .entry-tag        {{ background: rgba(180,100,50,0.15); color: #dd8844; border-color: #443322; }}

/* ── Thoughts ── */
.thoughts-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}}
.thought-card {{
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 1rem;
}}
.thought-name {{
  font-family: {header_font};
  font-size: 1rem;
  color: var(--accent);
  margin-bottom: 0.3rem;
}}
.thought-desc {{
  font-size: 0.82rem;
  color: var(--text);
  opacity: 0.8;
  margin-bottom: 0.6rem;
  line-height: 1.5;
}}
.thought-phase {{
  font-size: 0.75rem;
  margin-bottom: 0.5rem;
}}
.phase-internalized {{ color: #44cc88; }}
.phase-researching  {{ color: #ccaa44; }}
.modifier-badges {{
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}}
.mod-badge {{
  font-size: 0.7rem;
  padding: 0.15rem 0.4rem;
  border-radius: 2px;
  font-weight: bold;
}}
.mod-badge-pos {{ background: rgba(50,180,100,0.2);  color: #44cc88; }}
.mod-badge-neg {{ background: rgba(180,50,50,0.2);   color: #cc6666; }}

/* ── Equipment ── */
.equipment-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}}
.equip-slot-card {{
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 1rem;
}}
.equip-slot-name {{
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--muted);
  margin-bottom: 0.3rem;
}}
.equip-item-name {{
  font-size: 0.95rem;
  color: var(--accent);
  font-weight: bold;
  margin-bottom: 0.3rem;
}}
.equip-item-desc {{
  font-size: 0.78rem;
  color: var(--text);
  opacity: 0.75;
  margin-bottom: 0.5rem;
  line-height: 1.4;
}}
.equip-empty {{
  font-size: 0.85rem;
  color: var(--muted);
  font-style: italic;
}}

/* ── Inventory ── */
.inventory-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 0.75rem;
  margin-bottom: 2rem;
}}
.inventory-item {{
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0.75rem;
}}
.inventory-item-name {{
  font-size: 0.9rem;
  color: var(--accent);
  font-weight: bold;
}}
.inventory-count {{
  font-size: 0.75rem;
  color: var(--muted);
}}
.inventory-desc {{
  font-size: 0.78rem;
  color: var(--text);
  opacity: 0.75;
  margin-top: 0.3rem;
  line-height: 1.4;
}}

/* ── Active effects ── */
.effects-list {{
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  margin-bottom: 2rem;
}}
.effect-card {{
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0.75rem 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.5rem;
}}
.effect-name {{
  font-size: 0.9rem;
  color: var(--accent);
  font-weight: bold;
}}
.effect-remaining {{
  font-size: 0.75rem;
  color: var(--muted);
}}

/* ── Inline stat bars (for skill tables) ── */
.inline-bar-wrap {{
  width: 80px;
  height: 6px;
  background: rgba(255,255,255,0.08);
  border-radius: 3px;
  overflow: hidden;
  display: inline-block;
  vertical-align: middle;
}}
.inline-bar {{
  height: 100%;
  border-radius: 3px;
}}

/* ── 404 ── */
.not-found {{
  text-align: center;
  padding: 4rem 2rem;
}}
.not-found h2 {{
  font-family: {header_font};
  font-size: 3rem;
  color: var(--accent);
  margin-bottom: 1rem;
}}
.not-found p {{
  color: var(--muted);
  margin-bottom: 1.5rem;
}}

footer {{
  text-align: center;
  padding: 1.5rem;
  font-size: 0.72rem;
  color: var(--muted);
  border-top: 1px solid var(--border);
  margin-top: 2rem;
  letter-spacing: 0.08em;
}}

/* ── SVG Visualizations ── */
.dashboard-hero {{
  display: flex;
  gap: 2rem;
  justify-content: center;
  align-items: flex-start;
  margin-bottom: 2rem;
  padding: 1.5rem;
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
}}
.svg-container {{
  display: flex;
  justify-content: center;
}}
.svg-container svg {{
  max-width: 100%;
  height: auto;
}}
.sparkline-container {{
  background: var(--bg2);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 1rem;
  margin-bottom: 2rem;
}}
@media (max-width: 700px) {{
  .dashboard-hero {{
    flex-direction: column;
    align-items: center;
  }}
}}

{extra_css}
"#,
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

// ── SVG Visualizations ────────────────────────────────────────────────────────

pub(crate) fn svg_radar_chart(ch: &Character) -> String {
    let t = storybook::theme(ch.genre);
    let attrs = [
        Attribute::Intellect,
        Attribute::Psyche,
        Attribute::Physique,
        Attribute::Motorics,
    ];
    let attr_colors = ["#4488cc", "#cc44cc", "#cc4444", "#ccaa44"];
    let attr_labels = ["Intellect", "Psyche", "Physique", "Motorics"];

    let cx = 150.0_f64;
    let cy = 150.0_f64;
    let max_r = 120.0_f64;
    let max_val = 10.0_f64;

    // Axis angles (degrees): top, right, bottom, left
    let angles_deg: [f64; 4] = [-90.0, 0.0, 90.0, 180.0];

    // Compute average effective skill per attribute
    let mut avgs = [0.0_f64; 4];
    for (i, attr) in attrs.iter().enumerate() {
        let skills: Vec<Skill> = Skill::all()
            .iter()
            .copied()
            .filter(|s| s.attribute() == *attr)
            .collect();
        if !skills.is_empty() {
            let sum: f64 = skills.iter().map(|s| ch.effective_skill(*s) as f64).sum();
            avgs[i] = (sum / skills.len() as f64).min(max_val).max(0.0);
        }
    }

    // Grid rings
    let mut grid_svg = String::new();
    for level in [2u8, 4, 6, 8] {
        let r = max_r * level as f64 / max_val;
        let mut pts = String::new();
        for &ang in &angles_deg {
            let rad = ang.to_radians();
            let x = cx + r * rad.cos();
            let y = cy + r * rad.sin();
            pts.push_str(&format!("{x:.1},{y:.1} "));
        }
        grid_svg.push_str(&format!(
            "<polygon points=\"{pts}\" fill=\"none\" stroke=\"rgba(255,255,255,0.1)\" stroke-width=\"1\"/>\n"
        ));
    }

    // Axis lines
    let mut axes_svg = String::new();
    for (i, &ang) in angles_deg.iter().enumerate() {
        let rad = ang.to_radians();
        let x2 = cx + max_r * rad.cos();
        let y2 = cy + max_r * rad.sin();
        axes_svg.push_str(&format!(
            "<line x1=\"{cx:.1}\" y1=\"{cy:.1}\" x2=\"{x2:.1}\" y2=\"{y2:.1}\" stroke=\"{}\" stroke-width=\"1\" stroke-opacity=\"0.3\"/>\n",
            attr_colors[i]
        ));
    }

    // Data polygon
    let mut poly_pts = String::new();
    let mut dots_svg = String::new();
    for (i, &ang) in angles_deg.iter().enumerate() {
        let r = max_r * avgs[i] / max_val;
        let rad = ang.to_radians();
        let x = cx + r * rad.cos();
        let y = cy + r * rad.sin();
        poly_pts.push_str(&format!("{x:.1},{y:.1} "));
        dots_svg.push_str(&format!(
            "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"5\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
            t.accent, t.accent
        ));
    }

    // Labels at axis tips
    let mut labels_svg = String::new();
    for (i, &ang) in angles_deg.iter().enumerate() {
        let rad = ang.to_radians();
        let lx = cx + (max_r + 18.0) * rad.cos();
        let ly = cy + (max_r + 18.0) * rad.sin();
        let anchor = if i == 1 { "start" } else if i == 3 { "end" } else { "middle" };
        let dy = if i == 2 { "1em" } else if i == 0 { "-0.3em" } else { "0.35em" };
        labels_svg.push_str(&format!(
            "<text x=\"{lx:.1}\" y=\"{ly:.1}\" fill=\"{}\" font-size=\"10\" font-family=\"monospace\" text-anchor=\"{anchor}\" dy=\"{dy}\">{}</text>\n",
            attr_colors[i], attr_labels[i]
        ));
    }

    format!(
        r#"<svg width="300" height="300" viewBox="0 0 300 300" xmlns="http://www.w3.org/2000/svg" style="background:transparent">
{grid_svg}{axes_svg}<polygon points="{poly_pts}" fill="{accent_fill}" stroke="{accent}" stroke-width="2"/>
{dots_svg}{labels_svg}</svg>"#,
        accent_fill = format!("{}40", t.accent),  // ~25% opacity hex suffix
        accent = t.accent,
    )
}

pub(crate) fn svg_portrait(ch: &Character) -> String {
    use crate::substances::Substance;

    let has_cigarette = ch.active_effects.iter().any(|e| e.substance == Substance::Cigarette);

    // Slot colors
    let slot_color = |slot: &EquipSlot| match slot {
        EquipSlot::Hat       => "#4488cc",
        EquipSlot::Jacket    => "#cc44cc",
        EquipSlot::Shirt     => "#44bbcc",
        EquipSlot::Pants     => "#ccaa44",
        EquipSlot::Shoes     => "#44cc88",
        EquipSlot::Accessory => "#44cccc",
    };

    let slot_item = |slot: &EquipSlot| ch.loadout.slots.get(slot);

    let equipped_rect = |slot: &EquipSlot, x: f64, y: f64, w: f64, h: f64, rx: f64| -> String {
        let color = slot_color(slot);
        if slot_item(slot).is_some() {
            format!(
                "<rect x=\"{x:.0}\" y=\"{y:.0}\" width=\"{w:.0}\" height=\"{h:.0}\" rx=\"{rx:.0}\" fill=\"{color}\" fill-opacity=\"0.7\" stroke=\"{color}\" stroke-width=\"1.5\"/>"
            )
        } else {
            format!(
                "<rect x=\"{x:.0}\" y=\"{y:.0}\" width=\"{w:.0}\" height=\"{h:.0}\" rx=\"{rx:.0}\" fill=\"none\" stroke=\"rgba(255,255,255,0.15)\" stroke-width=\"1\" stroke-dasharray=\"4,3\"/>"
            )
        }
    };

    let item_label = |slot: &EquipSlot, x: f64, y: f64| -> String {
        if let Some(item) = slot_item(slot) {
            let name = escape_html(&item.name);
            let color = slot_color(slot);
            format!("<text x=\"{x:.0}\" y=\"{y:.0}\" fill=\"{color}\" font-size=\"7\" font-family=\"monospace\" text-anchor=\"middle\" opacity=\"0.9\">{name}</text>")
        } else {
            String::new()
        }
    };

    // Eye style based on active substances
    let (eye_left, eye_right) = {
        use crate::substances::Substance::*;
        let active: Vec<_> = ch.active_effects.iter().map(|e| &e.substance).collect();
        let mut el = format!("<circle cx=\"90\" cy=\"65\" r=\"4\" fill=\"var(--accent)\"/>");
        let mut er = format!("<circle cx=\"130\" cy=\"65\" r=\"4\" fill=\"var(--accent)\"/>");
        for sub in &active {
            let (txt_l, txt_r) = match sub {
                Lsd        => ("✿", "✿"),
                Mushrooms  => ("☯", "☯"),
                Pyrholidon => ("◈", "◈"),
                Speed      => ("◎", "◎"),
                Alcohol    => ("≈", "≈"),
                Cigarette  => ("°", "°"),
                Coffee     => ("•", "•"),
                _          => continue,
            };
            el = format!("<text x=\"90\" y=\"69\" font-size=\"10\" text-anchor=\"middle\" fill=\"var(--accent)\">{txt_l}</text>");
            er = format!("<text x=\"130\" y=\"69\" font-size=\"10\" text-anchor=\"middle\" fill=\"var(--accent)\">{txt_r}</text>");
            break;
        }
        (el, er)
    };

    // HP/Morale bars
    let hp_pct = if ch.max_health > 0 { (ch.health as f64 / ch.max_health as f64 * 150.0).min(150.0) } else { 0.0 };
    let morale_pct = if ch.max_morale > 0 { (ch.morale as f64 / ch.max_morale as f64 * 150.0).min(150.0) } else { 0.0 };

    let smoke = if has_cigarette {
        r#"<path d="M125,35 Q130,20 120,5 Q115,-10 125,-25" stroke="rgba(255,255,255,0.2)" fill="none" stroke-width="1.5">
  <animate attributeName="d" values="M125,35 Q130,20 120,5;M125,35 Q120,15 130,0;M125,35 Q130,20 120,5" dur="3s" repeatCount="indefinite"/>
  <animate attributeName="opacity" values="0.3;0.1;0.3" dur="3s" repeatCount="indefinite"/>
</path>"#.to_string()
    } else {
        String::new()
    };

    let accessory_dot = if ch.loadout.slots.get(&EquipSlot::Accessory).is_some() {
        "<circle cx=\"155\" cy=\"55\" r=\"5\" fill=\"#44cccc\" stroke=\"#44cccc\" stroke-width=\"1\"/>"
    } else {
        ""
    };

    let body_fill = "#2a2a3a";
    let hp_color = "#cc4444";
    let morale_color = "#cc44cc";

    format!(
        r#"<svg width="220" height="420" viewBox="0 0 220 420" xmlns="http://www.w3.org/2000/svg" style="background:transparent">
<!-- Hat -->
{hat}
{hat_label}
<!-- Head -->
<rect x="80" y="40" width="60" height="65" rx="10" fill="{body_fill}" stroke="rgba(255,255,255,0.2)" stroke-width="1"/>
{eye_left}
{eye_right}
<line x1="100" x2="120" y1="80" y2="80" stroke="rgba(255,255,255,0.3)" stroke-width="1.5" stroke-linecap="round"/>
{accessory_dot}
{smoke}
<!-- Neck -->
<rect x="105" y="108" width="10" height="15" rx="2" fill="{body_fill}" stroke="rgba(255,255,255,0.1)" stroke-width="1"/>
<!-- Jacket -->
{jacket}
{jacket_label}
<!-- Shirt -->
{shirt}
{shirt_label}
<!-- Arms -->
<rect x="42" y="128" width="15" height="80" rx="4" fill="{body_fill}" stroke="rgba(255,255,255,0.1)" stroke-width="1"/>
<rect x="163" y="128" width="15" height="80" rx="4" fill="{body_fill}" stroke="rgba(255,255,255,0.1)" stroke-width="1"/>
<!-- Pants -->
{pants_l}
{pants_r}
{pants_label}
<!-- Shoes -->
{shoes_l}
{shoes_r}
{shoes_label}
<!-- HP bar -->
<text x="28" y="377" fill="{hp_color}" font-size="8" font-family="monospace">HP</text>
<rect x="40" y="369" width="150" height="8" rx="2" fill="rgba(255,255,255,0.08)"/>
<rect x="40" y="369" width="{hp_pct:.0}" height="8" rx="2" fill="{hp_color}"/>
<!-- Morale bar -->
<text x="22" y="395" fill="{morale_color}" font-size="8" font-family="monospace">MP</text>
<rect x="40" y="387" width="150" height="8" rx="2" fill="rgba(255,255,255,0.08)"/>
<rect x="40" y="387" width="{morale_pct:.0}" height="8" rx="2" fill="{morale_color}"/>
</svg>"#,
        hat = equipped_rect(&EquipSlot::Hat, 75.0, 10.0, 70.0, 25.0, 5.0),
        hat_label = item_label(&EquipSlot::Hat, 110.0, 43.0),
        jacket = equipped_rect(&EquipSlot::Jacket, 65.0, 125.0, 90.0, 100.0, 6.0),
        jacket_label = item_label(&EquipSlot::Jacket, 110.0, 236.0),
        shirt = equipped_rect(&EquipSlot::Shirt, 75.0, 130.0, 70.0, 60.0, 4.0),
        shirt_label = item_label(&EquipSlot::Shirt, 110.0, 202.0),
        pants_l = equipped_rect(&EquipSlot::Pants, 65.0, 230.0, 30.0, 90.0, 3.0),
        pants_r = equipped_rect(&EquipSlot::Pants, 125.0, 230.0, 30.0, 90.0, 3.0),
        pants_label = item_label(&EquipSlot::Pants, 110.0, 329.0),
        shoes_l = equipped_rect(&EquipSlot::Shoes, 62.0, 325.0, 35.0, 20.0, 4.0),
        shoes_r = equipped_rect(&EquipSlot::Shoes, 123.0, 325.0, 35.0, 20.0, 4.0),
        shoes_label = item_label(&EquipSlot::Shoes, 110.0, 356.0),
    )
}

fn svg_activity_sparkline(ch: &Character) -> String {
    let history: Vec<_> = ch.check_history.iter().rev().take(40).collect();
    let n = history.len();

    if n == 0 {
        return format!(
            r#"<svg width="600" height="60" viewBox="0 0 600 60" xmlns="http://www.w3.org/2000/svg" style="background:transparent">
<line x1="10" y1="30" x2="590" y2="30" stroke="rgba(255,255,255,0.1)" stroke-width="1"/>
<text x="300" y="34" fill="rgba(255,255,255,0.3)" font-size="10" font-family="monospace" text-anchor="middle">no checks recorded</text>
</svg>"#
        );
    }

    let mut circles = String::new();
    // Spread checks across width 20..580
    let span = 560.0_f64;
    let step = if n > 1 { span / (n as f64 - 1.0) } else { 0.0 };

    // history is newest-first; render oldest-first (left to right)
    let ordered: Vec<_> = history.iter().rev().collect();
    for (i, record) in ordered.iter().enumerate() {
        let x = 20.0 + i as f64 * step;
        let is_crit = record.roll.0 == record.roll.1;
        let (cy, color, r) = if record.success {
            if is_crit && record.roll.0 == 6 {
                (15.0_f64, "#44cc88", 7.0_f64)
            } else {
                (20.0_f64, "#44cc88", 5.0_f64)
            }
        } else if is_crit && record.roll.0 == 1 {
            (45.0_f64, "#cc4444", 7.0_f64)
        } else {
            (40.0_f64, "#cc4444", 5.0_f64)
        };
        let stroke = if r > 5.0 { format!("stroke=\"{}\" stroke-width=\"2\"", color) } else { String::new() };
        circles.push_str(&format!(
            "<circle cx=\"{x:.1}\" cy=\"{cy}\" r=\"{r}\" fill=\"{color}\" {stroke}/>\n"
        ));
    }

    format!(
        r#"<svg width="600" height="60" viewBox="0 0 600 60" xmlns="http://www.w3.org/2000/svg" style="background:transparent">
<line x1="10" y1="30" x2="590" y2="30" stroke="rgba(255,255,255,0.1)" stroke-width="1"/>
{circles}</svg>"#
    )
}

// ── Pages ─────────────────────────────────────────────────────────────────────

fn page_character(ch: &Character) -> String {
    let stats = compute_stats(ch);
    let total = stats.total_checks;
    let pass_rate = if total > 0 {
        format!("{:.0}%", stats.successes as f64 / total as f64 * 100.0)
    } else {
        "—".to_string()
    };

    // Vitals
    let hp_pct = if ch.max_health > 0 {
        ch.health as f64 / ch.max_health as f64 * 100.0
    } else {
        0.0
    };
    let morale_pct = if ch.max_morale > 0 {
        ch.morale as f64 / ch.max_morale as f64 * 100.0
    } else {
        0.0
    };
    let xp_pct = {
        let needed = ch.xp_to_next_level();
        if needed > 0 { ch.xp as f64 / needed as f64 * 100.0 } else { 100.0 }
    };

    let vitals_html = format!(
        r#"<div class="vitals-grid">
  <div class="vital-card">
    <div class="vital-label">Health</div>
    <div class="vital-value">{hp} / {max_hp}</div>
    <div class="bar-track"><div class="bar-fill bar-health" style="width:{hp_pct:.0}%"></div></div>
  </div>
  <div class="vital-card">
    <div class="vital-label">Morale</div>
    <div class="vital-value">{morale} / {max_morale}</div>
    <div class="bar-track"><div class="bar-fill bar-morale" style="width:{morale_pct:.0}%"></div></div>
  </div>
  <div class="vital-card">
    <div class="vital-label">Experience</div>
    <div class="vital-value">{xp} / {next_xp} XP</div>
    <div class="bar-track"><div class="bar-fill bar-xp" style="width:{xp_pct:.0}%"></div></div>
  </div>
</div>"#,
        hp = ch.health,
        max_hp = ch.max_health,
        morale = ch.morale,
        max_morale = ch.max_morale,
        xp = ch.xp,
        next_xp = ch.xp_to_next_level(),
    );

    // Stat cards
    let stat_cards_html = format!(
        r#"<div class="stat-cards">
  <div class="stat-card"><div class="stat-num">{level}</div><div class="stat-label">Level</div></div>
  <div class="stat-card"><div class="stat-num">{total}</div><div class="stat-label">Total Checks</div></div>
  <div class="stat-card"><div class="stat-num">{pass_rate}</div><div class="stat-label">Pass Rate</div></div>
  <div class="stat-card"><div class="stat-num">{sp}</div><div class="stat-label">Skill Points</div></div>
</div>"#,
        level = ch.level,
        sp = ch.skill_points,
    );

    // Attributes
    let attrs = [
        (Attribute::Intellect, "intellect"),
        (Attribute::Psyche, "psyche"),
        (Attribute::Physique, "physique"),
        (Attribute::Motorics, "motorics"),
    ];

    let attr_color = |attr: Attribute| match attr {
        Attribute::Intellect => "#4488cc",
        Attribute::Psyche    => "#cc44cc",
        Attribute::Physique  => "#cc4444",
        Attribute::Motorics  => "#ccaa44",
    };

    let mut attr_html = String::from("<div class=\"attributes-grid\">\n");
    for (attr, css_class) in &attrs {
        let attr_val = ch.attributes.get(attr).copied().unwrap_or(1);
        let skills_in_attr: Vec<Skill> = Skill::all()
            .iter()
            .copied()
            .filter(|s| s.attribute() == *attr)
            .collect();

        let mut skills_html = String::new();
        for skill in &skills_in_attr {
            let base = *ch.skills.get(skill).unwrap_or(&1) as i8;
            let effective = ch.effective_skill(*skill);
            let modifier = effective - base;
            let is_sig = ch.signature_skill == Some(*skill);
            let sig_star = if is_sig { "★ " } else { "" };
            let bar_pct = (effective.max(0) as f64 / 10.0 * 100.0).min(100.0);
            let color = attr_color(*attr);

            let mod_html = if modifier > 0 {
                format!("<span class=\"skill-modifier mod-pos\">+{modifier}</span>")
            } else if modifier < 0 {
                format!("<span class=\"skill-modifier mod-neg\">{modifier}</span>")
            } else {
                "<span class=\"skill-modifier\"></span>".to_string()
            };

            skills_html.push_str(&format!(
                r#"<div class="skill-row">
  <span class="skill-name">{sig_star}{skill}</span>
  <span class="skill-level">{effective}</span>
  <div class="skill-bar-wrap"><div class="skill-bar" style="width:{bar_pct:.0}%;background:{color}"></div></div>
  {mod_html}
</div>
<div class="skill-domain">{domain}</div>
"#,
                skill = escape_html(&skill.to_string()),
                domain = skill.claude_domain(),
            ));
        }

        attr_html.push_str(&format!(
            r#"<div class="attribute-block">
  <div class="attribute-header">
    <span class="attribute-name attr-{css_class}">{attr}</span>
    <span class="attribute-value" style="color:{color}">{attr_val}</span>
  </div>
  {skills_html}
</div>
"#,
            attr = attr.to_string(),
            color = attr_color(*attr),
        ));
    }
    attr_html.push_str("</div>\n");

    let portrait = svg_portrait(ch);
    let radar = svg_radar_chart(ch);

    let body = format!(
        r#"    <div class="dashboard-hero">
      <div class="svg-container">{portrait}</div>
      <div class="svg-container">{radar}</div>
    </div>
    <h2 class="section-title">Vitals</h2>
    {vitals_html}
    <h2 class="section-title">Statistics</h2>
    {stat_cards_html}
    <h2 class="section-title">Skills</h2>
    {attr_html}"#
    );

    render_layout(ch, "Character", "Character", &body)
}

fn page_checks(ch: &Character) -> String {
    let stats = compute_stats(ch);
    let total = stats.total_checks;
    let pass_rate = if total > 0 {
        format!("{:.0}%", stats.successes as f64 / total as f64 * 100.0)
    } else {
        "—".to_string()
    };

    // Overview cards
    let overview = format!(
        r#"<div class="stat-cards">
  <div class="stat-card"><div class="stat-num">{total}</div><div class="stat-label">Total</div></div>
  <div class="stat-card"><div class="stat-num">{succ}</div><div class="stat-label">Successes</div></div>
  <div class="stat-card"><div class="stat-num">{fail}</div><div class="stat-label">Failures</div></div>
  <div class="stat-card"><div class="stat-num">{pass_rate}</div><div class="stat-label">Pass Rate</div></div>
  <div class="stat-card"><div class="stat-num">{cs}</div><div class="stat-label">Critical ✓</div></div>
  <div class="stat-card"><div class="stat-num">{cf}</div><div class="stat-label">Critical ✗</div></div>
  <div class="stat-card"><div class="stat-num">{best}</div><div class="stat-label">Best Streak</div></div>
  <div class="stat-card"><div class="stat-num">{worst}</div><div class="stat-label">Worst Streak</div></div>
</div>"#,
        succ = stats.successes,
        fail = stats.failures,
        cs = stats.critical_successes,
        cf = stats.critical_failures,
        best = stats.best_streak,
        worst = stats.worst_streak,
    );

    // Per-skill breakdown
    let mut skill_rows = String::new();
    let mut skill_stats: Vec<(Skill, usize, usize)> = Skill::all()
        .iter()
        .filter_map(|s| {
            stats.by_skill.get(s).map(|(succ, total)| (*s, *succ, *total))
        })
        .collect();
    skill_stats.sort_by(|a, b| b.2.cmp(&a.2));

    for (skill, succ, stotal) in &skill_stats {
        let rate = if *stotal > 0 {
            format!("{:.0}%", *succ as f64 / *stotal as f64 * 100.0)
        } else {
            "—".to_string()
        };
        let bar_pct = if *stotal > 0 {
            *succ as f64 / *stotal as f64 * 100.0
        } else {
            0.0
        };
        let color = match skill.attribute() {
            Attribute::Intellect => "#4488cc",
            Attribute::Psyche    => "#cc44cc",
            Attribute::Physique  => "#cc4444",
            Attribute::Motorics  => "#ccaa44",
        };
        skill_rows.push_str(&format!(
            r#"<tr>
  <td>{skill}</td>
  <td>{stotal}</td>
  <td>{succ}</td>
  <td>{rate}</td>
  <td><div class="inline-bar-wrap"><div class="inline-bar" style="width:{bar_pct:.0}%;background:{color}"></div></div></td>
</tr>"#,
            skill = escape_html(&skill.to_string()),
        ));
    }

    let skill_table = if skill_rows.is_empty() {
        "<p style=\"color:var(--muted)\">No checks recorded yet.</p>".to_string()
    } else {
        format!(
            r#"<table class="data-table">
  <thead><tr><th>Skill</th><th>Checks</th><th>Successes</th><th>Rate</th><th>Bar</th></tr></thead>
  <tbody>{skill_rows}</tbody>
</table>"#
        )
    };

    // Recent history (last 50, newest first)
    let mut history_rows = String::new();
    for record in ch.check_history.iter().rev().take(50) {
        let result_html = if record.success {
            "<span class=\"result-pass\">✓</span>"
        } else {
            "<span class=\"result-fail\">✗</span>"
        };
        let color_badge = match record.check_color {
            CheckColor::White => "<span class=\"badge badge-white\">White</span>",
            CheckColor::Red   => "<span class=\"badge badge-red\">Red</span>",
        };
        let ts = format_timestamp(&record.timestamp);
        let mod_str = if record.modifier >= 0 {
            format!("+{}", record.modifier)
        } else {
            record.modifier.to_string()
        };
        history_rows.push_str(&format!(
            r#"<tr>
  <td style="color:var(--muted);font-size:0.72rem">{ts}</td>
  <td>{skill}</td>
  <td>{d1}+{d2}</td>
  <td>{mod_str}</td>
  <td>{total}</td>
  <td>{dc}</td>
  <td>{result_html}</td>
  <td style="font-size:0.78rem;max-width:200px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{ctx}</td>
  <td>{color_badge}</td>
</tr>"#,
            skill = escape_html(&record.skill.to_string()),
            d1 = record.roll.0,
            d2 = record.roll.1,
            total = record.total,
            dc = record.difficulty,
            ctx = escape_html(&record.context),
        ));
    }

    let history_table = if history_rows.is_empty() {
        "<p style=\"color:var(--muted)\">No checks recorded yet.</p>".to_string()
    } else {
        format!(
            r#"<table class="data-table">
  <thead><tr><th>Time</th><th>Skill</th><th>Roll</th><th>Mod</th><th>Total</th><th>DC</th><th>Result</th><th>Context</th><th>Type</th></tr></thead>
  <tbody>{history_rows}</tbody>
</table>"#
        )
    };

    let sparkline = svg_activity_sparkline(ch);

    let body = format!(
        r#"    <h2 class="section-title">Overview</h2>
    {overview}
    <h2 class="section-title">Activity</h2>
    <div class="svg-container sparkline-container">{sparkline}</div>
    <h2 class="section-title">By Skill</h2>
    {skill_table}
    <h2 class="section-title">Recent History</h2>
    {history_table}"#
    );

    render_layout(ch, "Checks", "Checks", &body)
}

fn page_journal(ch: &Character) -> String {
    let mut entries_html = String::from("<div class=\"journal-list\">\n");

    if ch.journal.is_empty() {
        entries_html.push_str("<p style=\"color:var(--muted)\">No journal entries yet.</p>\n");
    } else {
        for entry in ch.journal.iter().rev() {
            let cls = entry_type_css_class(entry.entry_type);
            let tag_label = entry.entry_type.to_string();
            let ts = format_timestamp(&entry.timestamp);
            let content = escape_html(&entry.content);
            entries_html.push_str(&format!(
                r#"<div class="journal-entry {cls}">
  <div class="journal-entry-meta">
    <span class="entry-tag">{tag_label}</span>
    <span class="entry-time">{ts}</span>
  </div>
  <div class="journal-entry-content">{content}</div>
</div>
"#
            ));
        }
    }
    entries_html.push_str("</div>\n");

    let count = ch.journal.len();
    let body = format!(
        r#"    <h2 class="section-title">Journal — {count} Entries</h2>
    {entries_html}"#
    );

    render_layout(ch, "Journal", "Journal", &body)
}

fn page_thoughts(ch: &Character) -> String {
    let internalized: Vec<_> = ch
        .thoughts
        .iter()
        .filter(|t| t.phase == crate::character::ThoughtPhase::Internalized)
        .collect();
    let researching: Vec<_> = ch
        .thoughts
        .iter()
        .filter(|t| matches!(t.phase, crate::character::ThoughtPhase::Researching { .. }))
        .collect();

    fn thought_card(t: &crate::character::Thought) -> String {
        let name = escape_html(&t.name);
        let desc = escape_html(&t.description);

        let phase_html = match t.phase {
            crate::character::ThoughtPhase::Internalized => {
                "<span class=\"thought-phase phase-internalized\">✓ Internalized</span>".to_string()
            }
            crate::character::ThoughtPhase::Researching { checks_remaining } => format!(
                "<span class=\"thought-phase phase-researching\">⟳ Researching — {checks_remaining} checks remaining</span>"
            ),
        };

        let mut mod_badges = String::from("<div class=\"modifier-badges\">");
        let mut mods: Vec<_> = t.skill_modifiers.iter().collect();
        mods.sort_by_key(|(s, _)| s.to_string());
        for (skill, val) in &mods {
            let cls = if **val >= 0 { "mod-badge-pos" } else { "mod-badge-neg" };
            let sign = if **val >= 0 { "+" } else { "" };
            mod_badges.push_str(&format!(
                "<span class=\"mod-badge {cls}\">{sign}{val} {skill}</span>",
                skill = escape_html(&skill.to_string()),
            ));
        }
        mod_badges.push_str("</div>");

        format!(
            r#"<div class="thought-card">
  <div class="thought-name">{name}</div>
  <div class="thought-desc">{desc}</div>
  {phase_html}
  {mod_badges}
</div>
"#
        )
    }

    let mut intern_html = String::from("<div class=\"thoughts-grid\">\n");
    if internalized.is_empty() {
        intern_html.push_str("<p style=\"color:var(--muted)\">No internalized thoughts yet.</p>\n");
    } else {
        for t in &internalized {
            intern_html.push_str(&thought_card(t));
        }
    }
    intern_html.push_str("</div>\n");

    let mut research_html = String::from("<div class=\"thoughts-grid\">\n");
    if researching.is_empty() {
        research_html.push_str("<p style=\"color:var(--muted)\">Nothing being researched.</p>\n");
    } else {
        for t in &researching {
            research_html.push_str(&thought_card(t));
        }
    }
    research_html.push_str("</div>\n");

    let body = format!(
        r#"    <h2 class="section-title">Internalized ({ic})</h2>
    {intern_html}
    <h2 class="section-title">Researching ({rc})</h2>
    {research_html}"#,
        ic = internalized.len(),
        rc = researching.len(),
    );

    render_layout(ch, "Thoughts", "Thoughts", &body)
}

fn page_equipment(ch: &Character) -> String {
    let slots = [
        EquipSlot::Hat,
        EquipSlot::Jacket,
        EquipSlot::Shirt,
        EquipSlot::Pants,
        EquipSlot::Shoes,
        EquipSlot::Accessory,
    ];

    let mut equip_html = String::from("<div class=\"equipment-grid\">\n");
    for slot in &slots {
        let slot_label = slot.to_string();
        if let Some(item) = ch.loadout.slots.get(slot) {
            let name = escape_html(&item.name);
            let desc = escape_html(&item.description);
            let mut mod_badges = String::from("<div class=\"modifier-badges\">");
            let mut mods: Vec<_> = item.skill_modifiers.iter().collect();
            mods.sort_by_key(|(s, _)| s.to_string());
            for (skill, val) in &mods {
                let cls = if **val >= 0 { "mod-badge-pos" } else { "mod-badge-neg" };
                let sign = if **val >= 0 { "+" } else { "" };
                mod_badges.push_str(&format!(
                    "<span class=\"mod-badge {cls}\">{sign}{val} {skill}</span>",
                    skill = escape_html(&skill.to_string()),
                ));
            }
            mod_badges.push_str("</div>");
            equip_html.push_str(&format!(
                r#"<div class="equip-slot-card">
  <div class="equip-slot-name">{slot_label}</div>
  <div class="equip-item-name">{name}</div>
  <div class="equip-item-desc">{desc}</div>
  {mod_badges}
</div>
"#
            ));
        } else {
            equip_html.push_str(&format!(
                r#"<div class="equip-slot-card">
  <div class="equip-slot-name">{slot_label}</div>
  <div class="equip-empty">Empty</div>
</div>
"#
            ));
        }
    }
    equip_html.push_str("</div>\n");

    // Inventory
    let mut inv_html = String::from("<div class=\"inventory-grid\">\n");
    if ch.inventory.is_empty() {
        inv_html.push_str("<p style=\"color:var(--muted)\">Inventory is empty.</p>\n");
    } else {
        let mut inv_items: Vec<_> = ch.inventory.iter().collect();
        inv_items.sort_by_key(|(s, _)| s.to_string());
        for (substance, count) in inv_items {
            let info = substance.info();
            inv_html.push_str(&format!(
                r#"<div class="inventory-item">
  <div class="inventory-item-name">{name}</div>
  <div class="inventory-count">x{count}</div>
  <div class="inventory-desc">{desc}</div>
</div>
"#,
                name = escape_html(info.name),
                desc = escape_html(info.description),
            ));
        }
    }
    inv_html.push_str("</div>\n");

    // Active effects
    let mut effects_html = String::from("<div class=\"effects-list\">\n");
    if ch.active_effects.is_empty() {
        effects_html.push_str("<p style=\"color:var(--muted)\">No active effects.</p>\n");
    } else {
        for effect in &ch.active_effects {
            let name = effect.substance.to_string();
            let remaining = effect.checks_remaining;
            let mut mod_badges = String::from("<div class=\"modifier-badges\">");
            let mut mods: Vec<_> = effect.skill_modifiers.iter().collect();
            mods.sort_by_key(|(s, _)| s.to_string());
            for (skill, val) in &mods {
                let cls = if **val >= 0 { "mod-badge-pos" } else { "mod-badge-neg" };
                let sign = if **val >= 0 { "+" } else { "" };
                mod_badges.push_str(&format!(
                    "<span class=\"mod-badge {cls}\">{sign}{val} {skill}</span>",
                    skill = escape_html(&skill.to_string()),
                ));
            }
            mod_badges.push_str("</div>");
            effects_html.push_str(&format!(
                r#"<div class="effect-card">
  <div>
    <div class="effect-name">{name}</div>
    {mod_badges}
  </div>
  <div class="effect-remaining">{remaining} checks remaining</div>
</div>
"#
            ));
        }
    }
    effects_html.push_str("</div>\n");

    let body = format!(
        r#"    <h2 class="section-title">Loadout</h2>
    {equip_html}
    <h2 class="section-title">Inventory</h2>
    {inv_html}
    <h2 class="section-title">Active Effects</h2>
    {effects_html}"#
    );

    render_layout(ch, "Equipment", "Equipment", &body)
}

fn page_leaderboard() -> String {
    let profiles = crate::character::list_profiles();

    struct Entry {
        name: String,
        archetype: String,
        level: u32,
        total_checks: usize,
        pass_rate: f64,
        best_streak: usize,
        total_xp: u32,
        critical_successes: usize,
    }

    let mut entries: Vec<Entry> = profiles
        .iter()
        .filter_map(|name| {
            let ch = crate::character::Character::load(name).ok()?;
            let stats = compute_stats(&ch);
            let pass_rate = if stats.total_checks > 0 {
                stats.successes as f64 / stats.total_checks as f64 * 100.0
            } else {
                0.0
            };
            Some(Entry {
                name: ch.name,
                archetype: ch.archetype,
                level: ch.level,
                total_checks: stats.total_checks,
                pass_rate,
                best_streak: stats.best_streak,
                total_xp: stats.total_xp,
                critical_successes: stats.critical_successes,
            })
        })
        .collect();

    // Sort by level descending, then by XP descending
    entries.sort_by(|a, b| b.level.cmp(&a.level).then(b.total_xp.cmp(&a.total_xp)));

    let mut rows = String::new();
    for (i, e) in entries.iter().enumerate() {
        let rank = i + 1;
        let rate_str = if e.total_checks > 0 {
            format!("{:.0}%", e.pass_rate)
        } else {
            "—".to_string()
        };

        // Medal emoji for top 3
        let medal = match rank {
            1 => "🥇 ",
            2 => "🥈 ",
            3 => "🥉 ",
            _ => "",
        };

        rows.push_str(&format!(
            r#"<tr>
  <td style="text-align:center;font-weight:bold">{medal}{rank}</td>
  <td style="color:var(--accent);font-weight:bold">{name}</td>
  <td>{archetype}</td>
  <td style="text-align:center;font-weight:bold">{level}</td>
  <td style="text-align:center">{checks}</td>
  <td style="text-align:center">{rate}</td>
  <td style="text-align:center">{streak}</td>
  <td style="text-align:center">{crits}</td>
  <td style="text-align:right">{xp}</td>
</tr>"#,
            name = escape_html(&e.name),
            archetype = escape_html(&e.archetype),
            level = e.level,
            checks = e.total_checks,
            rate = rate_str,
            streak = e.best_streak,
            crits = e.critical_successes,
            xp = e.total_xp,
        ));
    }

    let count = entries.len();

    let table = if rows.is_empty() {
        "<p style=\"color:var(--muted)\">No profiles found.</p>".to_string()
    } else {
        format!(
            r#"<table class="data-table">
  <thead><tr>
    <th style="text-align:center">Rank</th>
    <th>Name</th>
    <th>Archetype</th>
    <th style="text-align:center">Level</th>
    <th style="text-align:center">Checks</th>
    <th style="text-align:center">Pass Rate</th>
    <th style="text-align:center">Best Streak</th>
    <th style="text-align:center">Criticals</th>
    <th style="text-align:right">XP</th>
  </tr></thead>
  <tbody>{rows}</tbody>
</table>"#
        )
    };

    let body = format!(
        r#"    <h2 class="section-title">Leaderboard — {count} Detectives</h2>
    {table}"#
    );

    // Need a character for layout theming — load active or use defaults
    match crate::character::Character::load_active() {
        Ok(ch) => render_layout(&ch, "Leaderboard", "Leaderboard", &body),
        Err(_) => {
            // Minimal fallback if no active character
            format!(
                r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>Leaderboard</title></head>
<body style="background:#1a1a2e;color:#d4a574;font-family:monospace;padding:2rem">
{body}
</body></html>"#
            )
        }
    }
}

fn page_not_found() -> String {
    let body = r#"<div class="not-found">
  <h2>404</h2>
  <p>The path you seek does not exist in this precinct.</p>
  <a href="/">Return to character sheet</a>
</div>"#;

    // Minimal layout without character
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>404 — Inland Empire</title>
  <style>
    body {{ background: #1a1a2e; color: #d4a574; font-family: 'Courier New', monospace;
           display: flex; align-items: center; justify-content: center; min-height: 100vh; }}
    .not-found {{ text-align: center; }}
    .not-found h2 {{ font-size: 5rem; color: #f0c070; margin-bottom: 1rem; }}
    .not-found p {{ color: #8a7060; margin-bottom: 1.5rem; }}
    a {{ color: #f0c070; }}
  </style>
</head>
<body>
  {body}
</body>
</html>"#
    )
}
