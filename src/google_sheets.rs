use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
    token_uri: String,
}

#[derive(Debug, serde::Serialize)]
struct Claims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct SheetsResponse {
    values: Option<Vec<Vec<String>>>,
}

#[derive(Debug)]
pub struct TriggerEntry {
    pub response: String,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub image_url: String,
}

async fn get_access_token(
    service_account_key_path: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let key_data = tokio::fs::read_to_string(service_account_key_path).await?;
    let key: ServiceAccountKey = serde_json::from_str(&key_data)?;

    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        iss: key.client_email,
        scope: "https://www.googleapis.com/auth/spreadsheets.readonly".to_string(),
        aud: key.token_uri.clone(),
        iat: now,
        exp: now + 3600,
    };

    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(key.private_key.as_bytes())?;
    let jwt = jsonwebtoken::encode(&header, &claims, &encoding_key)?;

    let client = reqwest::Client::new();
    let response = client
        .post(&key.token_uri)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
        ])
        .send()
        .await?;

    let response = response.error_for_status()?;
    let token_response: TokenResponse = response.json().await?;
    Ok(token_response.access_token)
}

pub async fn search_trigger(
    service_account_key_path: &str,
    spreadsheet_id: &str,
    keyword: &str,
) -> Result<Option<TriggerEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let access_token = get_access_token(service_account_key_path).await?;

    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/trigger",
        spreadsheet_id
    );

    let client = reqwest::Client::new();
    let response = client.get(&url).bearer_auth(&access_token).send().await?;

    let response = response.error_for_status()?;
    let sheets_response: SheetsResponse = response.json().await?;

    let values = match sheets_response.values {
        Some(v) => v,
        None => return Ok(None),
    };

    // values[1] is the header row (spreadsheet row 2)
    if values.len() < 3 {
        return Ok(None);
    }
    let header = &values[1];

    // Find column indices for search fields
    let trigger_col = header.iter().position(|h| h == "trigger");
    let alias01_col = header.iter().position(|h| h == "alias01");
    let alias02_col = header.iter().position(|h| h == "alias02");

    // Find column indices for result fields
    let response_col = header.iter().position(|h| h == "response");
    let title_col = header.iter().position(|h| h == "title");
    let description_col = header.iter().position(|h| h == "description");
    let thumbnail_col = header.iter().position(|h| h == "right_small_image_URL");
    let image_col = header.iter().position(|h| h == "big_image_URL");

    let search_cols = [trigger_col, alias01_col, alias02_col];
    let keyword_lower = keyword.to_lowercase();

    // Search data rows (starting from index 2)
    for row in values.iter().skip(2) {
        let matched = search_cols.iter().any(|col| {
            col.and_then(|idx| row.get(idx))
                .is_some_and(|cell| cell.to_lowercase() == keyword_lower)
        });

        if matched {
            let get_field = |col: Option<usize>| -> String {
                col.and_then(|idx| row.get(idx))
                    .map(|s| s.to_string())
                    .unwrap_or_default()
            };

            return Ok(Some(TriggerEntry {
                response: get_field(response_col),
                title: get_field(title_col),
                description: get_field(description_col),
                thumbnail_url: get_field(thumbnail_col),
                image_url: get_field(image_col),
            }));
        }
    }

    Ok(None)
}
