'use strict';

const { invoke } = window.__TAURI__.core;

// ── État ─────────────────────────────────────────────────────────────────────
let activeId      = null;
let timerStart    = null;
let timerIntvl    = null;
let cadPollIntvl  = null;
let clickCount    = 0;
let editId        = null;
let activeCadFile = null;

// ── Horloge ───────────────────────────────────────────────────────────────────
function tickClock() {
  document.getElementById('clock').textContent =
    new Date().toTimeString().slice(0, 8);
}
setInterval(tickClock, 1000);
tickClock();

// ── Compteur de clics souris ──────────────────────────────────────────────────
document.addEventListener('mousedown', () => {
  if (!activeId) return;
  if (event.target.closest('#btn-stop')) return;
  clickCount++;
  document.getElementById('click-count').textContent = clickCount;
});

// ── Init ──────────────────────────────────────────────────────────────────────
async function init() {
  await Promise.all([loadPeople(), loadJournal(), loadStats()]);
}

// ── Personnes ─────────────────────────────────────────────────────────────────
async function loadPeople() {
  const people = await invoke('get_people').catch(e => { showToast(e, 'error'); return []; });
  renderPeople(people);
}

function initials(name) {
  return name.trim().split(/\s+/).map(w => w[0]).join('').toUpperCase().slice(0, 2);
}

function renderPeople(people) {
  const list = document.getElementById('people-list');
  list.innerHTML = '';

  if (people.length === 0) {
    const msg = document.createElement('div');
    msg.className = 'people-empty';
    msg.textContent = 'Ajoutez vos collaborateurs ci-dessous.';
    list.appendChild(msg);
    return;
  }

  people.forEach(p => {
    const btn = document.createElement('button');
    btn.className = 'person-btn' + (activeId && btn.dataset.id == activeId ? ' p-active' : '');
    btn.dataset.id   = p.id;
    btn.dataset.name = p.name;
    btn.dataset.role = p.role;
    btn.disabled = !!activeId;

    btn.innerHTML = `
      <div class="person-avatar">${esc(initials(p.name))}</div>
      <div class="person-info">
        <div class="person-name">${esc(p.name)}</div>
        ${p.role ? `<div class="person-role">${esc(p.role)}</div>` : ''}
      </div>
      <span class="person-count" style="display:none" data-pid="${p.id}"></span>
      <div class="person-actions" onclick="event.stopPropagation()">
        <button class="btn-person-action"     title="Renommer" onclick="openEdit(${p.id},'${esc(p.name).replace(/'/g,"\\'")}','${esc(p.role).replace(/'/g,"\\'")}')">✎</button>
        <button class="btn-person-action del" title="Supprimer" onclick="deletePerson(${p.id})">×</button>
      </div>
    `;
    btn.addEventListener('click', () => startInterruption(p));
    list.appendChild(btn);
  });

  // Injecter les compteurs du jour depuis le journal courant
  refreshPersonCounts();
}

function refreshPersonCounts() {
  // Récupère les comptages depuis le journal déjà affiché
  const counts = {};
  document.querySelectorAll('.log-entry[data-pid]').forEach(row => {
    const pid = row.dataset.pid;
    counts[pid] = (counts[pid] || 0) + 1;
  });
  document.querySelectorAll('.person-count[data-pid]').forEach(el => {
    const n = counts[el.dataset.pid] || 0;
    el.textContent = n + '×';
    el.style.display = n > 0 ? '' : 'none';
  });
}

// ── Ajouter une personne ──────────────────────────────────────────────────────
document.getElementById('btn-add-person').addEventListener('click', addPerson);
['input-name', 'input-role'].forEach(id =>
  document.getElementById(id).addEventListener('keydown', e => { if (e.key === 'Enter') addPerson(); })
);

async function addPerson() {
  const name = document.getElementById('input-name').value.trim();
  const role = document.getElementById('input-role').value.trim();
  if (!name) { document.getElementById('input-name').focus(); return; }

  await invoke('add_person', { name, role }).catch(e => showToast(e, 'error'));
  document.getElementById('input-name').value = '';
  document.getElementById('input-role').value = '';
  document.getElementById('input-name').focus();
  await loadPeople();
}

// ── Supprimer ─────────────────────────────────────────────────────────────────
async function deletePerson(id) {
  if (!confirm('Supprimer cette personne ? L\'historique reste dans le journal.')) return;
  await invoke('delete_person', { id }).catch(e => showToast(e, 'error'));
  await loadPeople();
}

// ── Modal renommer ────────────────────────────────────────────────────────────
function openEdit(id, name, role) {
  editId = id;
  document.getElementById('modal-name').value = name;
  document.getElementById('modal-role').value = role;
  document.getElementById('modal-overlay').classList.remove('hidden');
  document.getElementById('modal-name').focus();
}

document.getElementById('modal-cancel').addEventListener('click', closeModal);
document.getElementById('modal-overlay').addEventListener('click', e => {
  if (e.target === document.getElementById('modal-overlay')) closeModal();
});
document.getElementById('modal-confirm').addEventListener('click', confirmEdit);
document.addEventListener('keydown', e => {
  if (e.key === 'Escape') closeModal();
  if (e.key === 'Enter' && !document.getElementById('modal-overlay').classList.contains('hidden')) confirmEdit();
});

function closeModal() {
  document.getElementById('modal-overlay').classList.add('hidden');
  editId = null;
}

async function confirmEdit() {
  const name = document.getElementById('modal-name').value.trim();
  const role = document.getElementById('modal-role').value.trim();
  if (!name || editId === null) return;
  await invoke('update_person', { id: editId, name, role }).catch(e => showToast(e, 'error'));
  closeModal();
  await loadPeople();
}

// ── Démarrer interruption ─────────────────────────────────────────────────────
async function startInterruption(person) {
  if (activeId) return;

  const id = await invoke('start_interruption', {
    personId: person.id, personName: person.name,
  }).catch(e => { showToast(e, 'error'); return null; });
  if (id === null) return;

  activeId   = id;
  timerStart = Date.now();
  clickCount = 0;

  document.getElementById('active-name').textContent  = person.name;
  document.getElementById('active-role').textContent  = person.role || '';
  document.getElementById('active-start').textContent = new Date().toTimeString().slice(0, 5);
  document.getElementById('click-count').textContent  = '0';
  document.getElementById('timer').textContent        = '00:00';
  document.getElementById('timer').className          = 'timer-display';

  document.getElementById('idle-msg').style.display = 'none';
  document.getElementById('active-zone').classList.add('visible');

  // Marquer le bouton actif
  document.querySelectorAll('.person-btn').forEach(b => {
    b.disabled = true;
    if (parseInt(b.dataset.id) === person.id) b.classList.add('p-active');
  });

  timerIntvl = setInterval(tickTimer, 500);

  // Détection fichier CAD — immédiate puis toutes les 5s
  activeCadFile = null;
  updateCadFile();
  cadPollIntvl = setInterval(updateCadFile, 5000);
}

async function updateCadFile() {
  const file = await invoke('get_active_cad_file').catch(() => null);
  activeCadFile = file ?? null;
  const el = document.getElementById('active-file');
  if (el) el.textContent = activeCadFile ?? 'non détecté';
}

function tickTimer() {
  if (!timerStart) return;
  const elapsed = Math.floor((Date.now() - timerStart) / 1000);
  const m = Math.floor(elapsed / 60).toString().padStart(2, '0');
  const s = (elapsed % 60).toString().padStart(2, '0');
  const el = document.getElementById('timer');
  el.textContent = `${m}:${s}`;
  el.className = 'timer-display' + (elapsed > 300 ? ' long' : '');
}

// ── Arrêter interruption ──────────────────────────────────────────────────────
document.getElementById('btn-stop').addEventListener('click', stopInterruption);

async function stopInterruption() {
  if (!activeId) return;
  clearInterval(timerIntvl);
  clearInterval(cadPollIntvl);
  timerIntvl = null;
  cadPollIntvl = null;

  await invoke('stop_interruption', {
    id: activeId, mouseClicks: clickCount, activeWindow: activeCadFile,
  }).catch(e => showToast(e, 'error'));

  activeId   = null;
  timerStart = null;

  document.getElementById('idle-msg').style.display = '';
  document.getElementById('active-zone').classList.remove('visible');

  document.querySelectorAll('.person-btn').forEach(b => {
    b.disabled = false;
    b.classList.remove('p-active');
  });

  await Promise.all([loadJournal(), loadStats()]);
  refreshPersonCounts();
}

// ── Journal ───────────────────────────────────────────────────────────────────
async function loadJournal() {
  const items = await invoke('get_today_interruptions').catch(e => { showToast(e, 'error'); return []; });

  const list = document.getElementById('log-list');
  const done = items.filter(i => i.end_time !== null);

  list.innerHTML = '';

  if (done.length === 0) {
    const msg = document.createElement('div');
    msg.className = 'log-empty';
    msg.textContent = 'Aucune session enregistrée';
    list.appendChild(msg);
    return;
  }

  // En-tête colonnes
  const hdr = document.createElement('div');
  hdr.className = 'log-col-header';
  hdr.innerHTML = '<span>Heure</span><span>Personne</span><span>DWG</span><span>Durée</span><span>Clics</span><span></span><span></span>';
  list.appendChild(hdr);

  for (const item of done) {
    const row = document.createElement('div');
    row.className = 'log-entry';
    row.dataset.pid = item.person_id ?? '';
    const heure   = formatTime(item.start_time);
    const duree   = item.duration_seconds !== null ? formatDur(item.duration_seconds) : '—';
    row.innerHTML = `
      <span class="log-time">${heure}</span>
      <span class="log-person">${esc(item.person_name)}</span>
      <span class="log-role">${esc(item.active_window ?? '—')}</span>
      <span class="log-duration">${duree}</span>
      <span class="log-clicks">${item.mouse_clicks ?? '—'}</span>
      <span class="log-file">—</span>
      <button class="log-delete" title="Supprimer" data-id="${item.id}">×</button>
    `;
    list.appendChild(row);
  }

  list.querySelectorAll('.log-delete').forEach(btn =>
    btn.addEventListener('click', async e => {
      // Suppression locale uniquement (pas de commande Tauri dédiée)
      e.target.closest('.log-entry').remove();
      renderStats();
      refreshPersonCounts();
    })
  );

  refreshPersonCounts();
}

// ── Stats ─────────────────────────────────────────────────────────────────────
async function loadStats() {
  const s = await invoke('get_stats_today').catch(() => null);
  if (!s) return;

  document.getElementById('stat-count').textContent = s.total_interruptions;

  const m   = Math.floor(s.total_seconds / 60);
  const sec = s.total_seconds % 60;
  const timeStr = m > 0 ? `${m}m${sec.toString().padStart(2,'0')}s` : `${sec}s`;

  document.getElementById('stat-time').textContent = timeStr;
  document.getElementById('footer-total-time').textContent = m + ' min';
  document.getElementById('footer-sessions').textContent   = s.total_interruptions;
  document.getElementById('stat-top').textContent =
    s.top_interruptor_name ? `${s.top_interruptor_name} (${s.top_interruptor_count}×)` : '—';
}

// Recalcul rapide depuis le DOM (après suppression locale)
function renderStats() {
  const entries = document.querySelectorAll('.log-entry');
  document.getElementById('stat-count').textContent       = entries.length;
  document.getElementById('footer-sessions').textContent  = entries.length;
}

// ── Export CSV ────────────────────────────────────────────────────────────────
document.getElementById('btn-export').addEventListener('click', async () => {
  const path = await invoke('export_csv').catch(e => { showToast(e, 'error'); return null; });
  if (path) showToast('CSV exporté :\n' + path, 'success');
});

// ── Utilitaires ───────────────────────────────────────────────────────────────
function formatTime(iso) {
  try { return new Date(iso).toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' }); }
  catch { return iso; }
}

function formatDur(s) {
  const m = Math.floor(s / 60), r = s % 60;
  if (m === 0) return `${r}s`;
  return r > 0 ? `${m}m${r}s` : `${m}m`;
}

function esc(str) {
  return String(str ?? '')
    .replace(/&/g,'&amp;').replace(/</g,'&lt;')
    .replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

let toastTimer = null;
function showToast(msg, type = '') {
  const el = document.getElementById('toast');
  el.textContent = msg;
  el.className = 'toast' + (type ? ' ' + type : '');
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => el.classList.add('hidden'), 4000);
}

// ── Lancement ─────────────────────────────────────────────────────────────────
init();
