//! Unit tests for type deserialization.
//!
//! These tests run without a live Tandoor instance. They verify that the types
//! correctly handle the API response shapes we've observed in the wild, including
//! the `created_by` field that ships as a full user object rather than an integer.

use mcp_tandoor::client::{MealPlan, MealType, PaginatedResponse, Recipe};

// ── Recipe deserialization ────────────────────────────────────────────────────

#[test]
fn recipe_created_by_as_user_object() {
    let json = r#"{
        "id": 1,
        "name": "Test Recipe",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "internal": false,
        "created_by": {
            "id": 1,
            "username": "admin",
            "first_name": "Admin",
            "last_name": "",
            "display_name": "Admin",
            "is_staff": true,
            "is_superuser": true,
            "is_active": true
        }
    }"#;

    let recipe: Recipe = serde_json::from_str(json).expect("should deserialize with user object");
    assert_eq!(recipe.id, 1);
    assert_eq!(recipe.name, "Test Recipe");
    assert!(recipe.created_by.is_some());
}

#[test]
fn recipe_created_by_as_integer() {
    let json = r#"{
        "id": 2,
        "name": "Another Recipe",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "internal": false,
        "created_by": 42
    }"#;

    let recipe: Recipe = serde_json::from_str(json).expect("should deserialize with integer");
    assert_eq!(recipe.id, 2);
    assert!(recipe.created_by.is_some());
}

#[test]
fn recipe_created_by_as_null() {
    let json = r#"{
        "id": 3,
        "name": "Null Creator Recipe",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "internal": false,
        "created_by": null
    }"#;

    let recipe: Recipe = serde_json::from_str(json).expect("should deserialize with null");
    assert_eq!(recipe.id, 3);
    assert!(recipe.created_by.is_none());
}

#[test]
fn recipe_created_by_absent() {
    let json = r#"{
        "id": 4,
        "name": "No Creator Field",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "internal": false
    }"#;

    let recipe: Recipe = serde_json::from_str(json).expect("should deserialize without field");
    assert_eq!(recipe.id, 4);
    assert!(recipe.created_by.is_none());
}

// ── Keyword deserialization ───────────────────────────────────────────────────

#[test]
fn keyword_with_label_field() {
    // Some Tandoor endpoints return "label" instead of "name"
    let json = r#"{"id": 1, "label": "dinner"}"#;
    let kw: mcp_tandoor::client::Keyword =
        serde_json::from_str(json).expect("should deserialize label as name");
    assert_eq!(kw.name, "dinner");
}

#[test]
fn keyword_with_name_field() {
    let json = r#"{"id": 2, "name": "breakfast"}"#;
    let kw: mcp_tandoor::client::Keyword =
        serde_json::from_str(json).expect("should deserialize name field");
    assert_eq!(kw.name, "breakfast");
}

// ── PaginatedResponse deserialization ────────────────────────────────────────

#[test]
fn paginated_response_with_recipes() {
    let json = r#"{
        "count": 2,
        "next": null,
        "previous": null,
        "results": [
            {
                "id": 1,
                "name": "Pasta",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "internal": false,
                "created_by": {"id": 1, "username": "admin", "first_name": "", "last_name": "", "display_name": "admin", "is_staff": true, "is_superuser": false, "is_active": true}
            },
            {
                "id": 2,
                "name": "Soup",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "internal": false
            }
        ]
    }"#;

    let resp: PaginatedResponse<Recipe> =
        serde_json::from_str(json).expect("should deserialize paginated response");
    assert_eq!(resp.count, 2);
    assert_eq!(resp.results.len(), 2);
    assert_eq!(resp.results[0].name, "Pasta");
    assert_eq!(resp.results[1].name, "Soup");
}

// ── Auth token handling ───────────────────────────────────────────────────────

#[test]
fn auth_token_deserialization() {
    let json = r#"{"token": "tda_00000000_0000_0000_0000_000000000000"}"#;
    let token: mcp_tandoor::client::AuthToken =
        serde_json::from_str(json).expect("should deserialize auth token");
    assert_eq!(token.token, "tda_00000000_0000_0000_0000_000000000000");
}

#[test]
fn tandoor_client_set_token_marks_authenticated() {
    let mut client = mcp_tandoor::TandoorClient::new("http://localhost:8080".to_string());
    assert!(!client.is_authenticated());
    client.set_token("tda_test_token".to_string());
    assert!(client.is_authenticated());
    assert_eq!(client.get_token(), Some("tda_test_token"));
}

// ── MealType deserialization ──────────────────────────────────────────────────

#[test]
fn meal_type_current_tandoor_shape() {
    // Current Tandoor omits `default` and may return a null `color`
    let json = r##"{
        "count": 2,
        "next": null,
        "previous": null,
        "results": [
            {"id": 1, "name": "Breakfast", "order": 0, "time": "08:00:00", "color": "#ffc107", "created_by": 1},
            {"id": 2, "name": "Dinner", "order": 2, "time": null, "color": null, "created_by": 1}
        ]
    }"##;

    let page: PaginatedResponse<MealType> =
        serde_json::from_str(json).expect("should deserialize current meal-type response");
    assert_eq!(page.results.len(), 2);
    assert_eq!(page.results[0].time.as_deref(), Some("08:00:00"));
    assert_eq!(page.results[1].color, None);
    assert!(!page.results[1].default);
}

// ── MealPlan deserialization ──────────────────────────────────────────────────

#[test]
fn meal_plan_current_tandoor_shape() {
    // Captured from Tandoor's /api/meal-plan/: decimal servings, from_date/to_date
    // datetimes, and no date/created/updated fields
    let json = r#"{
        "id": 1, "title": "",
        "recipe": {"id": 1, "name": "Spaghetti", "description": null, "image": null, "keywords": [],
            "working_time": 0, "waiting_time": 0,
            "created_by": {"id": 1, "username": "admin", "first_name": "", "last_name": "",
                "display_name": "admin", "is_staff": true, "is_superuser": true, "is_active": true},
            "created_at": "2026-10-14T14:59:58.036237-04:00", "updated_at": "2026-10-14T14:59:58.039789-04:00",
            "internal": false, "private": false, "servings": 1, "servings_text": "", "rating": null,
            "last_cooked": null, "new": true},
        "servings": 6.0, "note": "", "note_markdown": "",
        "from_date": "2026-10-19T12:00:00-04:00", "to_date": "2026-10-20T12:00:00-04:00",
        "meal_type": {"id": 1, "name": "Dinner", "order": 0, "time": null, "color": null, "created_by": 1},
        "created_by": 1, "recipe_name": "Spaghetti", "meal_type_name": "Dinner", "shopping": false
    }"#;

    let plan: MealPlan = serde_json::from_str(json).expect("should deserialize current meal plan");
    assert_eq!(plan.servings, 6.0);
    assert_eq!(plan.date(), "2026-10-19");
    assert_eq!(plan.end_date(), "2026-10-20");
    assert_eq!(plan.recipe.map(|r| r.name).as_deref(), Some("Spaghetti"));
}

#[test]
fn create_meal_plan_request_omits_unset_fields() {
    // Tandoor rejects null title/note with "This field may not be null."
    let request = mcp_tandoor::client::types::CreateMealPlanRequest {
        recipe: Some(31),
        title: None,
        servings: 6.0,
        from_date: chrono::NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(),
        to_date: chrono::NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(),
        meal_type: 1,
        note: None,
    };
    let body = serde_json::to_value(&request).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "recipe": 31, "servings": 6.0, "from_date": "2026-10-19",
            "to_date": "2026-10-19", "meal_type": 1
        })
    );
}
