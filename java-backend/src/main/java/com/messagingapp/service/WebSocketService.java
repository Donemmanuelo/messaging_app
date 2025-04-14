package com.messagingapp.service;

import com.messagingapp.dto.MessageDTO;
import org.springframework.messaging.simp.SimpMessagingTemplate;
import org.springframework.stereotype.Service;

@Service
public class WebSocketService {

    private final SimpMessagingTemplate messagingTemplate;

    public WebSocketService(SimpMessagingTemplate messagingTemplate) {
        this.messagingTemplate = messagingTemplate;
    }

    public void notifyMessageReceived(MessageDTO message) {
        messagingTemplate.convertAndSend("/topic/chat/" + message.getChatId(), message);
    }

    public void notifyMessageStatusUpdated(MessageDTO message) {
        messagingTemplate.convertAndSend("/topic/chat/" + message.getChatId() + "/status", message);
    }

    public void notifyUserStatusChanged(Long userId, boolean isOnline) {
        messagingTemplate.convertAndSend("/topic/users/" + userId + "/status", isOnline);
    }
} 