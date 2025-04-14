class MessagingApp {
    constructor() {
        this.currentUser = null;
        this.currentChat = null;
        this.wsConnection = null;
        this.initializeWebSocket();
        this.initializeEventListeners();
    }

    initializeWebSocket() {
        // Connect to WebSocket server
        this.wsConnection = new WebSocket('ws://localhost:8080/ws');
        
        this.wsConnection.onopen = () => {
            console.log('WebSocket connection established');
        };

        this.wsConnection.onmessage = (event) => {
            const message = JSON.parse(event.data);
            this.handleIncomingMessage(message);
        };

        this.wsConnection.onclose = () => {
            console.log('WebSocket connection closed');
            // Attempt to reconnect after 5 seconds
            setTimeout(() => this.initializeWebSocket(), 5000);
        };
    }

    initializeEventListeners() {
        // Message input handling
        const messageInput = document.querySelector('.message-input textarea');
        const sendButton = document.querySelector('.send-button');

        messageInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                this.sendMessage();
            }
        });

        sendButton.addEventListener('click', () => this.sendMessage());

        // Contact search
        const searchInput = document.querySelector('.search-box input');
        searchInput.addEventListener('input', (e) => {
            this.filterContacts(e.target.value);
        });
    }

    async sendMessage() {
        const messageInput = document.querySelector('.message-input textarea');
        const message = messageInput.value.trim();
        
        if (!message || !this.currentChat) return;

        const messageData = {
            type: 'message',
            content: message,
            chatId: this.currentChat.id,
            timestamp: new Date().toISOString()
        };

        try {
            // Send message through WebSocket
            this.wsConnection.send(JSON.stringify(messageData));
            
            // Add message to UI
            this.addMessageToUI(messageData, true);
            
            // Clear input
            messageInput.value = '';
        } catch (error) {
            console.error('Error sending message:', error);
        }
    }

    handleIncomingMessage(message) {
        switch (message.type) {
            case 'message':
                this.addMessageToUI(message, false);
                break;
            case 'status':
                this.updateMessageStatus(message);
                break;
            case 'contact':
                this.updateContactList(message);
                break;
            default:
                console.log('Unknown message type:', message.type);
        }
    }

    addMessageToUI(message, isSent) {
        const messagesContainer = document.querySelector('.messages-container');
        const messageElement = document.createElement('div');
        messageElement.className = `message ${isSent ? 'sent' : 'received'}`;
        
        const time = new Date(message.timestamp).toLocaleTimeString();
        messageElement.innerHTML = `
            <div class="message-content">${message.content}</div>
            <div class="message-time">${time}</div>
        `;
        
        messagesContainer.appendChild(messageElement);
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }

    filterContacts(searchTerm) {
        const contacts = document.querySelectorAll('.contact-item');
        searchTerm = searchTerm.toLowerCase();
        
        contacts.forEach(contact => {
            const name = contact.querySelector('.contact-name').textContent.toLowerCase();
            contact.style.display = name.includes(searchTerm) ? 'flex' : 'none';
        });
    }

    updateContactList(contactData) {
        const contactList = document.querySelector('.contact-list');
        const existingContact = contactList.querySelector(`[data-id="${contactData.id}"]`);
        
        if (existingContact) {
            // Update existing contact
            existingContact.querySelector('.contact-name').textContent = contactData.name;
            existingContact.querySelector('.last-message').textContent = contactData.lastMessage;
            existingContact.querySelector('.timestamp').textContent = contactData.timestamp;
        } else {
            // Add new contact
            const contactElement = document.createElement('div');
            contactElement.className = 'contact-item';
            contactElement.setAttribute('data-id', contactData.id);
            contactElement.innerHTML = `
                <img src="assets/default-avatar.png" alt="${contactData.name}" class="contact-pic">
                <div class="contact-info">
                    <span class="contact-name">${contactData.name}</span>
                    <span class="last-message">${contactData.lastMessage}</span>
                </div>
                <span class="timestamp">${contactData.timestamp}</span>
            `;
            
            contactElement.addEventListener('click', () => this.selectChat(contactData.id));
            contactList.appendChild(contactElement);
        }
    }

    selectChat(chatId) {
        this.currentChat = { id: chatId };
        // Update UI to show selected chat
        document.querySelectorAll('.contact-item').forEach(item => {
            item.classList.toggle('active', item.getAttribute('data-id') === chatId);
        });
        
        // Load chat messages
        this.loadChatMessages(chatId);
    }

    async loadChatMessages(chatId) {
        try {
            const response = await fetch(`/api/chats/${chatId}/messages`);
            const messages = await response.json();
            
            const messagesContainer = document.querySelector('.messages-container');
            messagesContainer.innerHTML = '';
            
            messages.forEach(message => {
                this.addMessageToUI(message, message.senderId === this.currentUser.id);
            });
        } catch (error) {
            console.error('Error loading messages:', error);
        }
    }
}

// Initialize the app when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.messagingApp = new MessagingApp();
}); 