//! Обработчики для работы с клетками

use axum::extract::State;
use axum::Json;

use crate::{AppState, errors::ApiError, errors::lock_arc_mutex};

/// Получить список всех клеток
pub async fn get_cells(State(state): State<AppState>) -> Result<Json<Vec<crate::CellResponse>>, ApiError> {
    let stem = lock_arc_mutex(&state.stem)?;
    let cells: Vec<crate::CellResponse> = stem
        .cells()
        .values()
        .map(|cell| crate::CellResponse {
            id: cell.id.clone(),
            role: format!("{:?}", cell.role),
            generation: cell.generation,
            age_ms: cell.age_millis(),
            activity: cell.activity,
        })
        .collect();
    Ok(Json(cells))
}

/// Получить распределение ролей
pub async fn get_distribution(State(state): State<AppState>) -> Result<Json<crate::DistributionResponse>, ApiError> {
    use soma_core::CellRole;
    
    let stem = lock_arc_mutex(&state.stem)?;
    let dist = stem.role_distribution();

    Ok(Json(crate::DistributionResponse {
        sensor: *dist.get(&CellRole::Sensor).unwrap_or(&0),
        logic: *dist.get(&CellRole::Logic).unwrap_or(&0),
        motor: *dist.get(&CellRole::Motor).unwrap_or(&0),
        total: stem.cell_count(),
    }))
}

