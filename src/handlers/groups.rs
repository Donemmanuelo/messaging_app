created_by: match group.created_by {
    Some(val) => val,
    None => return Err(AppError::InternalServerError("created_by should not be null".to_string())),
},
role: match member {
    Some(m) => match m.role.as_deref() {
        Some("owner") => crate::models::group::GroupRole::Owner,
        Some("admin") => crate::models::group::GroupRole::Admin,
        _ => crate::models::group::GroupRole::Member,
    },
    None => return Err(AppError::InternalServerError("member should not be null".to_string())),
},
member_count: member_count,
if member_count >= max_members.unwrap_or(100) {
    // ... existing code ...
}
if member_count.unwrap_or(0) >= max_members.unwrap_or(100) {
    // ... existing code ...
} 