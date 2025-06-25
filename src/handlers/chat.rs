if is_participant != Some(true) {
    return Err(AppError::Forbidden("Not a participant in this chat".into()));
} 