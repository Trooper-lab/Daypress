//! WordPress Meta Boxes (Collapse, Move Up/Down/Column, Drag) & Screen Options (`scripts/screen_options.rs`).

pub const JS_SCREEN_OPTIONS: &str = r###"
            let editorScreenState = {
                columns: 2,
                hidden: {},
                collapsed: {},
                mainOrder: [],
                sideOrder: []
            };
            let draggedMetaBoxEl = null;

            function getEditorScreenStorageKey() {
                return 'daypress_editor_screen_prefs_v1';
            }

            function saveEditorScreenState() {
                const mainCol = document.getElementById('editorMainColumn');
                const sideCol = document.getElementById('editorSideColumn');
                if (mainCol) {
                    editorScreenState.mainOrder = Array.from(mainCol.querySelectorAll(':scope > .day-card[data-metabox-id]'))
                        .map(el => el.id || el.getAttribute('data-metabox-id'));
                }
                if (sideCol) {
                    editorScreenState.sideOrder = Array.from(sideCol.querySelectorAll(':scope > .day-card[data-metabox-id]'))
                        .map(el => el.id || el.getAttribute('data-metabox-id'));
                }
                try {
                    localStorage.setItem(getEditorScreenStorageKey(), JSON.stringify(editorScreenState));
                } catch (_) {}
                fetch('/wp-json/daypress/v1/user-preferences', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ screens: { post_editor: editorScreenState } })
                }).catch(() => {});
            }

            function applyEditorScreenStateToDom() {
                const grid = document.getElementById('editorColumnsGrid');
                const mainCol = document.getElementById('editorMainColumn');
                const sideCol = document.getElementById('editorSideColumn');
                const cols = editorScreenState.columns === 1 ? 1 : 2;
                if (grid) {
                    grid.style.gridTemplateColumns = cols === 1 ? '1fr' : '3fr 1fr';
                }
                const r1 = document.getElementById('editorCols1');
                const r2 = document.getElementById('editorCols2');
                if (r1) r1.checked = cols === 1;
                if (r2) r2.checked = cols === 2;

                const placeCards = (colEl, orderArr) => {
                    if (!colEl || !Array.isArray(orderArr)) return;
                    orderArr.forEach(ident => {
                        const card = document.getElementById(ident) || document.querySelector(`.day-card[data-metabox-id="${ident}"]`);
                        if (card && card.hasAttribute('data-metabox-id')) {
                            colEl.appendChild(card);
                        }
                    });
                };
                placeCards(mainCol, editorScreenState.mainOrder);
                placeCards(sideCol, editorScreenState.sideOrder);

                document.querySelectorAll('.day-card[data-metabox-id]').forEach((card, idx) => {
                    const boxId = card.getAttribute('data-metabox-id');
                    const cardKey = card.id || (boxId + '_' + idx);
                    const isHidden = !!editorScreenState.hidden[boxId];
                    if (isHidden) {
                        card.dataset.dpScreenHidden = 'true';
                        card.style.display = 'none';
                    } else if (card.dataset.dpScreenHidden === 'true') {
                        delete card.dataset.dpScreenHidden;
                        card.style.display = '';
                    }
                    const isCollapsed = !!editorScreenState.collapsed[cardKey] || !!editorScreenState.collapsed[boxId];
                    setMetaBoxCardCollapsed(card, isCollapsed, false);
                });

                document.querySelectorAll('.dp-screen-opt-cb[data-box-id]').forEach(cb => {
                    const boxId = cb.getAttribute('data-box-id');
                    cb.checked = !editorScreenState.hidden[boxId];
                });
            }

            function setMetaBoxCardCollapsed(card, collapsed, persist) {
                if (!card) return;
                const body = card.querySelector('.day-card-body');
                const header = card.querySelector('.day-card-header');
                const toggleBtn = card.querySelector('.dp-metabox-collapse-btn');
                const boxId = card.getAttribute('data-metabox-id');
                const cardKey = card.id || boxId;
                if (body) body.style.display = collapsed ? 'none' : '';
                if (header) {
                    header.style.borderBottom = collapsed ? 'none' : '';
                    header.style.marginBottom = collapsed ? '0' : '';
                    header.style.paddingBottom = collapsed ? '0' : '';
                }
                if (toggleBtn) {
                    toggleBtn.textContent = collapsed ? '▸' : '▾';
                    toggleBtn.title = collapsed ? 'Expand Meta Box' : 'Collapse Meta Box';
                    toggleBtn.setAttribute('aria-expanded', collapsed ? 'false' : 'true');
                }
                card.classList.toggle('dp-metabox-collapsed', !!collapsed);
                if (persist) {
                    editorScreenState.collapsed[cardKey] = !!collapsed;
                    editorScreenState.collapsed[boxId] = !!collapsed;
                    saveEditorScreenState();
                }
            }

            function moveEditorMetaBox(card, dir) {
                if (!card || !card.parentElement) return;
                const parent = card.parentElement;
                const cards = Array.from(parent.querySelectorAll(':scope > .day-card[data-metabox-id]'));
                const idx = cards.indexOf(card);
                if (idx === -1) return;
                const targetIdx = idx + dir;
                if (targetIdx < 0 || targetIdx >= cards.length) return;
                const targetCard = cards[targetIdx];
                if (dir < 0) parent.insertBefore(card, targetCard);
                else parent.insertBefore(targetCard, card);
                saveEditorScreenState();
            }

            function moveEditorMetaBoxColumn(card) {
                if (!card) return;
                const mainCol = document.getElementById('editorMainColumn');
                const sideCol = document.getElementById('editorSideColumn');
                if (!mainCol || !sideCol) return;
                if (card.parentElement === sideCol) {
                    mainCol.appendChild(card);
                    showToast('Moved meta box to Main Column');
                } else {
                    sideCol.appendChild(card);
                    showToast('Moved meta box to Sidebar Column');
                }
                saveEditorScreenState();
            }

            function initEditorMetaBoxesAndScreenOptions() {
                document.querySelectorAll('.day-card[data-metabox-id]').forEach((card, idx) => {
                    if (!card.id) card.id = 'dp-metabox-' + card.getAttribute('data-metabox-id') + '-' + idx;
                    const header = card.querySelector('.day-card-header');
                    if (!header || header.querySelector('.dp-metabox-controls')) return;
                    header.style.display = 'flex';
                    header.style.justifyContent = 'space-between';
                    header.style.alignItems = 'center';
                    header.style.cursor = 'grab';
                    header.addEventListener('dblclick', (ev) => {
                        if (ev.target.closest('button, a, input, select')) return;
                        const isCol = card.classList.contains('dp-metabox-collapsed');
                        setMetaBoxCardCollapsed(card, !isCol, true);
                    });

                    card.setAttribute('draggable', 'true');
                    card.addEventListener('dragstart', (ev) => {
                        if (ev.target.tagName === 'INPUT' || ev.target.tagName === 'TEXTAREA' || ev.target.tagName === 'SELECT') {
                            ev.preventDefault();
                            return;
                        }
                        draggedMetaBoxEl = card;
                        card.style.opacity = '0.6';
                    });
                    card.addEventListener('dragend', () => {
                        card.style.opacity = '';
                        draggedMetaBoxEl = null;
                    });
                    card.addEventListener('dragover', (ev) => {
                        if (!draggedMetaBoxEl || draggedMetaBoxEl === card) return;
                        ev.preventDefault();
                    });
                    card.addEventListener('drop', (ev) => {
                        if (!draggedMetaBoxEl || draggedMetaBoxEl === card) return;
                        ev.preventDefault();
                        const parent = card.parentElement;
                        if (parent) {
                            parent.insertBefore(draggedMetaBoxEl, card);
                            saveEditorScreenState();
                        }
                    });

                    const controls = document.createElement('div');
                    controls.className = 'dp-metabox-controls';
                    controls.style.cssText = 'display: inline-flex; align-items: center; gap: 3px; margin-left: auto; padding-left: 8px;';
                    controls.innerHTML = `
                        <button type="button" class="btn btn-secondary dp-metabox-up-btn" style="padding: 1px 5px; font-size: 11px; height: 22px; min-width: 22px;" title="Move Meta Box Up">↑</button>
                        <button type="button" class="btn btn-secondary dp-metabox-down-btn" style="padding: 1px 5px; font-size: 11px; height: 22px; min-width: 22px;" title="Move Meta Box Down">↓</button>
                        <button type="button" class="btn btn-secondary dp-metabox-col-btn" style="padding: 1px 5px; font-size: 11px; height: 22px; min-width: 22px;" title="Move Between Main &amp; Sidebar Column">⇄</button>
                        <button type="button" class="btn btn-secondary dp-metabox-collapse-btn" aria-expanded="true" style="padding: 1px 6px; font-size: 11px; height: 22px; min-width: 22px;" title="Collapse / Expand Meta Box">▾</button>
                    `;
                    controls.querySelector('.dp-metabox-up-btn').addEventListener('click', (e) => { e.stopPropagation(); moveEditorMetaBox(card, -1); });
                    controls.querySelector('.dp-metabox-down-btn').addEventListener('click', (e) => { e.stopPropagation(); moveEditorMetaBox(card, 1); });
                    controls.querySelector('.dp-metabox-col-btn').addEventListener('click', (e) => { e.stopPropagation(); moveEditorMetaBoxColumn(card); });
                    controls.querySelector('.dp-metabox-collapse-btn').addEventListener('click', (e) => {
                        e.stopPropagation();
                        const isCol = card.classList.contains('dp-metabox-collapsed');
                        setMetaBoxCardCollapsed(card, !isCol, true);
                    });
                    header.appendChild(controls);
                });

                try {
                    const localRaw = localStorage.getItem(getEditorScreenStorageKey());
                    if (localRaw) {
                        const parsed = JSON.parse(localRaw);
                        if (parsed && typeof parsed === 'object') editorScreenState = Object.assign(editorScreenState, parsed);
                    }
                } catch (_) {}
                applyEditorScreenStateToDom();

                fetch('/wp-json/daypress/v1/user-preferences')
                    .then(r => r.ok ? r.json() : null)
                    .then(data => {
                        const remote = data && data.preferences && data.preferences.screens && data.preferences.screens.post_editor;
                        if (remote && typeof remote === 'object') {
                            editorScreenState = Object.assign(editorScreenState, remote);
                            applyEditorScreenStateToDom();
                        }
                    })
                    .catch(() => {});
            }

            function toggleEditorScreenOptions() {
                const panel = document.getElementById('editorScreenOptionsPanel');
                if (!panel) return;
                const open = panel.style.display === 'none' || !panel.style.display;
                panel.style.display = open ? 'block' : 'none';
            }

            function onEditorScreenOptionToggle(boxId, visible) {
                editorScreenState.hidden[boxId] = !visible;
                applyEditorScreenStateToDom();
                saveEditorScreenState();
            }

            function setEditorColumnLayout(cols) {
                editorScreenState.columns = Number(cols) === 1 ? 1 : 2;
                applyEditorScreenStateToDom();
                saveEditorScreenState();
            }

            function toggleAllEditorMetaBoxes(collapse) {
                document.querySelectorAll('.day-card[data-metabox-id]').forEach(card => {
                    setMetaBoxCardCollapsed(card, !!collapse, false);
                    const boxId = card.getAttribute('data-metabox-id');
                    editorScreenState.collapsed[card.id || boxId] = !!collapse;
                    editorScreenState.collapsed[boxId] = !!collapse;
                });
                saveEditorScreenState();
            }

            function resetEditorScreenOptions() {
                editorScreenState = { columns: 2, hidden: {}, collapsed: {}, mainOrder: [], sideOrder: [] };
                try { localStorage.removeItem(getEditorScreenStorageKey()); } catch (_) {}
                saveEditorScreenState();
                window.location.reload();
            }

            function toggleEditorDistractionFree() {
                const active = document.body.classList.toggle('dp-editor-fullscreen');
                const sidebar = document.getElementById('day-sidebar');
                const topbar = document.getElementById('day-topbar');
                const btn = document.getElementById('btnEditorFullscreen');
                if (sidebar) sidebar.style.display = active ? 'none' : '';
                if (topbar) topbar.style.display = active ? 'none' : '';
                if (btn) btn.textContent = active ? '✕ Exit Fullscreen' : '⛶ Fullscreen';
                showToast(active ? 'Distraction-Free Fullscreen enabled' : 'Exited Fullscreen mode');
            }
"###;
