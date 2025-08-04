// Superior 6 Frontend JavaScript

document.addEventListener('DOMContentLoaded', function() {
    // Initialize all functionality
    initPredictionForm();
    initCountdown();
    initFormValidation();
    initMobileMenu();
});

// Prediction form functionality
function initPredictionForm() {
    const predictionForm = document.getElementById('prediction-form');
    if (!predictionForm) return;

    // Validate score inputs
    const scoreInputs = predictionForm.querySelectorAll('.score-input');
    scoreInputs.forEach(input => {
        input.addEventListener('input', function() {
            validateScoreInput(this);
        });

        input.addEventListener('blur', function() {
            formatScoreInput(this);
        });
    });

    // Form submission
    predictionForm.addEventListener('submit', function(e) {
        if (!validatePredictionForm()) {
            e.preventDefault();
            showError('Please check your predictions and try again.');
        }
    });
}

// Validate individual score input
function validateScoreInput(input) {
    const value = parseInt(input.value);
    const isValid = !isNaN(value) && value >= 0 && value <= 20;

    input.classList.toggle('error-input', !isValid);
    input.classList.toggle('success-input', isValid);

    return isValid;
}

// Format score input (ensure it's a valid number)
function formatScoreInput(input) {
    const value = parseInt(input.value);
    if (!isNaN(value) && value >= 0 && value <= 20) {
        input.value = value;
    } else {
        input.value = '';
    }
}

// Validate entire prediction form
function validatePredictionForm() {
    const scoreInputs = document.querySelectorAll('.score-input');
    let isValid = true;

    scoreInputs.forEach(input => {
        if (!validateScoreInput(input)) {
            isValid = false;
        }
    });

    return isValid && scoreInputs.length > 0;
}

// Countdown timer for deadline
function initCountdown() {
    const countdownElement = document.getElementById('deadline-countdown');
    if (!countdownElement) return;

    const deadline = new Date(countdownElement.dataset.deadline);

    function updateCountdown() {
        const now = new Date();
        const timeLeft = deadline - now;

        if (timeLeft <= 0) {
            countdownElement.innerHTML = '<span class="text-red-600 font-bold">DEADLINE PASSED</span>';
            disablePredictionForm();
            return;
        }

        const days = Math.floor(timeLeft / (1000 * 60 * 60 * 24));
        const hours = Math.floor((timeLeft % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
        const minutes = Math.floor((timeLeft % (1000 * 60 * 60)) / (1000 * 60));
        const seconds = Math.floor((timeLeft % (1000 * 60)) / 1000);

        let countdownText = '';
        if (days > 0) {
            countdownText = `${days}d ${hours}h ${minutes}m`;
        } else if (hours > 0) {
            countdownText = `${hours}h ${minutes}m ${seconds}s`;
        } else {
            countdownText = `${minutes}m ${seconds}s`;
        }

        countdownElement.innerHTML = `<span class="font-mono text-lg ${timeLeft < 3600000 ? 'text-red-600' : 'text-blue-600'}">${countdownText}</span>`;
    }

    updateCountdown();
    setInterval(updateCountdown, 1000);
}

// Disable prediction form when deadline passed
function disablePredictionForm() {
    const form = document.getElementById('prediction-form');
    if (!form) return;

    const inputs = form.querySelectorAll('input, button');
    inputs.forEach(input => {
        input.disabled = true;
    });

    form.classList.add('opacity-50');
}

// General form validation
function initFormValidation() {
    const forms = document.querySelectorAll('form[data-validate]');

    forms.forEach(form => {
        form.addEventListener('submit', function(e) {
            if (!validateForm(this)) {
                e.preventDefault();
            }
        });
    });
}

// Validate form based on HTML5 constraints
function validateForm(form) {
    const inputs = form.querySelectorAll('input[required], select[required], textarea[required]');
    let isValid = true;

    inputs.forEach(input => {
        if (!input.checkValidity()) {
            input.classList.add('error-input');
            isValid = false;
        } else {
            input.classList.remove('error-input');
        }
    });

    return isValid;
}

// Mobile menu functionality
function initMobileMenu() {
    const menuButton = document.getElementById('mobile-menu-button');
    const mobileMenu = document.getElementById('mobile-menu');

    if (!menuButton || !mobileMenu) return;

    menuButton.addEventListener('click', function() {
        const isOpen = mobileMenu.classList.contains('hidden');

        if (isOpen) {
            mobileMenu.classList.remove('hidden');
            menuButton.setAttribute('aria-expanded', 'true');
        } else {
            mobileMenu.classList.add('hidden');
            menuButton.setAttribute('aria-expanded', 'false');
        }
    });

    // Close menu when clicking outside
    document.addEventListener('click', function(e) {
        if (!menuButton.contains(e.target) && !mobileMenu.contains(e.target)) {
            mobileMenu.classList.add('hidden');
            menuButton.setAttribute('aria-expanded', 'false');
        }
    });
}

// Utility functions
function showSuccess(message) {
    showNotification(message, 'success');
}

function showError(message) {
    showNotification(message, 'error');
}

function showNotification(message, type = 'info') {
    // Create notification element
    const notification = document.createElement('div');
    notification.className = `fixed top-4 right-4 p-4 rounded-lg shadow-lg z-50 ${getNotificationClasses(type)}`;
    notification.textContent = message;

    // Add to page
    document.body.appendChild(notification);

    // Auto remove after 5 seconds
    setTimeout(() => {
        notification.remove();
    }, 5000);

    // Add click to dismiss
    notification.addEventListener('click', () => {
        notification.remove();
    });
}

function getNotificationClasses(type) {
    switch (type) {
        case 'success':
            return 'bg-green-500 text-white';
        case 'error':
            return 'bg-red-500 text-white';
        case 'warning':
            return 'bg-yellow-500 text-white';
        default:
            return 'bg-blue-500 text-white';
    }
}

// AJAX utility for future use
function makeRequest(url, method = 'GET', data = null) {
    return fetch(url, {
        method: method,
        headers: {
            'Content-Type': 'application/json',
            'X-Requested-With': 'XMLHttpRequest'
        },
        body: data ? JSON.stringify(data) : null
    })
        .then(response => {
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            return response.json();
        })
        .catch(error => {
            console.error('Request failed:', error);
            showError('Something went wrong. Please try again.');
            throw error;
        });
}

// Export for potential module use
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        validateScoreInput,
        validatePredictionForm,
        showSuccess,
        showError,
        makeRequest
    };
}