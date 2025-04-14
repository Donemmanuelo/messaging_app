package com.messagingapp.dto;

import lombok.Data;
import java.time.LocalDateTime;

@Data
public class MessageDTO {
    private String type;
    private String content;
    private String senderId;
    private String chatId;
    private LocalDateTime timestamp;
    private String status;
} 