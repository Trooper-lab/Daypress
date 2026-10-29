//! ACF and Gutenberg blocks inspector, cards rendering, and templates (`scripts/acf_blocks.rs`).

pub const JS_ACF_BLOCKS: &str = r###"
            function parseAndRenderAcfBlocks() {
                const raw = contentArea.value || '';
                acfBlocks = [];

                if (raw.includes('<!-- wp:')) {
                    const commentRegex = /<!--\s*(\/)?wp:([a-z0-9_-]+(?:\/[a-z0-9_-]+)?)\s*([\s\S]*?)?-->/gi;
                    let lastIndex = 0;
                    let match;
                    const blockStack = [];

                    while ((match = commentRegex.exec(raw)) !== null) {
                        const fullComment = match[0];
                        const isClose = match[1] === '/';
                        const blockName = match[2];
                        const rest = match[3] || '';
                        const isSelfClosing = rest.trim().endsWith('/');
                        const matchStart = match.index;
                        const matchEnd = commentRegex.lastIndex;

                        if (matchStart > lastIndex && blockStack.length === 0) {
                            const pre = raw.substring(lastIndex, matchStart).trim();
                            if (pre) {
                                acfBlocks.push({
                                    idx: acfBlocks.length,
                                    blockType: 'Custom HTML',
                                    badgeColor: '#475569',
                                    badgeBg: '#f1f5f9',
                                    isGutenberg: false,
                                    headings: [],
                                    paragraphs: [pre],
                                    buttons: [],
                                    slideCount: 0,
                                    outerHtml: pre,
                                    rawOpen: '',
                                    rawClose: '',
                                    showRaw: false
                                });
                            }
                        }

                        if (isSelfClosing) {
                            if (blockStack.length === 0) {
                                acfBlocks.push({
                                    idx: acfBlocks.length,
                                    blockType: 'Gutenberg: ' + blockName,
                                    badgeColor: '#2563eb',
                                    badgeBg: '#eff6ff',
                                    isGutenberg: true,
                                    headings: [],
                                    paragraphs: [],
                                    buttons: [],
                                    slideCount: 0,
                                    outerHtml: fullComment,
                                    rawOpen: fullComment,
                                    rawClose: '',
                                    showRaw: false
                                });
                            }
                        } else if (!isClose) {
                            blockStack.push({ blockName, fullComment, startIndex: matchEnd, fullStart: matchStart });
                        } else {
                            if (blockStack.length > 0) {
                                const top = blockStack.pop();
                                if (blockStack.length === 0) {
                                    const innerHtml = raw.substring(top.startIndex, matchStart).trim();
                                    const fullBlockStr = raw.substring(top.fullStart, matchEnd);
                                    const parser = new DOMParser();
                                    const doc = parser.parseFromString('<div>' + innerHtml + '</div>', 'text/html');
                                    const el = doc.body.firstElementChild || doc.body;
                                    const inspected = inspectAcfDomNode(el, acfBlocks.length);
                                    inspected.blockType = 'Gutenberg: ' + top.blockName;
                                    inspected.badgeColor = '#2563eb';
                                    inspected.badgeBg = '#eff6ff';
                                    inspected.isGutenberg = true;
                                    inspected.rawOpen = top.fullComment;
                                    inspected.rawClose = fullComment;
                                    inspected.innerHtml = innerHtml;
                                    inspected.outerHtml = fullBlockStr;
                                    acfBlocks.push(inspected);
                                }
                            }
                        }
                        lastIndex = matchEnd;
                    }

                    if (lastIndex < raw.length && blockStack.length === 0) {
                        const tail = raw.substring(lastIndex).trim();
                        if (tail) {
                            acfBlocks.push({
                                idx: acfBlocks.length,
                                blockType: 'Custom HTML',
                                badgeColor: '#475569',
                                badgeBg: '#f1f5f9',
                                isGutenberg: false,
                                headings: [],
                                paragraphs: [tail],
                                buttons: [],
                                slideCount: 0,
                                outerHtml: tail,
                                rawOpen: '',
                                rawClose: '',
                                showRaw: false
                            });
                        }
                    }
                } else {
                    const parser = new DOMParser();
                    const doc = parser.parseFromString('<div id="acf-root">' + raw + '</div>', 'text/html');
                    const root = doc.getElementById('acf-root');
                    if (root) {
                        const children = Array.from(root.children);
                        if (children.length > 0) {
                            children.forEach((el, idx) => {
                                acfBlocks.push(inspectAcfDomNode(el, idx));
                            });
                        } else if (raw.trim()) {
                            const p = doc.createElement('div');
                            p.innerHTML = raw;
                            acfBlocks.push(inspectAcfDomNode(p, 0));
                        }
                    }
                }
                renderAcfBlocksList();
            }

            function inspectAcfDomNode(el, idx) {
                const cls = el.className || '';
                const html = el.outerHTML || el.innerHTML || '';
                let blockType = 'Gutenberg Block';
                let badgeColor = '#475569';
                let badgeBg = '#f1f5f9';
                let isPageheader = cls.includes('pageheader');
                let isSection = cls.includes('section');

                let bg = 'white';
                if (cls.includes('black')) bg = 'black';
                else if (cls.includes('lblue')) bg = 'lblue';
                else if (cls.includes('blue')) bg = 'blue';

                let hasAngle = cls.includes('angle');
                let heroFull = cls.includes('full');

                if (isPageheader) {
                    blockType = 'Hero Pageheader (ACF)';
                    badgeColor = '#1d4ed8';
                    badgeBg = '#dbeafe';
                } else if (html.includes('logoSwiper') || html.includes('section logo')) {
                    blockType = 'ACF Block: Client Logos (acf/logos)';
                    badgeColor = '#047857';
                    badgeBg = '#d1fae5';
                } else if (html.includes('threeSwiper')) {
                    blockType = 'ACF Block: 3-Col Feature Swiper (acf/threecol)';
                    badgeColor = '#6d28d9';
                    badgeBg = '#ede9fe';
                } else if (html.includes('trainSwiper')) {
                    blockType = 'ACF Block: Showcase Carousel (acf/showcase)';
                    badgeColor = '#b45309';
                    badgeBg = '#fef3c7';
                } else if (html.includes('quoteSwiper')) {
                    blockType = 'ACF Block: Quotes Slider (acf/quotes)';
                    badgeColor = '#be185d';
                    badgeBg = '#fce7f3';
                } else if (html.includes('accordion')) {
                    blockType = 'ACF Block: FAQ Accordion (acf/faq)';
                    badgeColor = '#0369a1';
                    badgeBg = '#e0f2fe';
                } else if (html.includes('vacatures') || html.includes('filter')) {
                    blockType = 'ACF Block: CPT Overview Grid';
                    badgeColor = '#4338ca';
                    badgeBg = '#e0e7ff';
                } else if (isSection) {
                    blockType = 'ACF Section Wrapper (acf/section)';
                    badgeColor = '#0f172a';
                    badgeBg = '#e2e8f0';
                }

                const headings = Array.from(el.querySelectorAll('h1, h2, h3, h4')).map(h => ({
                    tag: h.tagName.toLowerCase(),
                    text: h.textContent.trim()
                }));
                const paragraphs = Array.from(el.querySelectorAll('p')).filter(p => !p.closest('.swiper-slide') && p.textContent.trim().length > 0).map(p => p.innerHTML.trim());
                const buttons = Array.from(el.querySelectorAll('a.btn, a.wp-block-button__link')).map(a => ({
                    text: a.textContent.trim(),
                    href: a.getAttribute('href') || '#'
                }));
                const slideCount = el.querySelectorAll('.swiper-slide, .accordion-item, .col-md-4, .col-md-6').length;

                return {
                    idx,
                    blockType,
                    badgeColor,
                    badgeBg,
                    isPageheader,
                    isSection,
                    bg,
                    hasAngle,
                    heroFull,
                    headings,
                    paragraphs,
                    buttons,
                    slideCount,
                    outerHtml: html,
                    showRaw: false
                };
            }

            function syncAcfBlocksToTextarea() {
                contentArea.value = acfBlocks.map(b => b.outerHtml).join('\n\n');
                visualEditor.innerHTML = cleanWpContent(contentArea.value);
                updateWordCount();
                updateSerpPreview();
                markDirty();
            }

            function mutateAcfBlockDom(idx, mutatorFn) {
                const b = acfBlocks[idx];
                if (!b) return;
                const targetHtml = b.innerHtml !== undefined ? b.innerHtml : b.outerHtml;
                const parser = new DOMParser();
                const doc = parser.parseFromString('<div id="mut-root">' + targetHtml + '</div>', 'text/html');
                const el = doc.getElementById('mut-root').firstElementChild || doc.getElementById('mut-root');
                if (!el) return;
                mutatorFn(el, doc);
                const updatedTargetHtml = el.outerHTML;
                if (b.innerHtml !== undefined) {
                    b.innerHtml = updatedTargetHtml;
                    b.outerHtml = (b.rawOpen || '') + updatedTargetHtml + (b.rawClose || '');
                } else {
                    b.outerHtml = updatedTargetHtml;
                }
                const inspected = inspectAcfDomNode(el, idx);
                inspected.rawOpen = b.rawOpen;
                inspected.rawClose = b.rawClose;
                inspected.innerHtml = b.innerHtml;
                inspected.outerHtml = b.outerHtml;
                inspected.blockType = b.blockType;
                inspected.badgeColor = b.badgeColor;
                inspected.badgeBg = b.badgeBg;
                inspected.isGutenberg = b.isGutenberg;
                acfBlocks[idx] = inspected;
                syncAcfBlocksToTextarea();
            }

            function updateAcfSectionProp(idx, prop, val) {
                mutateAcfBlockDom(idx, (el) => {
                    if (prop === 'bg') {
                        el.classList.remove('white', 'black', 'blue', 'lblue');
                        el.classList.add(val);
                    } else if (prop === 'angle') {
                        if (val) el.classList.add('angle');
                        else el.classList.remove('angle');
                    } else if (prop === 'heroFull') {
                        if (val) el.classList.add('full');
                        else el.classList.remove('full');
                    }
                });
                renderAcfBlocksList();
            }

            function updateAcfHeading(idx, hIdx, text) {
                mutateAcfBlockDom(idx, (el) => {
                    const hs = el.querySelectorAll('h1, h2, h3, h4');
                    if (hs[hIdx]) hs[hIdx].textContent = text;
                });
            }

            function updateAcfParagraph(idx, pIdx, htmlVal) {
                mutateAcfBlockDom(idx, (el) => {
                    const ps = Array.from(el.querySelectorAll('p')).filter(p => !p.closest('.swiper-slide') && p.textContent.trim().length > 0);
                    if (ps[pIdx]) ps[pIdx].innerHTML = htmlVal;
                });
            }

            function updateAcfButton(idx, btnIdx, field, val) {
                mutateAcfBlockDom(idx, (el) => {
                    const btns = el.querySelectorAll('a.btn, a.wp-block-button__link');
                    if (btns[btnIdx]) {
                        if (field === 'text') btns[btnIdx].textContent = val;
                        else if (field === 'href') btns[btnIdx].setAttribute('href', val);
                    }
                });
            }

            function toggleAcfRawHtml(idx) {
                if (!acfBlocks[idx]) return;
                acfBlocks[idx].showRaw = !acfBlocks[idx].showRaw;
                renderAcfBlocksList();
            }

            function updateAcfRawHtml(idx, rawHtml) {
                if (!acfBlocks[idx]) return;
                acfBlocks[idx].outerHtml = rawHtml;
                syncAcfBlocksToTextarea();
            }

            function moveAcfBlock(idx, dir) {
                const target = idx + dir;
                if (target < 0 || target >= acfBlocks.length) return;
                const tmp = acfBlocks[idx];
                acfBlocks[idx] = acfBlocks[target];
                acfBlocks[target] = tmp;
                syncAcfBlocksToTextarea();
                renderAcfBlocksList();
            }

            function duplicateAcfBlock(idx) {
                if (!acfBlocks[idx]) return;
                const copy = JSON.parse(JSON.stringify(acfBlocks[idx]));
                acfBlocks.splice(idx + 1, 0, copy);
                syncAcfBlocksToTextarea();
                renderAcfBlocksList();
                showToast('✓ ACF Section duplicated');
            }

            function deleteAcfBlock(idx) {
                if (!acfBlocks[idx]) return;
                acfBlocks.splice(idx, 1);
                syncAcfBlocksToTextarea();
                renderAcfBlocksList();
                showToast('Section removed');
            }

            function insertNewAcfSection() {
                const sel = document.getElementById('newAcfBlockSelect');
                const tpl = sel ? sel.value : 'section_white';
                let snippet = '';
                if (tpl === 'section_white') {
                    snippet = `<section class="section white"><div class="container"><div class="row justify-content-center"><div class="col-md-10"><h2>CONTENT SECTION</h2><p>Add your rich text and block components here.</p><div class="wp-block-buttons"><div class="wp-block-button"><a class="wp-block-button__link wp-element-button" href="/contact/">Get in touch</a></div></div></div></div></div></section>`;
                } else if (tpl === 'section_black') {
                    snippet = `<section class="section black angle"><div class="container"><div class="row justify-content-center"><div class="col-md-10"><h2>HIGH-CONTRAST SECTION</h2><p>High-contrast section with modern accent background.</p><a href="/about/" class="btn btn-primary">Learn more</a></div></div></div></section>`;
                } else if (tpl === 'section_blue') {
                    snippet = `<section class="section blue angle"><div class="container"><div class="row justify-content-center"><div class="col-md-10"><h2>PRIMARY CALL TO ACTION</h2><p>Featured call-to-action section in theme primary color.</p><a href="/contact/" class="btn btn-primary">Schedule a consultation</a></div></div></div></section>`;
                } else if (tpl === 'acf_3col') {
                    snippet = `<section class="section white"><div class="container"><h2>THREE-COLUMN FEATURES</h2><div class="row"><div class="col-md-4"><div class="card"><div class="card-body"><h3>Feature One</h3><p>Fast, modular architecture designed for performance.</p><a href="#" class="btn btn-primary">Read more</a></div></div></div><div class="col-md-4"><div class="card"><div class="card-body"><h3>Feature Two</h3><p>Native Rust execution with zero PHP latency.</p><a href="#" class="btn btn-primary">Read more</a></div></div></div><div class="col-md-4"><div class="card"><div class="card-body"><h3>Feature Three</h3><p>1-Click seamless migration from legacy WordPress.</p><a href="#" class="btn btn-primary">Read more</a></div></div></div></div></div></section>`;
                } else if (tpl === 'acf_logos') {
                    snippet = `<section class="section logo white"><div class="container"><h2 class="text-center">TRUSTED PARTNERS</h2><div class="logo-grid" style="display: flex; justify-content: space-around; align-items: center; gap: 24px; flex-wrap: wrap;"><div class="logo-item" style="font-weight: 700; font-size: 18px; color: var(--ink-muted);">PARTNER 1</div><div class="logo-item" style="font-weight: 700; font-size: 18px; color: var(--ink-muted);">PARTNER 2</div><div class="logo-item" style="font-weight: 700; font-size: 18px; color: var(--ink-muted);">PARTNER 3</div><div class="logo-item" style="font-weight: 700; font-size: 18px; color: var(--ink-muted);">PARTNER 4</div></div></div></section>`;
                } else if (tpl === 'acf_trainingen') {
                    snippet = `<section class="section black angle"><div class="container"><h2>FEATURE SHOWCASE</h2><div class="row"><div class="col-md-4"><div class="card"><div class="card-body"><h3>Showcase Item</h3><p>Detailed overview of key deliverables and outcomes.</p><a href="/showcase/" class="btn btn-primary">View details</a></div></div></div></div></div></section>`;
                } else if (tpl === 'acf_faq') {
                    snippet = `<section class="section white"><div class="container"><h2>FREQUENTLY ASKED QUESTIONS</h2><div class="accordion"><div class="accordion-item"><h3 class="accordion-header"><button class="accordion-button" type="button">What features are supported?</button></h3><div class="accordion-collapse collapse show"><div class="accordion-body"><p>Daypress natively supports Elementor, Divi, Gutenberg block themes, and ACF fields with sub-millisecond SSR latency.</p></div></div></div></div></div></section>`;
                } else if (tpl === 'acf_quotes') {
                    snippet = `<section class="section blue angle"><div class="container"><h2>CLIENT TESTIMONIALS</h2><div class="swiper quoteSwiper"><div class="swiper-wrapper"><div class="swiper-slide"><blockquote><p>"Daypress delivered instant acceleration and rock-solid stability."</p><footer>— Technology Director</footer></blockquote></div></div></div></div></section>`;
                }

                const parser = new DOMParser();
                const doc = parser.parseFromString('<div id="new-root">' + snippet + '</div>', 'text/html');
                const el = doc.getElementById('new-root').firstElementChild;
                if (el) {
                    acfBlocks.push(inspectAcfDomNode(el, acfBlocks.length));
                    syncAcfBlocksToTextarea();
                    renderAcfBlocksList();
                    showToast('✓ Added ' + (sel.options[sel.selectedIndex]?.text || 'ACF Block'));
                }
            }
"###;
