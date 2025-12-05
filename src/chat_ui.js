// Agent Chat UI - Injected into web pages for user feedback
(function() {
  'use strict';

  // Prevent multiple injections
  if (window.__ROBERT_CHAT_UI_INJECTED__) {
    return;
  }
  window.__ROBERT_CHAT_UI_INJECTED__ = true;

  // Create the chat UI container
  const chatContainer = document.createElement('div');
  chatContainer.id = 'robert-chat-container';
  chatContainer.innerHTML = `
    <div id="robert-chat-sidebar">
      <div id="robert-chat-header">
        <h3>Agent Chat</h3>
        <button id="robert-chat-toggle" aria-label="Toggle chat">−</button>
      </div>
      <div id="robert-chat-messages"></div>
      <div id="robert-chat-input-area">
        <textarea id="robert-chat-input" placeholder="Send feedback to the agent..." rows="3"></textarea>
        <button id="robert-chat-send">Send</button>
      </div>
    </div>
  `;

  // Apply styles
  const styles = document.createElement('style');
  styles.textContent = `
    #robert-chat-container {
      --primary-color: #4a90e2;
      --bg-color: #ffffff;
      --border-color: #e0e0e0;
      --text-color: #333333;
      --shadow: 0 2px 10px rgba(0,0,0,0.1);
      position: fixed;
      top: 0;
      right: 0;
      width: 350px;
      height: 100vh;
      z-index: 2147483647;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      font-size: 14px;
      box-sizing: border-box;
    }

    #robert-chat-sidebar {
      background: var(--bg-color);
      border-left: 1px solid var(--border-color);
      box-shadow: -2px 0 10px rgba(0,0,0,0.05);
      display: flex;
      flex-direction: column;
      height: 100%;
      transition: transform 0.3s ease;
    }

    #robert-chat-sidebar.collapsed {
      transform: translateX(100%);
    }

    #robert-chat-header {
      background: var(--primary-color);
      color: white;
      padding: 15px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-shrink: 0;
    }

    #robert-chat-header h3 {
      margin: 0;
      font-size: 16px;
      font-weight: 600;
    }

    #robert-chat-toggle {
      background: transparent;
      border: none;
      color: white;
      font-size: 20px;
      cursor: pointer;
      padding: 0;
      width: 24px;
      height: 24px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 4px;
      transition: background 0.2s;
    }

    #robert-chat-toggle:hover {
      background: rgba(255,255,255,0.2);
    }

    #robert-chat-messages {
      flex: 1;
      overflow-y: auto;
      padding: 15px;
      display: flex;
      flex-direction: column;
      gap: 10px;
    }

    .robert-chat-message {
      padding: 10px 12px;
      border-radius: 8px;
      max-width: 85%;
      word-wrap: break-word;
      animation: slideIn 0.2s ease;
    }

    @keyframes slideIn {
      from {
        opacity: 0;
        transform: translateY(10px);
      }
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }

    .robert-chat-message.user {
      background: var(--primary-color);
      color: white;
      align-self: flex-end;
      margin-left: auto;
    }

    .robert-chat-message.agent {
      background: #f5f5f5;
      color: var(--text-color);
      align-self: flex-start;
    }

    .robert-chat-message .robert-chat-timestamp {
      font-size: 11px;
      opacity: 0.7;
      margin-top: 4px;
    }

    #robert-chat-input-area {
      padding: 15px;
      border-top: 1px solid var(--border-color);
      flex-shrink: 0;
    }

    #robert-chat-input {
      width: 100%;
      border: 1px solid var(--border-color);
      border-radius: 6px;
      padding: 10px;
      font-size: 14px;
      resize: none;
      font-family: inherit;
      box-sizing: border-box;
      margin-bottom: 8px;
    }

    #robert-chat-input:focus {
      outline: none;
      border-color: var(--primary-color);
      box-shadow: 0 0 0 2px rgba(74, 144, 226, 0.1);
    }

    #robert-chat-send {
      width: 100%;
      background: var(--primary-color);
      color: white;
      border: none;
      border-radius: 6px;
      padding: 10px;
      font-size: 14px;
      font-weight: 600;
      cursor: pointer;
      transition: background 0.2s;
    }

    #robert-chat-send:hover {
      background: #357ab8;
    }

    #robert-chat-send:active {
      transform: scale(0.98);
    }

    /* Collapse button when sidebar is collapsed */
    #robert-chat-container.collapsed #robert-chat-toggle {
      position: fixed;
      right: 10px;
      top: 10px;
      background: var(--primary-color);
      box-shadow: var(--shadow);
      width: 40px;
      height: 40px;
      border-radius: 50%;
    }

    /* Scrollbar styling */
    #robert-chat-messages::-webkit-scrollbar {
      width: 6px;
    }

    #robert-chat-messages::-webkit-scrollbar-track {
      background: transparent;
    }

    #robert-chat-messages::-webkit-scrollbar-thumb {
      background: #ccc;
      border-radius: 3px;
    }

    #robert-chat-messages::-webkit-scrollbar-thumb:hover {
      background: #aaa;
    }

    /* Feedback buttons */
    .robert-feedback-buttons {
      display: flex;
      gap: 8px;
      justify-content: flex-end;
      padding: 8px 15px;
      animation: slideIn 0.2s ease;
    }

    .feedback-btn {
      background: white;
      border: 1px solid var(--border-color);
      border-radius: 6px;
      padding: 6px 12px;
      font-size: 16px;
      cursor: pointer;
      transition: all 0.2s;
    }

    .feedback-btn:hover {
      transform: scale(1.1);
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .feedback-btn.thumbs-up:hover {
      background: #e8f5e9;
      border-color: #4caf50;
    }

    .feedback-btn.thumbs-down:hover {
      background: #ffebee;
      border-color: #f44336;
    }
  `;

  // Inject UI and styles into the page
  document.head.appendChild(styles);
  document.body.appendChild(chatContainer);

  // Chat state
  const chatState = {
    messages: [],
    collapsed: false
  };

  // Get DOM elements
  const sidebar = document.getElementById('robert-chat-sidebar');
  const messagesContainer = document.getElementById('robert-chat-messages');
  const inputArea = document.getElementById('robert-chat-input');
  const sendButton = document.getElementById('robert-chat-send');
  const toggleButton = document.getElementById('robert-chat-toggle');

  // Add a message to the chat
  function addMessage(text, sender = 'agent') {
    const messageDiv = document.createElement('div');
    messageDiv.className = `robert-chat-message ${sender}`;

    const timestamp = new Date().toLocaleTimeString();
    messageDiv.innerHTML = `
      <div class="robert-chat-content">${escapeHtml(text)}</div>
      <div class="robert-chat-timestamp">${timestamp}</div>
    `;

    messagesContainer.appendChild(messageDiv);
    messagesContainer.scrollTop = messagesContainer.scrollHeight;

    chatState.messages.push({
      text,
      sender,
      timestamp: Date.now()
    });

    // Store message for retrieval by the agent
    storeMessage(text, sender);
  }

  // Escape HTML to prevent XSS
  function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  // Store message in a way the agent can retrieve it
  function storeMessage(text, sender) {
    if (!window.__ROBERT_CHAT_MESSAGES__) {
      window.__ROBERT_CHAT_MESSAGES__ = [];
    }
    window.__ROBERT_CHAT_MESSAGES__.push({
      text,
      sender,
      timestamp: Date.now()
    });
  }

  // Send a message
  function sendMessage() {
    const text = inputArea.value.trim();
    if (!text) return;

    addMessage(text, 'user');
    inputArea.value = '';

    // Trigger custom event that the agent can listen to
    window.dispatchEvent(new CustomEvent('robert-chat-message', {
      detail: { text, sender: 'user', timestamp: Date.now() }
    }));

    // Mark as unprocessed for the agent to pick up
    if (!window.__ROBERT_UNPROCESSED_MESSAGES__) {
      window.__ROBERT_UNPROCESSED_MESSAGES__ = [];
    }
    window.__ROBERT_UNPROCESSED_MESSAGES__.push({
      text,
      sender: 'user',
      timestamp: Date.now()
    });

    // Show a message that it's waiting for processing
    addMessage('Message received. Waiting for agent to process...', 'agent');
  }

  // Add feedback buttons for an action
  function addFeedbackButtons(actionId, originalRequest, agentName, errorDescription) {
    const feedbackDiv = document.createElement('div');
    feedbackDiv.className = 'robert-feedback-buttons';
    feedbackDiv.innerHTML = `
      <button class="feedback-btn thumbs-up" data-action-id="${actionId}">👍</button>
      <button class="feedback-btn thumbs-down" data-action-id="${actionId}">👎</button>
    `;

    messagesContainer.appendChild(feedbackDiv);

    // Add event listeners
    feedbackDiv.querySelector('.thumbs-up').addEventListener('click', () => {
      submitFeedback(actionId, true, originalRequest, agentName);
      feedbackDiv.remove();
    });

    feedbackDiv.querySelector('.thumbs-down').addEventListener('click', () => {
      const comment = prompt('What went wrong? (optional)');
      submitFeedback(actionId, false, originalRequest, agentName, comment, errorDescription);
      feedbackDiv.remove();
    });
  }

  // Submit feedback - store for agent to retrieve
  function submitFeedback(actionId, positive, originalRequest, agentName, comment, errorDescription) {
    if (!window.__ROBERT_FEEDBACK__) {
      window.__ROBERT_FEEDBACK__ = [];
    }

    const feedback = {
      actionId: actionId,
      positive: positive,
      comment: comment || null,
      agentName: agentName,
      originalRequest: originalRequest,
      errorDescription: errorDescription || null,
      timestamp: Date.now()
    };

    window.__ROBERT_FEEDBACK__.push(feedback);
    addMessage(positive ? 'Thank you for your feedback! 👍' : 'Feedback noted. The agent will learn from this. 👎', 'agent');
  }

  // Toggle sidebar collapse
  function toggleSidebar() {
    chatState.collapsed = !chatState.collapsed;
    if (chatState.collapsed) {
      sidebar.classList.add('collapsed');
      toggleButton.textContent = '+';
    } else {
      sidebar.classList.remove('collapsed');
      toggleButton.textContent = '−';
    }
  }

  // Event listeners
  sendButton.addEventListener('click', sendMessage);
  inputArea.addEventListener('keypress', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  });
  toggleButton.addEventListener('click', toggleSidebar);

  // Expose API for the agent to send messages to the chat
  window.__ROBERT_CHAT_API__ = {
    sendMessage: (text) => addMessage(text, 'agent'),
    getMessages: () => window.__ROBERT_CHAT_MESSAGES__ || [],
    clearMessages: () => {
      messagesContainer.innerHTML = '';
      chatState.messages = [];
      window.__ROBERT_CHAT_MESSAGES__ = [];
    },
    collapse: () => {
      if (!chatState.collapsed) toggleSidebar();
    },
    expand: () => {
      if (chatState.collapsed) toggleSidebar();
    }
  };

  // Send welcome message
  addMessage('Chat UI loaded. You can provide feedback to the agent here.', 'agent');

  console.log('[Robert Chat UI] Injected successfully');
})();
