use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingList {
    pub id: i64,
    pub group_id: i64,
    pub created_by: i64,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListSummary {
    pub id: i64,
    pub name: String,
    pub group_id: i64,
    pub created_by: i64,
    pub creator_username: String,
    pub total_items: i64,
    pub completed_items: i64,
}

impl ShoppingListSummary {
    pub fn completion_percentage(&self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            (self.completed_items as f64 / self.total_items as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListItem {
    pub id: i64,
    pub shopping_list_id: i64,
    pub name: String,
    pub quantity: Option<String>,
    pub category: Option<String>,
    pub is_completed: bool,
    pub completed_by: Option<i64>,
    pub completed_by_username: Option<String>,
    pub completed_at: Option<OffsetDateTime>,
    pub position: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListActivity {
    pub id: i64,
    pub shopping_list_id: i64,
    pub user_id: i64,
    pub username: String,
    pub action: String,
    pub item_name: String,
    pub created_at: OffsetDateTime,
}

impl ShoppingListActivity {
    pub fn action_description(&self) -> String {
        match self.action.as_str() {
            "added_item" => format!("{} added {}", self.username, self.item_name),
            "completed_item" => format!("{} completed {}", self.username, self.item_name),
            "uncompleted_item" => format!("{} uncompleted {}", self.username, self.item_name),
            "deleted_item" => format!("{} deleted {}", self.username, self.item_name),
            _ => format!("{} {} {}", self.username, self.action, self.item_name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShoppingListEvent {
    ItemAdded {
        item_id: i64,
        name: String,
        quantity: Option<String>,
        category: Option<String>,
        position: i64,
        added_by_username: String,
    },
    ItemToggled {
        item_id: i64,
        is_completed: bool,
        completed_by_username: Option<String>,
    },
    ItemDeleted {
        item_id: i64,
    },
    ItemUpdated {
        item_id: i64,
        name: String,
        quantity: Option<String>,
        category: Option<String>,
    },
    ListUpdated {
        name: String,
    },
    ListDeleted,
}
