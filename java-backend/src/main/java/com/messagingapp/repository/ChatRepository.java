package com.messagingapp.repository;

import com.messagingapp.model.Chat;
import com.messagingapp.model.User;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import java.util.List;
import java.util.Optional;

public interface ChatRepository extends JpaRepository<Chat, Long> {
    @Query("SELECT c FROM Chat c JOIN c.participants p WHERE p.id = :userId")
    List<Chat> findByUserId(@Param("userId") Long userId);

    @Query("SELECT c FROM Chat c JOIN c.participants p1 JOIN c.participants p2 " +
           "WHERE p1.id = :userId1 AND p2.id = :userId2 AND c.participants.size = 2")
    Optional<Chat> findDirectChat(@Param("userId1") Long userId1, @Param("userId2") Long userId2);
} 