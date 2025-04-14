package com.messagingapp.service;

import com.messagingapp.dto.ChatDTO;
import com.messagingapp.dto.MessageDTO;
import com.messagingapp.model.Chat;
import com.messagingapp.model.Message;
import com.messagingapp.model.User;
import com.messagingapp.repository.ChatRepository;
import com.messagingapp.repository.MessageRepository;
import com.messagingapp.repository.UserRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

@Service
public class ChatService {

    private final ChatRepository chatRepository;
    private final MessageRepository messageRepository;
    private final UserRepository userRepository;
    private final WebSocketService webSocketService;

    public ChatService(ChatRepository chatRepository,
                      MessageRepository messageRepository,
                      UserRepository userRepository,
                      WebSocketService webSocketService) {
        this.chatRepository = chatRepository;
        this.messageRepository = messageRepository;
        this.userRepository = userRepository;
        this.webSocketService = webSocketService;
    }

    public List<ChatDTO> getUserChats(Long userId) {
        return chatRepository.findByUserId(userId).stream()
                .map(this::convertToDTO)
                .collect(Collectors.toList());
    }

    @Transactional
    public ChatDTO createChat(List<Long> participantIds, Long currentUserId) {
        Set<User> participants = new HashSet<>();
        participants.add(userRepository.findById(currentUserId)
                .orElseThrow(() -> new RuntimeException("Current user not found")));

        for (Long participantId : participantIds) {
            participants.add(userRepository.findById(participantId)
                    .orElseThrow(() -> new RuntimeException("User not found: " + participantId)));
        }

        Chat chat = new Chat();
        chat.setName(generateChatName(participants));
        chat.setParticipants(participants);
        
        return convertToDTO(chatRepository.save(chat));
    }

    public List<MessageDTO> getChatMessages(Long chatId, Long userId) {
        validateChatAccess(chatId, userId);
        return messageRepository.findByChatIdOrderByTimestamp(chatId).stream()
                .map(this::convertToDTO)
                .collect(Collectors.toList());
    }

    @Transactional
    public MessageDTO sendMessage(Long chatId, MessageDTO messageDTO, Long senderId) {
        Chat chat = validateChatAccess(chatId, senderId);
        User sender = userRepository.findById(senderId)
                .orElseThrow(() -> new RuntimeException("Sender not found"));

        Message message = new Message();
        message.setChat(chat);
        message.setSender(sender);
        message.setContent(messageDTO.getContent());
        
        Message savedMessage = messageRepository.save(message);
        MessageDTO savedMessageDTO = convertToDTO(savedMessage);
        
        // Notify all participants through WebSocket
        webSocketService.notifyMessageReceived(savedMessageDTO);
        
        return savedMessageDTO;
    }

    @Transactional
    public void updateMessageStatus(Long messageId, String status, Long userId) {
        Message message = messageRepository.findById(messageId)
                .orElseThrow(() -> new RuntimeException("Message not found"));

        validateChatAccess(message.getChat().getId(), userId);
        message.setStatus(Message.MessageStatus.valueOf(status.toUpperCase()));
        messageRepository.save(message);
        
        // Notify message status update through WebSocket
        webSocketService.notifyMessageStatusUpdated(convertToDTO(message));
    }

    private Chat validateChatAccess(Long chatId, Long userId) {
        return chatRepository.findById(chatId)
                .filter(chat -> chat.getParticipants().stream()
                        .anyMatch(participant -> participant.getId().equals(userId)))
                .orElseThrow(() -> new RuntimeException("Chat not found or access denied"));
    }

    private ChatDTO convertToDTO(Chat chat) {
        ChatDTO dto = new ChatDTO();
        dto.setId(chat.getId());
        dto.setName(chat.getName());
        dto.setParticipants(chat.getParticipants().stream()
                .map(this::convertToUserDTO)
                .collect(Collectors.toList()));
        dto.setLastMessageAt(chat.getLastMessageAt());
        return dto;
    }

    private MessageDTO convertToDTO(Message message) {
        MessageDTO dto = new MessageDTO();
        dto.setId(message.getId());
        dto.setChatId(message.getChat().getId());
        dto.setSenderId(message.getSender().getId());
        dto.setContent(message.getContent());
        dto.setTimestamp(message.getTimestamp());
        dto.setStatus(message.getStatus().name());
        return dto;
    }

    private UserDTO convertToUserDTO(User user) {
        UserDTO dto = new UserDTO();
        dto.setId(user.getId());
        dto.setUsername(user.getUsername());
        dto.setProfilePicture(user.getProfilePicture());
        dto.setStatus(user.getStatus());
        dto.setOnline(user.isOnline());
        return dto;
    }

    private String generateChatName(Set<User> participants) {
        if (participants.size() == 2) {
            return participants.stream()
                    .map(User::getUsername)
                    .collect(Collectors.joining(", "));
        }
        return "Group Chat";
    }
} 