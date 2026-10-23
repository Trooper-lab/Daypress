//! Formatting toolbar actions and HTML manipulations (`scripts/toolbar.rs`).

pub const JS_TOOLBAR: &str = r###"
            function formatBlockType(tag) {
                if (!tag) return;
                if (currentMode === 'visual') {
                    visualEditor.focus();
                    document.execCommand('formatBlock', false, tag);
                    syncVisualToTextarea();
                } else {
                    wrapSelectionInTextarea(tag);
                }
                markDirty();
            }

            function execStyle(cmd) {
                if (currentMode === 'visual') {
                    visualEditor.focus();
                    document.execCommand(cmd, false, null);
                    syncVisualToTextarea();
                } else {
                    const map = { 'bold': 'strong', 'italic': 'em', 'underline': 'u', 'strikeThrough': 's' };
                    wrapSelectionInTextarea(map[cmd] || cmd);
                }
                markDirty();
            }

            function wrapSelectionInTextarea(tag) {
                const start = contentArea.selectionStart;
                const end = contentArea.selectionEnd;
                const sel = contentArea.value.substring(start, end);
                const before = contentArea.value.substring(0, start);
                const after = contentArea.value.substring(end);
                const replacement = '<' + tag + '>' + (sel || 'text') + '</' + tag + '>';
                contentArea.value = before + replacement + after;
                contentArea.focus();
                contentArea.selectionStart = start + tag.length + 2;
                contentArea.selectionEnd = start + tag.length + 2 + (sel || 'text').length;
                updateWordCount();
                markDirty();
            }

            function insertInlineCode() {
                if (currentMode === 'visual') {
                    visualEditor.focus();
                    const sel = window.getSelection();
                    const text = sel ? sel.toString() : '';
                    document.execCommand('insertHTML', false, '<code>' + (text || 'code') + '</code>');
                    syncVisualToTextarea();
                } else {
                    wrapSelectionInTextarea('code');
                }
                markDirty();
            }

            function insertList(type) {
                if (currentMode === 'visual') {
                    visualEditor.focus();
                    if (type === 'ul') {
                        document.execCommand('insertUnorderedList', false, null);
                    } else {
                        document.execCommand('insertOrderedList', false, null);
                    }
                    syncVisualToTextarea();
                } else {
                    const text = '\n<' + type + '>\n  <li>List item 1</li>\n  <li>List item 2</li>\n</' + type + '>\n';
                    insertRaw(text);
                }
                markDirty();
            }

            function insertQuote() {
                if (currentMode === 'visual') {
                    visualEditor.focus();
                    document.execCommand('formatBlock', false, '<blockquote>');
                    syncVisualToTextarea();
                } else {
                    wrapSelectionInTextarea('blockquote');
                }
                markDirty();
            }

            function insertLinkPrompt() {
                const url = prompt("Enter URL destination (e.g. https://... or /about-us/):", "https://");
                if (!url) return;
                if (currentMode === 'visual') {
                    visualEditor.focus();
                    document.execCommand('createLink', false, url);
                    syncVisualToTextarea();
                } else {
                    const start = contentArea.selectionStart;
                    const end = contentArea.selectionEnd;
                    const sel = contentArea.value.substring(start, end) || url;
                    const replace = '<a href="' + url + '">' + sel + '</a>';
                    contentArea.value = contentArea.value.substring(0, start) + replace + contentArea.value.substring(end);
                    contentArea.focus();
                    updateWordCount();
                }
                markDirty();
            }

            function clearFormatting() {
                if (currentMode === 'visual') {
                    visualEditor.focus();
                    document.execCommand('removeFormat', false, null);
                    syncVisualToTextarea();
                }
                markDirty();
            }

            function insertTable() {
                const tableHtml = '\n<table class="neo-table" style="width: 100%; border-collapse: collapse; margin: 16px 0;">\n  <thead>\n    <tr style="background: #f8fafc;">\n      <th style="border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left;">Header 1</th>\n      <th style="border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left;">Header 2</th>\n      <th style="border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left;">Header 3</th>\n    </tr>\n  </thead>\n  <tbody>\n    <tr>\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data A1</td>\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data A2</td>\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data A3</td>\n    </tr>\n    <tr>\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data B1</td>\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data B2</td>\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data B3</td>\n    </tr>\n  </tbody>\n</table>\n<p></p>\n';
                if (currentMode === 'visual') {
                    visualEditor.focus();
                    document.execCommand('insertHTML', false, tableHtml);
                    syncVisualToTextarea();
                } else {
                    insertRaw(tableHtml);
                }
                markDirty();
            }

            function insertCallout() {
                const calloutHtml = '\n<div style="background: #eff6ff; border-left: 4px solid #2563eb; padding: 14px 18px; border-radius: 6px; margin: 16px 0; font-size: 14px; color: #1e3a8a;">\n    <strong>💡 Note:</strong> Add important highlighted details or tips here.\n</div>\n<p></p>\n';
                if (currentMode === 'visual') {
                    visualEditor.focus();
                    document.execCommand('insertHTML', false, calloutHtml);
                    syncVisualToTextarea();
                } else {
                    insertRaw(calloutHtml);
                }
                markDirty();
            }

            function insertRaw(text) {
                const start = contentArea.selectionStart;
                const end = contentArea.selectionEnd;
                contentArea.value = contentArea.value.substring(0, start) + text + contentArea.value.substring(end);
                contentArea.focus();
                updateWordCount();
                markDirty();
            }
"###;
