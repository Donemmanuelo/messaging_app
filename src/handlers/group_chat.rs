Ok(Json::<Vec<MyType>>(vec![]))

Json(_req): Json<UpdateGroupRequest>,

State(_state): State<Arc<AppState>>,

_auth_user: AuthUser,

Path(_group_id): Path<Uuid>, 