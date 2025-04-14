package com.messagingapp.controller;

import com.messagingapp.dto.ChatDTO;
import com.messagingapp.dto.MessageDTO;
import com.messagingapp.service.ChatService;
import org.springframework.http.ResponseEntity;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.web.bind.annotation.*;
import com.messagingapp.security.UserPrincipal;

import javax.validation.Valid;
import java.util.List;

@RestController
@RequestMapping("/api/chats")
public class ChatController {

    private final ChatService chatService;

    public ChatController(ChatService chatService) {
        this.chatService = chatService;
    }

    @GetMapping
    public ResponseEntity<List<ChatDTO>> getUserChats(@AuthenticationPrincipal UserPrincipal currentUser) {
        return ResponseEntity.ok(chatService.getUserChats(currentUser.getId()));
    }

    @PostMapping
    public ResponseEntity<ChatDTO> createChat(@RequestBody List<Long> participantIds,
                                            @AuthenticationPrincipal UserPrincipal currentUser) {
        return ResponseEntity.ok(chatService.createChat(participantIds, currentUser.getId()));
    }

    @GetMapping("/{chatId}/messages")
    public ResponseEntity<List<MessageDTO>> getChatMessages(@PathVariable Long chatId,
                                                          @AuthenticationPrincipal UserPrincipal currentUser) {
        return ResponseEntity.ok(chatService.getChatMessages(chatId, currentUser.getId()));
    }

    @PostMapping("/{chatId}/messages")
    public ResponseEntity<MessageDTO> sendMessage(@PathVariable Long chatId,
                                                @Valid @RequestBody MessageDTO messageDTO,
                                                @AuthenticationPrincipal UserPrincipal currentUser) {
        return ResponseEntity.ok(chatService.sendMessage(chatId, messageDTO, currentUser.getId()));
    }

    @PutMapping("/messages/{messageId}/status")
    public ResponseEntity<Void> updateMessageStatus(@PathVariable Long messageId,
                                                  @RequestParam String status,
                                                  @AuthenticationPrincipal UserPrincipal currentUser) {
        chatService.updateMessageStatus(messageId, status, currentUser.getId());
        return ResponseEntity.ok().build();
    }
} 