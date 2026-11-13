//! WordPress Meta Boxes (Collapse, Move Up/Down/Column, Drag) & Screen Options (`scripts/screen_options.rs`).

pub const JS_SCREEN_OPTIONS: &str = r###"
            let editorScreenState = {
                v: 2,
                columns: 2,
                hidden: {},
                collapsed: {},
                mainOrder: [],
                sideOrder: []
            };
            let draggedMetaBoxEl = null;

            function getEditorScreenStorageKey() {
                return 'daypress_editor_screen_prefs_v2';
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
                    grid.style.gridTemplateColumns = cols === 1 ? '1fr' : 'minmax(0, 1fr) 280px';
                }
                const r1 = document.getElementById('editorCols1');
                const r2 = document.getElementById('editorCols2');
                if (r1) r1.checked = cols === 1;
                if (r2) r2.checked = cols === 2;

                const placeCards = (colEl, orderArr) => {
                    if (!colEl || !Array.isArray(orderArr) || orderArr.length === 0) return;
                    orderArr.forEach(ident => {
                        const card = document.getElementById(ident) || document.querySelector(`.day-card[data-metabox-id="${ident}"]`);
                        if (card && card.hasAttribute('data-metabox-id')) {
                            colEl.appendChild(card);
                        }
                    });
                };
                placeCards(mainCol, editorScreenState.mainOrder);
                if (Array.isArray(editorScreenState.sideOrder) && editorScreenState.sideOrder.includes('metabox-status_visibility')) {
                    placeCards(sideCol, editorScreenState.sideOrder);
                }

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
                    toggleBtn.title = collapsed ? 'Toggle panel: Expand' : 'Toggle panel: Collapse';
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
                if (targetIdx < 0) {
                    const mainCol = document.getElementById('editorMainColumn');
                    const sideCol = document.getElementById('editorSideColumn');
                    if (parent === sideCol && mainCol) {
                        mainCol.appendChild(card);
                        saveEditorScreenState();
                    }
                    return;
                }
                if (targetIdx >= cards.length) {
                    const mainCol = document.getElementById('editorMainColumn');
                    const sideCol = document.getElementById('editorSideColumn');
                    if (parent === mainCol && sideCol) {
                        sideCol.insertBefore(card, sideCol.firstElementChild);
                        saveEditorScreenState();
                    }
                    return;
                }
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
                ['editorMainColumn', 'editorSideColumn'].forEach(colId => {
                    const col = document.getElementById(colId);
                    if (!col) return;
                    col.addEventListener('dragover', (ev) => {
                        if (!draggedMetaBoxEl) return;
                        ev.preventDefault();
                    });
                    col.addEventListener('drop', (ev) => {
                        if (!draggedMetaBoxEl) return;
                        if (ev.target.closest('.day-card[data-metabox-id]')) return;
                        ev.preventDefault();
                        col.appendChild(draggedMetaBoxEl);
                        saveEditorScreenState();
                    });
                });

                document.querySelectorAll('.day-card[data-metabox-id]').forEach((card, idx) => {
                    if (!card.id) card.id = 'dp-metabox-' + card.getAttribute('data-metabox-id') + '-' + idx;
                    const header = card.querySelector('.day-card-header');
                    if (!header || header.querySelector('.dp-metabox-controls')) return;
                    header.style.display = 'flex';
                    header.style.justifyContent = 'space-between';
                    header.style.alignItems = 'center';
                    header.style.cursor = 'move';
                    header.addEventListener('click', (ev) => {
                        if (ev.target.closest('button, a, input, select, label')) return;
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
                    controls.className = 'handle-actions dp-metabox-controls';
                    controls.innerHTML = `
                        <button type="button" class="handle-order-higher dp-metabox-up-btn" aria-label="Move up" title="Move up">
                            <span class="order-higher-indicator" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M6.5 14.4L12 8.9l5.5 5.5-1.1 1.1L12 11.1l-4.4 4.4z"/></svg></span>
                        </button>
                        <button type="button" class="handle-order-lower dp-metabox-down-btn" aria-label="Move down" title="Move down">
                            <span class="order-lower-indicator" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M17.5 9.6L12 15.1 6.5 9.6l1.1-1.1L12 12.9l4.4-4.4z"/></svg></span>
                        </button>
                        <button type="button" class="handlediv dp-metabox-collapse-btn" aria-expanded="true" aria-label="Toggle panel" title="Toggle panel">
                            <span class="toggle-indicator" aria-hidden="true"><svg viewBox="0 0 20 20"><path d="M5 13l5-5 5 5H5z"/></svg></span>
                        </button>
                    `;
                    controls.querySelector('.dp-metabox-up-btn').addEventListener('click', (e) => { e.stopPropagation(); moveEditorMetaBox(card, -1); });
                    controls.querySelector('.dp-metabox-down-btn').addEventListener('click', (e) => { e.stopPropagation(); moveEditorMetaBox(card, 1); });
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
                        if (remote && typeof remote === 'object' && remote.v === 2) {
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
