use crate::db::Database;
use tauri::{State, command};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub sender: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Serialize)]
struct ChatContent {
    role: String,
    parts: Vec<ChatPart>,
}

#[derive(Serialize)]
struct ChatPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<ChatContent>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
}

#[derive(Deserialize)]
struct GeminiContent {
    parts: Option<Vec<GeminiPart>>,
}

#[derive(Deserialize)]
struct GeminiPart {
    text: Option<String>,
}

#[command]
pub fn create_conversation(title: String, db: State<Database>) -> Result<i64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO conversations (title) VALUES (?1)",
        rusqlite::params![title],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[command]
pub fn get_conversations(db: State<Database>) -> Result<Vec<Conversation>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, title, created_at FROM conversations ORDER BY updated_at DESC").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut conversations = Vec::new();
    for row in rows {
        conversations.push(row.map_err(|e| e.to_string())?);
    }
    Ok(conversations)
}

#[command]
pub fn get_messages(conversation_id: i64, db: State<Database>) -> Result<Vec<Message>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, conversation_id, sender, content, created_at FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC").map_err(|e| e.to_string())?;
    let rows = stmt.query_map(rusqlite::params![conversation_id], |row| {
        Ok(Message {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            sender: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut messages = Vec::new();
    for row in rows {
        messages.push(row.map_err(|e| e.to_string())?);
    }
    Ok(messages)
}

#[command]
pub fn delete_conversation(conversation_id: i64, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM conversations WHERE id = ?1", rusqlite::params![conversation_id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn send_chat_message(conversation_id: i64, user_message: String, api_key: String, db: State<'_, Database>) -> Result<Message, String> {
    // 1. Save User Message
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO messages (conversation_id, sender, content) VALUES (?1, 'user', ?2)",
            rusqlite::params![conversation_id, user_message],
        ).map_err(|e| e.to_string())?;
        
        // Update conversation timestamp
        conn.execute("UPDATE conversations SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1", rusqlite::params![conversation_id]).map_err(|e| e.to_string())?;
    }

    // 2. Prepare API Call
    // Fetch full history to send context
    let history = get_messages(conversation_id, db.clone())?;
    let mut contents = Vec::new();
    for msg in history {
         contents.push(ChatContent {
             role: if msg.sender == "user" { "user".to_string() } else { "model".to_string() },
             parts: vec![ChatPart { text: msg.content }],
         });
    }
    
    let client = reqwest::Client::new();
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key);
    
    let request_body = GeminiRequest { contents };
    
    let response = client.post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
        
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("API Error: {}", error_text));
    }
    
    let gemini_resp: GeminiResponse = response.json().await.map_err(|e| format!("Parse error: {}", e))?;
    
    let ai_text = gemini_resp.candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content)
        .and_then(|c| c.parts)
        .and_then(|p| p.into_iter().next())
        .and_then(|p| p.text)
        .unwrap_or_else(|| "No response from AI.".to_string());
        
    // 3. Save AI Response
    let message: Message = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO messages (conversation_id, sender, content) VALUES (?1, 'ai', ?2)",
            rusqlite::params![conversation_id, ai_text],
        ).map_err(|e| e.to_string())?;
        let id = conn.last_insert_rowid();
        
        // Return full message object
         conn.query_row(
            "SELECT id, conversation_id, sender, content, created_at FROM messages WHERE id = ?1",
            rusqlite::params![id],
            |row| Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                sender: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        ).map_err(|e| e.to_string())?
    };
    
    Ok(message)
}
