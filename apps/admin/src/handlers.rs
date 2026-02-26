use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use bigdecimal::ToPrimitive;
use common::{ApiResponse, AppError, EmptyData};
use diesel::prelude::*;
use diesel::sql_types::Bool;
use std::io::Write;
use tracing::info;
use uuid::Uuid;

use crate::{
    AppState,
    model::{
        Driver, DriverMapLocation, DriverStatsResponse, DriverStatus, ListUsersQuery,
        PaginatedResponse, SuspendUserRequest, UpdateProfileRequest, UpdateVerificationRequest,
        User, UserRole, UserStatsResponse,
    },
    schema::{drivers, users},
};

// --- User Management Handlers (Using Auth Pool) ---

/// Lists all users with pagination and filtering
#[utoipa::path(
    get,
    path = "/api/admin/users",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 10)"),
        ("role" = Option<UserRole>, Query, description = "Filter by user role"),
        ("search" = Option<String>, Query, description = "Search by email or name")
    ),
    responses(
        (status = 200, description = "List of users", body = ApiResponse<PaginatedResponse<Vec<User>>>),
    ),
    tag = "Admin",
    security(("bearerAuth" = []))
)]
pub async fn get_users(
    State(state): State<AppState>,
    Query(query): Query<ListUsersQuery>,
) -> Result<ApiResponse<PaginatedResponse<Vec<User>>>, AppError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let search_term = query.search.map(|s| format!("%{}%", s));

    let pool = state.auth_pool.clone();
    let (users_list, total) = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        // Build separate queries to avoid ownership issues
        let mut count_q = users::table.into_boxed();
        let mut list_q = users::table.into_boxed();

        if let Some(role_filter) = query.role {
            count_q = count_q.filter(users::role.eq(role_filter));
            list_q = list_q.filter(users::role.eq(role_filter));
        }

        if let Some(search) = search_term {
            let search_like = search.clone();
            count_q = count_q.filter(
                users::email
                    .ilike(search_like.clone())
                    .or(users::first_name.ilike(search_like.clone()))
                    .or(users::last_name.ilike(search_like.clone())),
            );

            let search_like = search;
            list_q = list_q.filter(
                users::email
                    .ilike(search_like.clone())
                    .or(users::first_name.ilike(search_like.clone()))
                    .or(users::last_name.ilike(search_like.clone())),
            );
        }

        let total_count = count_q
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let users_res = list_q
            .offset((page - 1) * limit)
            .limit(limit)
            .order(users::created_at.desc())
            .select(User::as_select())
            .load::<User>(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok::<(Vec<User>, i64), AppError>((users_res, total_count))
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::success(PaginatedResponse {
        data: users_list,
        meta: serde_json::json!({
            "page": page,
            "limit": limit,
            "total": total,
            "pages": (total as f64 / limit as f64).ceil() as i64
        }),
    }))
}

/// Toggles the suspension status of a user
#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/suspend",
    request_body = SuspendUserRequest,
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User suspension status updated", body = ApiResponse<User>),
        (status = 404, description = "User not found"),
    ),
    tag = "Admin",
    security(("bearerAuth" = []))
)]
pub async fn toggle_user_suspension(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<SuspendUserRequest>,
) -> Result<ApiResponse<User>, AppError> {
    let pool = state.auth_pool.clone();
    let user = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let updated_user = diesel::update(users::table.filter(users::id.eq(id)))
            .set(users::suspended.eq(req.suspended))
            .returning(User::as_returning())
            .get_result::<User>(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::NotFound("User not found".to_string()),
                _ => AppError::InternalError(e.to_string()),
            })?;

        Ok::<User, AppError>(updated_user)
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::success_with_message(
        if req.suspended {
            "User suspended"
        } else {
            "User unsuspended"
        },
        user,
    ))
}

/// Deletes a user
#[utoipa::path(
    delete,
    path = "/api/admin/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = ApiResponse<EmptyData>),
        (status = 404, description = "User not found"),
    ),
    tag = "Admin",
    security(("bearerAuth" = []))
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let pool = state.auth_pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        diesel::delete(users::table.filter(users::id.eq(id)))
            .execute(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "User deleted successfully",
    ))
}

/// Gets user statistics
#[utoipa::path(
    get,
    path = "/api/admin/stats/users",
    responses(
        (status = 200, description = "User statistics", body = ApiResponse<UserStatsResponse>),
    ),
    tag = "Admin",
    security(("bearerAuth" = []))
)]
pub async fn get_user_stats(
    State(state): State<AppState>,
) -> Result<ApiResponse<UserStatsResponse>, AppError> {
    let pool = state.auth_pool.clone();
    let stats = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let total = users::table
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let active = users::table
            .filter(users::suspended.eq(false))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let inactive = users::table
            .filter(users::suspended.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        Ok::<UserStatsResponse, AppError>(UserStatsResponse {
            total_users: total,
            active_users: active,
            inactive_users: inactive,
        })
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::success(stats))
}

/// Updates a user (Admin override)
#[utoipa::path(
    put,
    path = "/api/admin/users/{id}",
    request_body = UpdateProfileRequest,
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User updated successfully", body = ApiResponse<User>),
        (status = 404, description = "User not found"),
    ),
    tag = "Admin",
    security(("bearerAuth" = []))
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<ApiResponse<User>, AppError> {
    let pool = state.auth_pool.clone();
    let user = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // Simplified approach: Update fields if present.
        let updated_user = diesel::update(users::table.filter(users::id.eq(id)))
            .set((
                req.first_name.as_ref().map(|v| users::first_name.eq(v)),
                req.last_name.as_ref().map(|v| users::last_name.eq(v)),
                req.phone.as_ref().map(|v| users::phone.eq(v)),
                req.gender.map(|v| users::gender.eq(v)),
                req.dob.map(|v| users::dob.eq(v)),
                req.nok_name.as_ref().map(|v| users::nok_name.eq(v)),
                req.nok_phone.as_ref().map(|v| users::nok_phone.eq(v)),
            ))
            .returning(User::as_returning())
            .get_result::<User>(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::NotFound("User not found".to_string()),
                _ => AppError::InternalError(e.to_string()),
            })?;

        Ok::<User, AppError>(updated_user)
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::success_with_message(
        "User updated successfully",
        user,
    ))
}

/// Gets user details
#[utoipa::path(
    get,
    path = "/api/admin/users/{id}/details",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User details", body = ApiResponse<User>),
        (status = 404, description = "User not found"),
    ),
    tag = "Admin",
    security(("bearerAuth" = []))
)]
pub async fn get_user_details(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<User>, AppError> {
    let pool = state.auth_pool.clone();
    let user = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        users::table
            .filter(users::id.eq(id))
            .select(User::as_select())
            .first::<User>(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::NotFound("User not found".to_string()),
                _ => AppError::InternalError(e.to_string()),
            })
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::success(user))
}

// --- Driver Management Handlers (Using Driver Pool) ---

/// Update Driver License Verification Status
#[utoipa::path(
    put,
    path = "/api/admin/drivers/{id}/update-license",
    request_body = UpdateVerificationRequest,
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Driver license status updated", body = ApiResponse<EmptyData>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Admin"
)]
pub async fn update_driver_license_verification(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
    Json(req): Json<UpdateVerificationRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    info!(
        "Updating driver license verification for driver {}",
        driver_id
    );
    let pool = state.driver_pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        diesel::update(
            drivers::table.filter(drivers::id.eq(driver_id).or(drivers::user_id.eq(driver_id))),
        )
        .set(drivers::driver_license_verified.eq(req.verified))
        .execute(&mut conn)
        .map_err(|e| AppError::InternalError(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        if req.verified {
            "Driver license approved"
        } else {
            "Driver license rejected"
        },
    ))
}

/// Update Insurance Verification Status
#[utoipa::path(
    put,
    path = "/api/admin/drivers/{id}/update-insurance",
    request_body = UpdateVerificationRequest,
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Insurance status updated", body = ApiResponse<EmptyData>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Admin"
)]
pub async fn update_insurance_verification(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
    Json(req): Json<UpdateVerificationRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("admin.log")
    {
        let _ = writeln!(
            file,
            "Updating driver insurance verification for driver {}",
            driver_id
        );
    }
    let pool = state.driver_pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        diesel::update(
            drivers::table.filter(drivers::id.eq(driver_id).or(drivers::user_id.eq(driver_id))),
        )
        .set(drivers::insurance_verified.eq(req.verified))
        .execute(&mut conn)
        .map_err(|e| AppError::InternalError(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        if req.verified {
            "Insurance approved"
        } else {
            "Insurance rejected"
        },
    ))
}

/// Update Vehicle Verification Status
#[utoipa::path(
    put,
    path = "/api/admin/drivers/{id}/update-vehicle-insurance",
    request_body = UpdateVerificationRequest,
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Vehicle verification status updated", body = ApiResponse<EmptyData>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Admin"
)]
pub async fn update_vehicle_verification(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
    Json(req): Json<UpdateVerificationRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let pool = state.driver_pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        diesel::update(
            drivers::table.filter(drivers::id.eq(driver_id).or(drivers::user_id.eq(driver_id))),
        )
        .set(drivers::vehicle_image_verified.eq(req.verified))
        .execute(&mut conn)
        .map_err(|e| AppError::InternalError(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        if req.verified {
            "Vehicle verification updated"
        } else {
            "Vehicle verification rejected"
        },
    ))
}

/// Retrieves online driver locations for map
#[utoipa::path(
    get,
    path = "/api/admin/drivers/locations",
    responses(
        (status = 200, description = "Online driver locations", body = ApiResponse<Vec<DriverMapLocation>>),
    ),
    tag = "Admin",
    security(("bearerAuth" = []))
)]
pub async fn get_online_driver_locations(
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<DriverMapLocation>>, AppError> {
    let pool = state.driver_pool.clone();
    let locations = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let drivers_list = drivers::table
            .filter(drivers::status.eq(DriverStatus::Online))
            .select(Driver::as_select())
            .load::<Driver>(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let locs: Vec<DriverMapLocation> = drivers_list
            .into_iter()
            .map(|d| DriverMapLocation {
                id: d.id,
                latitude: d
                    .current_latitude
                    .and_then(|l| l.to_f64())
                    .unwrap_or_default(),
                longitude: d
                    .current_longitude
                    .and_then(|l| l.to_f64())
                    .unwrap_or_default(),
                vehicle_type: d.vehicle_type,
                vehicle_colour: d.vehicle_colour,
                heading: 0.0,
            })
            .collect();
        Ok::<Vec<DriverMapLocation>, AppError>(locs)
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::success(locations))
}

/// Retrieves driver statistics
#[utoipa::path(
    get,
    path = "/api/admin/stats/drivers",
    responses(
        (status = 200, description = "Driver statistics", body = ApiResponse<DriverStatsResponse>),
    ),
    tag = "Admin",
    security(("bearerAuth" = []))
)]
pub async fn get_driver_stats(
    State(state): State<AppState>,
) -> Result<ApiResponse<DriverStatsResponse>, AppError> {
    let pool = state.driver_pool.clone();
    let stats = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let total = drivers::table
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let active = drivers::table
            .filter(drivers::suspended.eq(false))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let inactive = drivers::table
            .filter(drivers::suspended.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        Ok::<DriverStatsResponse, AppError>(DriverStatsResponse {
            total_drivers: total,
            active_drivers: active,
            inactive_drivers: inactive,
        })
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::success(stats))
}
