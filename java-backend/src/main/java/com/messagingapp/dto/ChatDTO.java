package com.messagingapp.dto;

import lombok.Data;
import java.time.LocalDateTime;
import java.util.List;

@Data
public class ChatDTO {
    private Long id;
    private String name;
    private List<UserDTO> participants;
    private LocalDateTime lastMessageAt;
    private MessageDTO lastMessage;
} 