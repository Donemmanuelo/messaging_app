package com.messagingapp.dto;

import lombok.Data;

@Data
public class UserDTO {
    private Long id;
    private String username;
    private String email;
    private String profilePicture;
    private String status;
    private boolean isOnline;
} 