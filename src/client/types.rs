//! Type definitions for the Tandoor API.
//!
//! This module contains all the data structures used for communicating with the Tandoor API,
//! including request and response types for recipes, shopping lists, meal plans, and more.
//!
//! ## Key Types
//!
//! - [`Recipe`] - Core recipe data with ingredients, steps, and metadata
//! - [`Keyword`] - Recipe categories and tags (with custom deserializer for API inconsistencies)
//! - [`ShoppingListEntry`] - Shopping list items with food, amounts, and completion status
//! - [`MealPlan`] - Scheduled meals with recipes and serving sizes
//! - [`PaginatedResponse`] - Standard API response wrapper for lists
//!
//! ## API Compatibility
//!
//! Some types include custom serialization logic to handle inconsistencies in the Tandoor API:
//! - [`Keyword`] handles both "name" and "label" fields depending on endpoint
//! - [`Recipe`] maps "created_at"/"updated_at" to "created"/"updated" fields
//! - Many fields are optional to handle varying API response formats

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OAuth2 authentication token response from Tandoor.
///
/// Contains the Bearer token used for authenticating subsequent API requests.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    /// The Bearer token string (format: `tda_xxxxxxxx_xxxx_xxxx_xxxx_xxxxxxxxxxxx`)
    pub token: String,
}

/// Authentication request payload for obtaining tokens.
///
/// Used with the `/api-token-auth/` endpoint to exchange credentials for a Bearer token.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    /// Tandoor username
    pub username: String,
    /// Tandoor password
    pub password: String,
}

/// A complete recipe with all its components and metadata.
///
/// Represents a recipe from the Tandoor API, including ingredients, cooking steps,
/// nutritional information, and various metadata fields.
#[derive(Debug, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique recipe identifier
    pub id: i32,
    /// Recipe name/title
    pub name: String,
    /// Optional recipe description
    pub description: Option<String>,
    /// Optional cooking instructions (may be in steps instead)
    pub instructions: Option<String>,
    /// Number of servings this recipe produces
    pub servings: Option<i32>,
    /// Active cooking time in minutes
    pub working_time: Option<i32>,
    /// Passive waiting time in minutes (e.g., rising, marinating)
    pub waiting_time: Option<i32>,
    /// Recipe creation timestamp
    #[serde(rename = "created_at")]
    pub created: DateTime<Utc>,
    /// Last modification timestamp
    #[serde(rename = "updated_at")]
    pub updated: DateTime<Utc>,
    /// Whether this is an internal/system recipe
    pub internal: bool,
    /// Display setting for ingredient overview
    #[serde(default)]
    pub show_ingredient_overview: Option<bool>,
    /// Associated keywords/tags for categorization
    #[serde(default)]
    pub keywords: Vec<Keyword>,
    /// Cooking steps with ingredients and instructions
    #[serde(default)]
    pub steps: Vec<Step>,
    /// Nutritional information if calculated
    #[serde(default)]
    pub nutrition: Option<Nutrition>,
    /// Recipe image URL or path
    #[serde(default)]
    pub image: Option<String>,
    /// User who created this recipe (id or full object depending on API version)
    #[serde(default)]
    pub created_by: Option<serde_json::Value>,
    /// Original source URL if imported
    #[serde(default)]
    pub source_url: Option<String>,
    /// Additional recipe properties
    #[serde(default)]
    pub properties: Option<Vec<serde_json::Value>>,
    /// Food-specific properties
    #[serde(default)]
    pub food_properties: Option<serde_json::Value>,
    /// File system path for recipe data
    #[serde(default)]
    pub file_path: Option<String>,
    /// Human-readable servings description
    #[serde(default)]
    pub servings_text: Option<String>,
    /// User rating (1-5 stars)
    #[serde(default)]
    pub rating: Option<f64>,
    /// When this recipe was last cooked
    #[serde(default)]
    pub last_cooked: Option<DateTime<Utc>>,
    /// Whether this recipe is private to the creator
    #[serde(default)]
    pub private: Option<bool>,
    /// Users/groups this recipe is shared with
    #[serde(default)]
    pub shared: Option<Vec<serde_json::Value>>,
    /// Whether this recipe is newly added
    #[serde(default)]
    pub new: Option<bool>,
}

/// Recipe keyword/tag for categorization and filtering.
///
/// Keywords are used to categorize recipes and can be hierarchical.
/// This type includes a custom deserializer to handle API inconsistencies
/// where some endpoints return "name" and others return "label".
#[derive(Debug, Serialize)]
pub struct Keyword {
    /// Unique keyword identifier
    pub id: i32,
    /// Keyword name (primary field)
    pub name: String,
    /// Alternative label field (API compatibility)
    #[serde(default)]
    pub label: Option<String>,
    /// Optional keyword description
    #[serde(default)]
    pub description: Option<String>,
    /// Creation timestamp
    #[serde(rename = "created_at", default)]
    pub created: Option<DateTime<Utc>>,
    /// Last modification timestamp
    #[serde(rename = "updated_at", default)]
    pub updated: Option<DateTime<Utc>>,
    /// Parent keyword for hierarchical organization
    #[serde(default)]
    pub parent: Option<serde_json::Value>,
    /// Number of child keywords
    #[serde(default)]
    pub numchild: Option<i32>,
    /// Full hierarchical name path
    #[serde(default)]
    pub full_name: Option<String>,
}

/// Custom deserializer for Keyword to handle API inconsistencies.
///
/// The Tandoor API returns keywords with different field names depending on the endpoint:
/// - Search endpoints use "label"
/// - Detail endpoints use "name"
impl<'de> serde::Deserialize<'de> for Keyword {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct KeywordVisitor;

        impl<'de> Visitor<'de> for KeywordVisitor {
            type Value = Keyword;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a keyword object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut label = None;
                let mut description = None;
                let mut created = None;
                let mut updated = None;
                let mut parent = None;
                let mut numchild = None;
                let mut full_name = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => id = Some(map.next_value()?),
                        "name" => name = Some(map.next_value()?),
                        "label" => label = Some(map.next_value()?),
                        "description" => description = map.next_value()?,
                        "created_at" => created = map.next_value()?,
                        "updated_at" => updated = map.next_value()?,
                        "parent" => parent = map.next_value()?,
                        "numchild" => numchild = map.next_value()?,
                        "full_name" => full_name = map.next_value()?,
                        _ => {
                            let _ = map.next_value::<serde_json::Value>()?;
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;

                // Use name if present, otherwise use label (handles API inconsistency)
                let name = name
                    .or_else(|| label.clone())
                    .ok_or_else(|| de::Error::missing_field("name or label"))?;

                Ok(Keyword {
                    id,
                    name,
                    label,
                    description,
                    created,
                    updated,
                    parent,
                    numchild,
                    full_name,
                })
            }
        }

        deserializer.deserialize_map(KeywordVisitor)
    }
}

/// A single cooking step within a recipe.
///
/// Steps contain ingredients and instructions, and can be ordered sequentially.
#[derive(Debug, Serialize, Deserialize)]
pub struct Step {
    /// Unique step identifier
    pub id: i32,
    /// Step name/title
    pub name: String,
    /// Cooking instruction text
    pub instruction: String,
    /// Ingredients needed for this step
    pub ingredients: Vec<StepIngredient>,
    /// Time required for this step in minutes
    pub time: Option<i32>,
    /// Display order within the recipe
    pub order: i32,
    /// Associated media file (image/video)
    pub file: Option<String>,
    /// Markdown-formatted instructions
    #[serde(default)]
    pub instructions_markdown: Option<String>,
    /// Whether to show this step as a section header
    #[serde(default)]
    pub show_as_header: Option<bool>,
    /// Reference to a sub-recipe
    #[serde(default)]
    pub step_recipe: Option<i32>,
    /// Sub-recipe data if applicable
    #[serde(default)]
    pub step_recipe_data: Option<serde_json::Value>,
    /// Whether to show ingredients in table format
    #[serde(default)]
    pub show_ingredients_table: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepIngredient {
    pub id: i32,
    /// None for section headers (is_header), which carry their text in `note`
    pub food: Option<Food>,
    pub unit: Option<Unit>,
    pub amount: f64,
    pub note: Option<String>,
    pub order: i32,
    pub is_header: bool,
    pub no_amount: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Food {
    pub id: i32,
    pub name: String,
    pub plural_name: Option<String>,
    pub description: Option<String>,
    pub recipe: Option<i32>,
    pub food_onhand: bool,
    #[serde(default)]
    pub substitute_onhand: bool,
    pub supermarket_category: Option<serde_json::Value>,
    pub inherit_fields: Vec<InheritField>,
    pub properties: Vec<FoodProperty>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: i32,
    pub name: String,
    pub plural_name: Option<String>,
    pub description: Option<String>,
    pub base_unit: Option<String>,
    pub type_: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InheritField {
    pub id: i32,
    pub name: String,
    pub field: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FoodProperty {
    pub id: i32,
    pub property_amount: f64,
    pub property_type: PropertyType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyType {
    pub id: i32,
    pub name: String,
    pub unit: String,
    pub order: i32,
    pub fdc_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nutrition {
    pub calories: Option<f64>,
    pub proteins: Option<f64>,
    pub fats: Option<f64>,
    pub carbs: Option<f64>,
}

/// An entry in a shopping list with food item and purchase details.
#[derive(Debug, Serialize, Deserialize)]
pub struct ShoppingListEntry {
    /// Unique entry identifier
    pub id: i32,
    /// Food item to purchase (Tandoor sends a slimmer food object here than elsewhere)
    pub food: ShoppingFood,
    /// Measurement unit (grams, cups, etc.)
    pub unit: Option<Unit>,
    /// Quantity to purchase
    pub amount: f64,
    /// Display order in the shopping list
    #[serde(default)]
    pub order: i32,
    /// Whether this item has been purchased
    pub checked: bool,
    /// When this entry was added
    #[serde(rename = "created_at", default)]
    pub created: Option<DateTime<Utc>>,
    /// When this item was marked as purchased
    #[serde(rename = "completed_at", default)]
    pub completed: Option<DateTime<Utc>>,
    /// Optional delay before showing this item
    #[serde(default)]
    pub delay_until: Option<DateTime<Utc>>,
    /// User who added this entry
    #[serde(default)]
    pub created_by: serde_json::Value,
    /// Set when the entry was added from a recipe or meal plan
    #[serde(default)]
    pub list_recipe: Option<i32>,
    /// Named shopping lists this entry is on
    #[serde(default)]
    pub shopping_lists: Vec<NamedRef>,
}

/// A minimal {id, name} reference to another object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedRef {
    pub id: i32,
    pub name: String,
}

/// The food object embedded in a shopping list entry.
#[derive(Debug, Serialize, Deserialize)]
pub struct ShoppingFood {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub plural_name: Option<String>,
    #[serde(default)]
    pub supermarket_category: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MealPlan {
    pub id: i32,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub recipe: Option<Recipe>,
    /// Tandoor stores servings as a decimal (e.g. `2.0`)
    pub servings: f64,
    #[serde(default)]
    pub note: Option<String>,
    /// Start as a local datetime, e.g. `2026-10-19T12:00:00-04:00`
    pub from_date: String,
    /// End as a local datetime; equals `from_date` for single-day plans
    #[serde(default)]
    pub to_date: Option<String>,
    pub meal_type: MealType,
    #[serde(default)]
    pub created_by: serde_json::Value,
    #[serde(default)]
    pub shopping: bool,
}

impl MealPlan {
    /// Start date as `YYYY-MM-DD`, in the Tandoor server's timezone.
    pub fn date(&self) -> &str {
        local_date(&self.from_date)
    }

    /// End date as `YYYY-MM-DD`, falling back to the start date.
    pub fn end_date(&self) -> &str {
        self.to_date.as_deref().map_or(self.date(), local_date)
    }
}

/// The `YYYY-MM-DD` prefix of a local ISO-8601 datetime (no timezone conversion).
fn local_date(datetime: &str) -> &str {
    datetime.get(..10).unwrap_or(datetime)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MealType {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub order: i32,
    /// Hex color; null when none was picked in Tandoor
    pub color: Option<String>,
    /// Default time of day (HH:MM:SS), if set
    #[serde(default)]
    pub time: Option<String>,
    /// Not returned by current Tandoor versions
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub created_by: serde_json::Value,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CookLog {
    pub id: i32,
    /// Recipe ID (current Tandoor) or a nested recipe object (older versions)
    pub recipe: serde_json::Value,
    #[serde(default)]
    pub servings: Option<i32>,
    #[serde(default)]
    pub rating: Option<i32>,
    #[serde(default)]
    pub comment: Option<String>,
    /// Local datetime, e.g. `2026-10-14T15:26:00.776588-04:00`
    #[serde(alias = "created")]
    pub created_at: String,
    #[serde(default)]
    pub created_by: serde_json::Value,
}

impl CookLog {
    pub fn recipe_id(&self) -> Option<i32> {
        match &self.recipe {
            serde_json::Value::Number(n) => n.as_i64().map(|id| id as i32),
            other => other.get("id")?.as_i64().map(|id| id as i32),
        }
    }

    /// Recipe name when Tandoor nests the recipe; current versions only send the ID.
    pub fn recipe_name(&self) -> Option<&str> {
        self.recipe.get("name")?.as_str()
    }

    /// Date cooked as `YYYY-MM-DD`, in the Tandoor server's timezone.
    pub fn date(&self) -> &str {
        local_date(&self.created_at)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeImport {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeFromSourceResponse {
    pub recipe: Option<serde_json::Value>,
    pub recipe_id: Option<i64>,
    pub images: Vec<serde_json::Value>,
    pub error: bool,
    pub msg: String,
    pub duplicates: Vec<serde_json::Value>,
}

/// Standard paginated response wrapper used by most Tandoor API endpoints.
///
/// This follows Django REST framework pagination format.
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Total number of items across all pages
    pub count: i32,
    /// URL for the next page of results
    pub next: Option<String>,
    /// URL for the previous page of results
    pub previous: Option<String>,
    /// Items for the current page
    pub results: Vec<T>,
}

// Request types for creating/updating resources
// These types are used when sending data to the API to create or modify entities.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRecipeRequest {
    pub name: String,
    // Tandoor rejects null for these, so omit them when unset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servings: Option<i32>,
    pub working_time: i32,
    pub waiting_time: i32,
    pub keywords: Vec<CreateKeywordRequest>,
    pub steps: Vec<CreateStepRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKeywordRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStepRequest {
    // Tandoor rejects null for these, so omit them when unset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub instruction: String,
    pub ingredients: Vec<CreateStepIngredientRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i32>,
    pub order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStepIngredientRequest {
    /// None for section headers
    pub food: Option<CreateFoodRequest>,
    pub unit: Option<CreateUnitRequest>,
    pub amount: String,
    pub note: Option<String>,
    pub order: i32,
    pub is_header: bool,
    pub no_amount: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFoodRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUnitRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShoppingListEntryRequest {
    pub food: i32,
    pub unit: Option<i32>,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMealPlanRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<i32>,
    /// Tandoor rejects `null` here, so unset fields are omitted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub servings: f64,
    pub from_date: chrono::NaiveDate,
    pub to_date: chrono::NaiveDate,
    pub meal_type: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCookLogRequest {
    pub recipe: i32,
    pub servings: i32,
    pub rating: Option<i32>,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewFoodRequest {
    pub name: String,
    pub food_onhand: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supermarket_category: Option<SupermarketCategoryRef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupermarketCategoryRef {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFoodRequest {
    pub food_onhand: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateShoppingListEntryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeBook {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub shared: Vec<serde_json::Value>,
    pub filter: Option<serde_json::Value>,
    pub order: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeBookEntry {
    pub id: i32,
    pub book: i32,
    pub book_content: Option<serde_json::Value>,
    pub recipe: i32,
    pub recipe_content: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Supermarket {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnitConversion {
    pub id: i32,
    pub name: Option<String>,
    pub base_amount: f64,
    pub base_unit: Option<serde_json::Value>,
    pub converted_amount: f64,
    pub converted_unit: Option<serde_json::Value>,
    pub food: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRecipeBookRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub shared: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRecipeBookEntryRequest {
    pub book: i32,
    pub recipe: i32,
}

impl StepIngredient {
    /// Food name, or "" for section headers that have no food
    pub fn food_name(&self) -> &str {
        self.food.as_ref().map(|f| f.name.as_str()).unwrap_or("")
    }
}

/// A place food is stored (Pantry, Fridge, Freezer, ...), owned by a household.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct InventoryLocation {
    pub id: i32,
    pub name: String,
    pub is_freezer: bool,
    pub household: Option<NamedRef>,
}

/// One item on the Pantry page: a food in a location, with amount and optional expiry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct InventoryEntry {
    pub id: i32,
    pub inventory_location: Option<InventoryLocation>,
    pub sub_location: Option<String>,
    pub food: Option<NamedRef>,
    pub unit: Option<NamedRef>,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub amount: f64,
    /// YYYY-MM-DD, or None when the entry doesn't expire
    pub expires: Option<String>,
    pub note: Option<String>,
}

impl InventoryEntry {
    pub fn food_id(&self) -> Option<i32> {
        self.food.as_ref().map(|f| f.id)
    }

    pub fn location_name(&self) -> &str {
        self.inventory_location
            .as_ref()
            .map(|l| l.name.as_str())
            .unwrap_or("")
    }
}

/// Tandoor decimals arrive as numbers or, depending on settings, as strings ("1.00").
fn deserialize_decimal<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        serde_json::Value::String(s) => s.trim().parse().unwrap_or(0.0),
        _ => 0.0,
    })
}
