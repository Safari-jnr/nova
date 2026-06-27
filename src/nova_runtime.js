// ═══════════════════════════════════════════════════════════════════
// Nova Interactive Runtime (Client-Side JavaScript)
// Makes Nova apps fully interactive with working buttons, navigation, CRUD
// ═══════════════════════════════════════════════════════════════════

const NOVA_RUNTIME = `
<script>
// ═══════════════════════════════════════════════════════════════════
// Nova Interactive Runtime v1.0
// Full client-side interactivity for Nova applications
// ═══════════════════════════════════════════════════════════════════

class NovaApp {
  constructor() {
    this.state = {};
    this.currentPage = 'home';
    this.listeners = {};
    this.storage = this.initStorage();
    this.init();
  }
  
  // ═══════════════════════════════════════════════════════════════════
  // STORAGE & PERSISTENCE
  // ═══════════════════════════════════════════════════════════════════
  
  initStorage() {
    const saved = localStorage.getItem('novaAppState');
    return saved ? JSON.parse(saved) : {
      notes: [],
      posts: [],
      tasks: [],
      settings: {}
    };
  }
  
  saveState() {
    localStorage.setItem('novaAppState', JSON.stringify(this.storage));
  }
  
  // ═══════════════════════════════════════════════════════════════════
  // NAVIGATION
  // ═══════════════════════════════════════════════════════════════════
  
  navigate(page) {
    console.log('Navigating to:', page);
    this.currentPage = page;
    
    // Hide all pages
    document.querySelectorAll('[data-page]').forEach(p => {
      p.style.display = 'none';
    });
    
    // Show selected page
    const targetPage = document.querySelector(\`[data-page="\${page}"]\`);
    if (targetPage) {
      targetPage.style.display = 'block';
    }
    
    // Update active nav buttons
    document.querySelectorAll('[data-nav]').forEach(btn => {
      btn.classList.remove('active');
      if (btn.dataset.nav === page) {
        btn.classList.add('active');
      }
    });
    
    this.trigger('navigate', { page });
  }
  
  // ═══════════════════════════════════════════════════════════════════
  // CRUD OPERATIONS
  // ═══════════════════════════════════════════════════════════════════
  
  create(collection, item) {
    if (!this.storage[collection]) {
      this.storage[collection] = [];
    }
    
    item.id = Date.now();
    item.createdAt = new Date().toISOString();
    this.storage[collection].push(item);
    this.saveState();
    this.trigger('create', { collection, item });
    return item;
  }
  
  read(collection, id = null) {
    if (!this.storage[collection]) return null;
    
    if (id) {
      return this.storage[collection].find(item => item.id === id);
    }
    return this.storage[collection];
  }
  
  update(collection, id, updates) {
    if (!this.storage[collection]) return null;
    
    const index = this.storage[collection].findIndex(item => item.id === id);
    if (index !== -1) {
      this.storage[collection][index] = {
        ...this.storage[collection][index],
        ...updates,
        updatedAt: new Date().toISOString()
      };
      this.saveState();
      this.trigger('update', { collection, id, updates });
      return this.storage[collection][index];
    }
    return null;
  }
  
  delete(collection, id) {
    if (!this.storage[collection]) return false;
    
    const index = this.storage[collection].findIndex(item => item.id === id);
    if (index !== -1) {
      const deleted = this.storage[collection].splice(index, 1)[0];
      this.saveState();
      this.trigger('delete', { collection, id, deleted });
      return true;
    }
    return false;
  }
  
  // ═══════════════════════════════════════════════════════════════════
  // MODAL & FORMS
  // ═══════════════════════════════════════════════════════════════════
  
  openModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
      modal.style.display = 'flex';
      document.body.style.overflow = 'hidden';
    }
  }
  
  closeModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
      modal.style.display = 'none';
      document.body.style.overflow = 'auto';
    }
  }
  
  getFormData(formId) {
    const form = document.getElementById(formId);
    if (!form) return {};
    
    const data = {};
    const inputs = form.querySelectorAll('input, textarea, select');
    
    inputs.forEach(input => {
      if (input.name) {
        data[input.name] = input.value;
      }
    });
    
    return data;
  }
  
  resetForm(formId) {
    const form = document.getElementById(formId);
    if (form) {
      form.reset();
    }
  }
  
  // ═══════════════════════════════════════════════════════════════════
  // EVENT SYSTEM
  // ═══════════════════════════════════════════════════════════════════
  
  on(event, callback) {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event].push(callback);
  }
  
  trigger(event, data) {
    if (this.listeners[event]) {
      this.listeners[event].forEach(callback => callback(data));
    }
  }
  
  // ═══════════════════════════════════════════════════════════════════
  // UI UPDATES
  // ═══════════════════════════════════════════════════════════════════
  
  render(elementId, html) {
    const element = document.getElementById(elementId);
    if (element) {
      element.innerHTML = html;
    }
  }
  
  toast(message, type = 'success') {
    const toast = document.createElement('div');
    toast.className = \`nova-toast nova-toast-\${type}\`;
    toast.textContent = message;
    toast.style.cssText = \`
      position: fixed;
      top: 20px;
      right: 20px;
      padding: 15px 25px;
      background: \${type === 'success' ? '#2ecc71' : type === 'error' ? '#e74c3c' : '#3498db'};
      color: white;
      border-radius: 8px;
      box-shadow: 0 4px 12px rgba(0,0,0,0.2);
      z-index: 10000;
      animation: slideIn 0.3s ease;
    \`;
    
    document.body.appendChild(toast);
    
    setTimeout(() => {
      toast.style.animation = 'slideOut 0.3s ease';
      setTimeout(() => toast.remove(), 300);
    }, 3000);
  }
  
  confirm(message, onConfirm) {
    if (window.confirm(message)) {
      onConfirm();
    }
  }
  
  // ═══════════════════════════════════════════════════════════════════
  // SEARCH & FILTER
  // ═══════════════════════════════════════════════════════════════════
  
  search(collection, query, fields = ['title', 'content']) {
    if (!this.storage[collection]) return [];
    
    const lowerQuery = query.toLowerCase();
    return this.storage[collection].filter(item => {
      return fields.some(field => {
        return item[field] && item[field].toLowerCase().includes(lowerQuery);
      });
    });
  }
  
  filter(collection, predicate) {
    if (!this.storage[collection]) return [];
    return this.storage[collection].filter(predicate);
  }
  
  // ═══════════════════════════════════════════════════════════════════
  // INITIALIZATION
  // ═══════════════════════════════════════════════════════════════════
  
  init() {
    // Add global click handler
    document.addEventListener('click', (e) => {
      // Navigation buttons
      if (e.target.dataset.nav) {
        e.preventDefault();
        this.navigate(e.target.dataset.nav);
      }
      
      // CRUD buttons
      if (e.target.dataset.action) {
        e.preventDefault();
        this.handleAction(e.target.dataset.action, e.target.dataset);
      }
      
      // Modal close on overlay
      if (e.target.classList.contains('nova-modal')) {
        this.closeModal(e.target.id);
      }
    });
    
    // Add CSS animations
    const style = document.createElement('style');
    style.textContent = \`
      @keyframes slideIn {
        from { transform: translateX(400px); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
      }
      @keyframes slideOut {
        from { transform: translateX(0); opacity: 1; }
        to { transform: translateX(400px); opacity: 0; }
      }
      .nova-modal {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0,0,0,0.5);
        display: none;
        align-items: center;
        justify-content: center;
        z-index: 9999;
      }
      .nova-modal-content {
        background: white;
        padding: 30px;
        border-radius: 12px;
        max-width: 600px;
        width: 90%;
        max-height: 80vh;
        overflow-y: auto;
      }
      .active {
        background: #2c3e50 !important;
      }
    \`;
    document.head.appendChild(style);
    
    console.log(' Nova Interactive Runtime initialized');
  }
  
  handleAction(action, data) {
    console.log('Action:', action, data);
    
    switch(action) {
      case 'create':
        this.openModal(data.modal || 'createModal');
        break;
      case 'edit':
        this.openModal(data.modal || 'editModal');
        break;
      case 'delete':
        this.confirm('Are you sure you want to delete this?', () => {
          this.delete(data.collection, parseInt(data.id));
          this.toast('Deleted successfully!');
          location.reload(); // Reload to show changes
        });
        break;
      case 'save':
        const formData = this.getFormData(data.form);
        if (data.id) {
          this.update(data.collection, parseInt(data.id), formData);
          this.toast('Updated successfully!');
        } else {
          this.create(data.collection, formData);
          this.toast('Created successfully!');
        }
        this.closeModal(data.modal);
        this.resetForm(data.form);
        location.reload(); // Reload to show changes
        break;
      case 'cancel':
        this.closeModal(data.modal);
        this.resetForm(data.form);
        break;
    }
  }
}

// Initialize app
const app = new NovaApp();
window.novaApp = app;

// Expose globally for easy access
window.navigate = (page) => app.navigate(page);
window.openModal = (id) => app.openModal(id);
window.closeModal = (id) => app.closeModal(id);

console.log(' Nova app ready! Use window.novaApp to interact');
</script>
`;

module.exports = NOVA_RUNTIME;