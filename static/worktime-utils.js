/**
 * Work Time Tracker JavaScript Utilities
 * 
 * Modular JavaScript functions for work time tracking functionality.
 * This file provides reusable functions to reduce duplication across templates.
 */

// API utilities
const WorkTimeAPI = {
    /**
     * Fetch work time statistics
     */
    async fetchStats() {
        try {
            const response = await fetch('/worktime/api/stats');
            if (!response.ok) throw new Error('Failed to fetch stats');
            return await response.json();
        } catch (error) {
            console.error('Stats refresh failed:', error);
            return null;
        }
    },

    /**
     * Start a timer
     */
    async startTimer(data) {
        try {
            const response = await fetch('/worktime/start', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded',
                },
                body: new URLSearchParams(data)
            });
            return response.ok;
        } catch (error) {
            console.error('Failed to start timer:', error);
            return false;
        }
    },

    /**
     * Stop the active timer
     */
    async stopTimer() {
        try {
            const response = await fetch('/worktime/stop', {
                method: 'POST'
            });
            return response.ok;
        } catch (error) {
            console.error('Failed to stop timer:', error);
            return false;
        }
    }
};

// Timer utilities
const TimerUtils = {
    /**
     * Calculate elapsed time from start time
     */
    calculateElapsed(startTimeStr) {
        const startTime = new Date(startTimeStr);
        const now = new Date();
        const elapsed = Math.floor((now - startTime) / 1000);
        
        // Handle negative values (when start time is in the future)
        const isNegative = elapsed < 0;
        const absElapsed = Math.abs(elapsed);
        
        return {
            elapsed: absElapsed,
            isNegative,
            hours: Math.floor(absElapsed / 3600),
            minutes: Math.floor((absElapsed % 3600) / 60),
            seconds: absElapsed % 60
        };
    },

    /**
     * Format elapsed time as HH:MM:SS
     */
    formatElapsed(startTimeStr, showSign = true) {
        const time = this.calculateElapsed(startTimeStr);
        const sign = time.isNegative && showSign ? '-' : '';
        
        return `${sign}${time.hours.toString().padStart(2, '0')}:${time.minutes.toString().padStart(2, '0')}:${time.seconds.toString().padStart(2, '0')}`;
    },

    /**
     * Update a timer display element
     */
    updateTimerDisplay(elementId, startTime) {
        const element = document.getElementById(elementId);
        if (!element || !startTime) return;
        
        element.textContent = this.formatElapsed(startTime);
    },

    /**
     * Start auto-updating a timer display
     */
    startTimerUpdates(elementId, startTime, interval = 1000) {
        // Update immediately
        this.updateTimerDisplay(elementId, startTime);
        
        // Set up interval
        return setInterval(() => {
            this.updateTimerDisplay(elementId, startTime);
        }, interval);
    }
};

// UI utilities
const UIUtils = {
    /**
     * Update statistics display
     */
    updateStats(stats) {
        const statCards = document.querySelectorAll('.stat-card');
        
        statCards.forEach((card, index) => {
            const valueElement = card.querySelector('.stat-value');
            if (!valueElement) return;
            
            // Update based on data attributes or position
            switch (index) {
                case 0:
                    if (stats.total_hours !== undefined) {
                        valueElement.textContent = stats.total_hours.toFixed(2);
                    }
                    break;
                case 1:
                    if (stats.total_earnings !== undefined) {
                        valueElement.textContent = '$' + stats.total_earnings.toFixed(2);
                    }
                    break;
                case 2:
                    if (stats.current_shift_earnings !== undefined) {
                        valueElement.textContent = '$' + stats.current_shift_earnings.toFixed(2);
                    }
                    break;
            }
        });
    },

    /**
     * Show loading state
     */
    setLoading(element, loading = true) {
        if (loading) {
            element.classList.add('loading');
        } else {
            element.classList.remove('loading');
        }
    },

    /**
     * Show notification
     */
    showNotification(message, type = 'info') {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = `alert alert-${type} alert-dismissible fade show position-fixed`;
        notification.style.cssText = 'top: 20px; right: 20px; z-index: 9999; max-width: 300px;';
        notification.innerHTML = `
            ${message}
            <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
        `;
        
        document.body.appendChild(notification);
        
        // Auto-remove after 5 seconds
        setTimeout(() => {
            if (notification.parentNode) {
                notification.remove();
            }
        }, 5000);
    }
};

// Menu utilities
const MenuUtils = {
    /**
     * Initialize hamburger menu
     */
    initializeMenu() {
        const menuToggle = document.getElementById('menu-toggle');
        const sideMenu = document.getElementById('side-menu');
        const sideMenuOverlay = document.getElementById('side-menu-overlay');
        const closeMenu = document.getElementById('close-menu');

        const openMenu = () => {
            if (sideMenu) sideMenu.classList.add('open');
            if (sideMenuOverlay) sideMenuOverlay.classList.add('show');
            document.body.style.overflow = 'hidden';
        };

        const closeMenuFunc = () => {
            if (sideMenu) sideMenu.classList.remove('open');
            if (sideMenuOverlay) sideMenuOverlay.classList.remove('show');
            document.body.style.overflow = '';
        };

        if (menuToggle) {
            menuToggle.addEventListener('click', openMenu);
        }

        if (closeMenu) {
            closeMenu.addEventListener('click', closeMenuFunc);
        }

        if (sideMenuOverlay) {
            sideMenuOverlay.addEventListener('click', closeMenuFunc);
        }

        // Close menu on escape key
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && sideMenu && sideMenu.classList.contains('open')) {
                closeMenuFunc();
            }
        });
    }
};

// PWA utilities
const PWAUtils = {
    /**
     * Handle PWA installation
     */
    initializeInstall() {
        let deferredPrompt;
        
        window.addEventListener('beforeinstallprompt', (e) => {
            e.preventDefault();
            deferredPrompt = e;
            
            const installBtn = document.getElementById('install-btn');
            if (installBtn) {
                installBtn.style.display = 'block';
                installBtn.addEventListener('click', () => this.installPWA(deferredPrompt));
            }
        });
    },

    /**
     * Install PWA
     */
    async installPWA(deferredPrompt) {
        if (deferredPrompt) {
            deferredPrompt.prompt();
            const choiceResult = await deferredPrompt.userChoice;
            
            if (choiceResult.outcome === 'accepted') {
                console.log('User accepted the install prompt');
                const installBtn = document.getElementById('install-btn');
                if (installBtn) {
                    installBtn.style.display = 'none';
                }
            }
            deferredPrompt = null;
        }
    },

    /**
     * Request notification permission
     */
    requestNotificationPermission() {
        if ('Notification' in window && Notification.permission === 'default') {
            Notification.requestPermission().then(permission => {
                console.log('Notification permission:', permission);
            });
        }
    }
};

// Form utilities
const FormUtils = {
    /**
     * Serialize form data
     */
    serialize(form) {
        const formData = new FormData(form);
        const data = {};
        for (let [key, value] of formData.entries()) {
            data[key] = value;
        }
        return data;
    },

    /**
     * Validate required fields
     */
    validateRequired(form) {
        const requiredFields = form.querySelectorAll('[required]');
        let isValid = true;
        
        requiredFields.forEach(field => {
            if (!field.value.trim()) {
                field.classList.add('is-invalid');
                isValid = false;
            } else {
                field.classList.remove('is-invalid');
            }
        });
        
        return isValid;
    }
};

// Auto-refresh functionality
const AutoRefresh = {
    intervals: new Map(),

    /**
     * Start auto-refreshing stats
     */
    startStatsRefresh(interval = 5000) {
        if (this.intervals.has('stats')) {
            clearInterval(this.intervals.get('stats'));
        }

        const refreshInterval = setInterval(async () => {
            const stats = await WorkTimeAPI.fetchStats();
            if (stats) {
                UIUtils.updateStats(stats);
            }
        }, interval);

        this.intervals.set('stats', refreshInterval);
    },

    /**
     * Stop auto-refresh
     */
    stop(name) {
        if (this.intervals.has(name)) {
            clearInterval(this.intervals.get(name));
            this.intervals.delete(name);
        }
    },

    /**
     * Stop all auto-refresh intervals
     */
    stopAll() {
        this.intervals.forEach((interval, name) => {
            clearInterval(interval);
        });
        this.intervals.clear();
    }
};

// Initialize on page load
document.addEventListener('DOMContentLoaded', () => {
    // Initialize menu functionality
    MenuUtils.initializeMenu();
    
    // Initialize PWA features
    PWAUtils.initializeInstall();
    PWAUtils.requestNotificationPermission();
    
    // Start timer updates if active timer exists
    const liveTimer = document.getElementById('live-timer');
    if (liveTimer && liveTimer.dataset.startTime) {
        TimerUtils.startTimerUpdates('live-timer', liveTimer.dataset.startTime);
    }
    
    // Start auto-refresh for stats (only if stats elements exist)
    if (document.querySelector('.stat-card')) {
        AutoRefresh.startStatsRefresh();
    }
});

// Cleanup on page unload
window.addEventListener('beforeunload', () => {
    AutoRefresh.stopAll();
});

// Export for use in other scripts
window.WorkTimeUtils = {
    API: WorkTimeAPI,
    Timer: TimerUtils,
    UI: UIUtils,
    Menu: MenuUtils,
    PWA: PWAUtils,
    Form: FormUtils,
    AutoRefresh
};