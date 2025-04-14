package com.messagingapp.websocket;

import org.springframework.web.socket.TextMessage;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.handler.TextWebSocketHandler;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

public class WebSocketHandler extends TextWebSocketHandler {
    private final ObjectMapper objectMapper = new ObjectMapper();
    private final Map<String, WebSocketSession> sessions = new ConcurrentHashMap<>();

    @Override
    public void afterConnectionEstablished(WebSocketSession session) {
        String userId = extractUserId(session);
        sessions.put(userId, session);
    }

    @Override
    public void handleTextMessage(WebSocketSession session, TextMessage message) {
        try {
            MessageDTO messageDTO = objectMapper.readValue(message.getPayload(), MessageDTO.class);
            handleMessage(messageDTO, session);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    @Override
    public void afterConnectionClosed(WebSocketSession session, org.springframework.web.socket.CloseStatus status) {
        String userId = extractUserId(session);
        sessions.remove(userId);
    }

    private void handleMessage(MessageDTO messageDTO, WebSocketSession senderSession) {
        switch (messageDTO.getType()) {
            case "message":
                handleChatMessage(messageDTO);
                break;
            case "status":
                handleStatusUpdate(messageDTO);
                break;
            default:
                System.out.println("Unknown message type: " + messageDTO.getType());
        }
    }

    private void handleChatMessage(MessageDTO messageDTO) {
        // Broadcast message to all participants in the chat
        String chatId = messageDTO.getChatId();
        // TODO: Implement chat message handling logic
    }

    private void handleStatusUpdate(MessageDTO messageDTO) {
        // Handle user status updates
        String userId = messageDTO.getSenderId();
        // TODO: Implement status update handling logic
    }

    private String extractUserId(WebSocketSession session) {
        // Extract user ID from session attributes or headers
        // TODO: Implement user ID extraction logic
        return "default-user-id";
    }

    private void sendMessageToUser(String userId, MessageDTO message) {
        WebSocketSession session = sessions.get(userId);
        if (session != null && session.isOpen()) {
            try {
                String messageJson = objectMapper.writeValueAsString(message);
                session.sendMessage(new TextMessage(messageJson));
            } catch (Exception e) {
                e.printStackTrace();
            }
        }
    }
} 