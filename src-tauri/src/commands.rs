use chrono::Local;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Manager, State};

type Db<'a> = State<'a, Mutex<Connection>>;

// ── Structures ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct Person {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct Interruption {
    pub id: i64,
    pub person_id: Option<i64>,
    pub person_name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_seconds: Option<i64>,
    pub mouse_clicks: Option<i64>,
    pub active_window: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct Stats {
    pub total_interruptions: i64,
    pub total_seconds: i64,
    pub top_interruptor_name: Option<String>,
    pub top_interruptor_count: i64,
}

// ── Personnes ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_people(db: Db<'_>) -> Result<Vec<Person>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, role, created_at FROM people ORDER BY name ASC")
        .map_err(|e| e.to_string())?;

    let people = stmt
        .query_map([], |row| {
            Ok(Person {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(people)
}

#[tauri::command]
pub fn add_person(db: Db<'_>, name: String, role: String) -> Result<Person, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let now = Local::now().to_rfc3339();

    conn.execute(
        "INSERT INTO people (name, role, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, role, now],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    Ok(Person { id, name, role, created_at: now })
}

#[tauri::command]
pub fn update_person(db: Db<'_>, id: i64, name: String, role: String) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE people SET name = ?1, role = ?2 WHERE id = ?3",
        rusqlite::params![name, role, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_person(db: Db<'_>, id: i64) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM people WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Interruptions ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn start_interruption(
    db: Db<'_>,
    person_id: i64,
    person_name: String,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let now = Local::now().to_rfc3339();

    conn.execute(
        "INSERT INTO interruptions (person_id, person_name, start_time, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![person_id, person_name, now, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn stop_interruption(
    db: Db<'_>,
    id: i64,
    mouse_clicks: i64,
    active_window: Option<String>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let now = Local::now();
    let now_str = now.to_rfc3339();

    let start_time: String = conn
        .query_row(
            "SELECT start_time FROM interruptions WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let start = chrono::DateTime::parse_from_rfc3339(&start_time)
        .map_err(|e| e.to_string())?;
    let duration_seconds = (now.signed_duration_since(start)).num_seconds();

    conn.execute(
        "UPDATE interruptions
         SET end_time = ?1, duration_seconds = ?2, mouse_clicks = ?3, active_window = ?4
         WHERE id = ?5",
        rusqlite::params![now_str, duration_seconds, mouse_clicks, active_window, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ── Journal & Stats ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_today_interruptions(db: Db<'_>) -> Result<Vec<Interruption>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut stmt = conn
        .prepare(
            "SELECT id, person_id, person_name, start_time, end_time,
                    duration_seconds, mouse_clicks, active_window, notes, created_at
             FROM interruptions
             WHERE substr(created_at, 1, 10) = ?1
             ORDER BY start_time DESC",
        )
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map(rusqlite::params![today], |row| {
            Ok(Interruption {
                id: row.get(0)?,
                person_id: row.get(1)?,
                person_name: row.get(2)?,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                duration_seconds: row.get(5)?,
                mouse_clicks: row.get(6)?,
                active_window: row.get(7)?,
                notes: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(items)
}

#[tauri::command]
pub fn get_stats_today(db: Db<'_>) -> Result<Stats, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let today = Local::now().format("%Y-%m-%d").to_string();

    let (total_interruptions, total_seconds): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(duration_seconds), 0)
             FROM interruptions
             WHERE substr(created_at, 1, 10) = ?1 AND end_time IS NOT NULL",
            rusqlite::params![today],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let top = conn
        .query_row(
            "SELECT person_name, COUNT(*) AS cnt
             FROM interruptions
             WHERE substr(created_at, 1, 10) = ?1 AND end_time IS NOT NULL
             GROUP BY person_name
             ORDER BY cnt DESC
             LIMIT 1",
            rusqlite::params![today],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .ok();

    Ok(Stats {
        total_interruptions,
        total_seconds,
        top_interruptor_name: top.as_ref().map(|(n, _)| n.clone()),
        top_interruptor_count: top.map(|(_, c)| c).unwrap_or(0),
    })
}

// ── Export CSV ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn export_csv(app: tauri::AppHandle, db: Db<'_>) -> Result<String, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.person_name, p.role, i.start_time, i.end_time,
                    i.duration_seconds, i.mouse_clicks, i.active_window, i.created_at
             FROM interruptions i
             LEFT JOIN people p ON p.id = i.person_id
             ORDER BY i.start_time ASC",
        )
        .map_err(|e| e.to_string())?;

    let mut csv =
        "id,personne,role,debut,fin,duree_secondes,clics_souris,fenetre_active,date\n"
            .to_string();

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (id, name, role, start, end, dur, clicks, window, created) =
            row.map_err(|e| e.to_string())?;
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            id,
            name,
            role.unwrap_or_default(),
            start,
            end.unwrap_or_default(),
            dur.map(|d| d.to_string()).unwrap_or_default(),
            clicks.map(|c| c.to_string()).unwrap_or_default(),
            window.unwrap_or_default(),
            created,
        ));
    }

    // Écrire dans Documents/InterruptLog/
    let docs_dir = app.path().document_dir().map_err(|e: tauri::Error| e.to_string())?;
    let dest_dir = docs_dir.join("InterruptLog");
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    let filename = format!(
        "export_{}.csv",
        Local::now().format("%Y%m%d_%H%M%S")
    );
    let path = dest_dir.join(&filename);
    std::fs::write(&path, &csv).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().into_owned())
}

// ── Détection fichier CAD actif ──────────────────────────────────────────────

#[tauri::command]
pub fn get_active_cad_file() -> Option<String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    // Récupère les titres de fenêtres des process ayant un .dwg ou .dxf
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; \
             Get-Process | \
             Where-Object { $_.MainWindowTitle -match '\\.(dwg|dxf)' } | \
             Select-Object -First 1 -ExpandProperty MainWindowTitle",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if title.is_empty() {
        return None;
    }

    // Extraire le nom de fichier du titre
    // Formats connus :
    //   "dessin.dwg - ZWCAD 2024"
    //   "C:\projets\dessin.dwg [Lecture seule] - ZWCAD"
    //   "Autodesk AutoCAD - [dessin.dwg]"
    let lower = title.to_lowercase();
    let ext_pos = lower.find(".dwg").or_else(|| lower.find(".dxf"))?;

    // Cherche le début du nom de fichier en remontant
    let before = &title[..ext_pos];
    let start = before
        .rfind(|c: char| matches!(c, '\\' | '/' | '[' | ' '))
        .map(|i| i + 1)
        .unwrap_or(0);

    let filename = title[start..ext_pos + 4].to_string();
    if filename.is_empty() {
        return None;
    }
    Some(filename)
}
