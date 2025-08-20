/**
 * EasyMDE Fallback - Simple markdown editor fallback when EasyMDE CDN is not available
 */

// Simple EasyMDE fallback class
class EasyMDEFallback {
    constructor(options = {}) {
        this.element = options.element;
        this.options = options;
        
        if (!this.element) {
            console.error('EasyMDE Fallback: No element provided');
            return;
        }
        
        this.init();
    }
    
    init() {
        // Add enhanced styling class
        this.element.classList.add('textarea-enhanced');
        
        // Create toolbar
        this.createToolbar();
        
        // Set placeholder if provided
        if (this.options.placeholder) {
            this.element.placeholder = this.options.placeholder;
        }
        
        // Set initial value if element has content
        this.initialValue = this.element.value;
        
        // Add keyboard shortcuts
        this.addKeyboardShortcuts();
        
        console.log('EasyMDE Fallback: Initialized successfully');
    }
    
    createToolbar() {
        const toolbar = document.createElement('div');
        toolbar.className = 'textarea-toolbar';
        
        const buttons = [
            { name: 'bold', title: 'Bold (Ctrl+B)', action: () => this.toggleBold() },
            { name: 'italic', title: 'Italic (Ctrl+I)', action: () => this.toggleItalic() },
            { name: 'heading', title: 'Heading', action: () => this.toggleHeading() },
            { name: 'separator', type: 'separator' },
            { name: 'quote', title: 'Quote', action: () => this.toggleQuote() },
            { name: 'list-ul', title: 'Unordered List', action: () => this.toggleUnorderedList() },
            { name: 'list-ol', title: 'Ordered List', action: () => this.toggleOrderedList() },
            { name: 'separator', type: 'separator' },
            { name: 'link', title: 'Link', action: () => this.insertLink() },
            { name: 'image', title: 'Image', action: () => this.insertImage() },
            { name: 'separator', type: 'separator' },
            { name: 'code', title: 'Code', action: () => this.toggleCode() },
            { name: 'table', title: 'Table', action: () => this.insertTable() }
        ];
        
        buttons.forEach(btn => {
            if (btn.type === 'separator') {
                const sep = document.createElement('span');
                sep.textContent = '|';
                sep.style.color = '#333';
                sep.style.margin = '0 5px';
                toolbar.appendChild(sep);
            } else {
                const button = document.createElement('button');
                button.type = 'button';
                button.title = btn.title;
                button.textContent = this.getButtonLabel(btn.name);
                button.onclick = btn.action;
                toolbar.appendChild(button);
            }
        });
        
        // Insert toolbar before the textarea
        this.element.parentNode.insertBefore(toolbar, this.element);
    }
    
    getButtonLabel(name) {
        const labels = {
            'bold': 'B',
            'italic': 'I',
            'heading': 'H',
            'quote': '"',
            'list-ul': '•',
            'list-ol': '1.',
            'link': '🔗',
            'image': '📷',
            'code': '<>',
            'table': '▦'
        };
        return labels[name] || name;
    }
    
    addKeyboardShortcuts() {
        this.element.addEventListener('keydown', (e) => {
            if (e.ctrlKey || e.metaKey) {
                switch (e.key.toLowerCase()) {
                    case 'b':
                        e.preventDefault();
                        this.toggleBold();
                        break;
                    case 'i':
                        e.preventDefault();
                        this.toggleItalic();
                        break;
                }
            }
        });
    }
    
    // Get current value
    value(newValue) {
        if (newValue !== undefined) {
            this.element.value = newValue;
            return this;
        }
        return this.element.value;
    }
    
    // Insert text at cursor position
    insertText(text) {
        const start = this.element.selectionStart;
        const end = this.element.selectionEnd;
        const currentValue = this.element.value;
        
        this.element.value = currentValue.substring(0, start) + text + currentValue.substring(end);
        
        // Move cursor to end of inserted text
        const newPosition = start + text.length;
        this.element.setSelectionRange(newPosition, newPosition);
        this.element.focus();
    }
    
    // Wrap selected text with given characters
    wrapSelection(before, after = before) {
        const start = this.element.selectionStart;
        const end = this.element.selectionEnd;
        const selectedText = this.element.value.substring(start, end);
        
        if (selectedText) {
            this.insertText(before + selectedText + after);
        } else {
            this.insertText(before + after);
            // Move cursor between the wrapper characters
            this.element.setSelectionRange(start + before.length, start + before.length);
        }
    }
    
    // Toolbar actions
    toggleBold() {
        this.wrapSelection('**');
    }
    
    toggleItalic() {
        this.wrapSelection('*');
    }
    
    toggleCode() {
        this.wrapSelection('`');
    }
    
    toggleHeading() {
        const start = this.element.selectionStart;
        const lines = this.element.value.split('\n');
        let lineStart = 0;
        let currentLineIndex = 0;
        
        // Find which line the cursor is on
        for (let i = 0; i < lines.length; i++) {
            if (start <= lineStart + lines[i].length) {
                currentLineIndex = i;
                break;
            }
            lineStart += lines[i].length + 1; // +1 for newline
        }
        
        const currentLine = lines[currentLineIndex];
        let newLine;
        
        if (currentLine.startsWith('### ')) {
            newLine = currentLine.substring(4);
        } else if (currentLine.startsWith('## ')) {
            newLine = '### ' + currentLine.substring(3);
        } else if (currentLine.startsWith('# ')) {
            newLine = '## ' + currentLine.substring(2);
        } else {
            newLine = '# ' + currentLine;
        }
        
        lines[currentLineIndex] = newLine;
        this.element.value = lines.join('\n');
        this.element.focus();
    }
    
    toggleQuote() {
        const start = this.element.selectionStart;
        const lines = this.element.value.split('\n');
        let lineStart = 0;
        let currentLineIndex = 0;
        
        // Find which line the cursor is on
        for (let i = 0; i < lines.length; i++) {
            if (start <= lineStart + lines[i].length) {
                currentLineIndex = i;
                break;
            }
            lineStart += lines[i].length + 1;
        }
        
        const currentLine = lines[currentLineIndex];
        let newLine;
        
        if (currentLine.startsWith('> ')) {
            newLine = currentLine.substring(2);
        } else {
            newLine = '> ' + currentLine;
        }
        
        lines[currentLineIndex] = newLine;
        this.element.value = lines.join('\n');
        this.element.focus();
    }
    
    toggleUnorderedList() {
        this.insertText('\n- ');
    }
    
    toggleOrderedList() {
        this.insertText('\n1. ');
    }
    
    insertLink() {
        const url = prompt('Enter URL:');
        if (url) {
            const text = prompt('Enter link text:') || 'Link';
            this.insertText(`[${text}](${url})`);
        }
    }
    
    insertImage() {
        const url = prompt('Enter image URL:');
        if (url) {
            const alt = prompt('Enter alt text:') || 'Image';
            this.insertText(`![${alt}](${url})`);
        }
    }
    
    insertTable() {
        const table = `
| Column 1 | Column 2 | Column 3 |
| -------- | -------- | -------- |
| Text     | Text     | Text     |
`;
        this.insertText(table);
    }
}

// Create global EasyMDE constructor if it doesn't exist
if (typeof EasyMDE === 'undefined') {
    console.log('EasyMDE not found, using fallback implementation');
    window.EasyMDE = EasyMDEFallback;
}

// Also provide a simple Bootstrap replacement for basic functionality
if (typeof bootstrap === 'undefined') {
    console.log('Bootstrap not found, using basic fallback');
    window.bootstrap = {
        Popover: function() {
            console.log('Bootstrap Popover fallback - no-op');
        }
    };
}