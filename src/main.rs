use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
    use serde::Deserialize;
    use std::env;
    use reqwest::Client;
    use regex::Regex;  // Importando o módulo regex

    #[derive(Deserialize)]
    struct CheckRequest {
      link: String,
      r#type: String,
    }

    #[post("/check-downloadable")]
    async fn check_downloadable(req: web::Json<CheckRequest>) -> impl Responder {
      let api_key = env::var("GOOGLE_DRIVE_API_KEY").expect("API key not set");
      let client = Client::new();
      let id = extract_id_from_link(&req.link);

      if id.is_none() {
        return HttpResponse::BadRequest().body("Invalid link format");
      }

      let id = id.unwrap();
      let downloadable = if req.link.contains("/folders/") {
        check_folder(&client, &api_key, &id, &req.r#type).await
      } else {
        is_downloadable(&client, &api_key, &id, &req.r#type).await
      };

      HttpResponse::Ok().json(serde_json::json!({ "result": if downloadable { "yes" } else { "no" } }))
    }

    async fn is_downloadable(client: &Client, api_key: &str, file_id: &str, file_type: &str) -> bool {
      let url = format!("https://www.googleapis.com/drive/v3/files/{}?fields=mimeType&key={}", file_id, api_key);
      if let Ok(res) = client.get(&url).send().await {
        if let Ok(json) = res.json::<serde_json::Value>().await {
          if let Some(mime_type) = json.get("mimeType").and_then(|v| v.as_str()) {
            return match file_type {
              "pdf" => mime_type == "application/pdf",
              "image" => mime_type.starts_with("image/"),
              "video" => mime_type.starts_with("video/"),
              _ => false,
            };
          }
        }
      }
      false
    }

    async fn check_folder(client: &Client, api_key: &str, folder_id: &str, file_type: &str) -> bool {
      let url = format!("https://www.googleapis.com/drive/v3/files?q='{}'+in+parents&fields=files(mimeType)&key={}", folder_id, api_key);
      if let Ok(res) = client.get(&url).send().await {
        if let Ok(json) = res.json::<serde_json::Value>().await {
          if let Some(files) = json.get("files").and_then(|v| v.as_array()) {
            for file in files {
              if let Some(mime_type) = file.get("mimeType").and_then(|v| v.as_str()) {
                if match file_type {
                  "pdf" => mime_type == "application/pdf",
                  "image" => mime_type.starts_with("image/"),
                  "video" => mime_type.starts_with("video/"),
                  _ => false,
                } {
                  return true;
                }
              }
            }
          }
        }
      }
      false
    }

    fn extract_id_from_link(link: &str) -> Option<String> {
      let file_id_re = Regex::new(r"/d/([a-zA-Z0-9_-]+)").unwrap();
      let folder_id_re = Regex::new(r"/folders/([a-zA-Z0-9_-]+)").unwrap();

      if let Some(caps) = file_id_re.captures(link) {
        return Some(caps[1].to_string());
      } else if let Some(caps) = folder_id_re.captures(link) {
        return Some(caps[1].to_string());
      }
      None
    }

    #[actix_web::main]
    async fn main() -> std::io::Result<()> {
      dotenv::dotenv().ok();
      let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
      HttpServer::new(|| {
        App::new()
          .service(check_downloadable)
      })
      .bind(("0.0.0.0", port.parse().unwrap()))?
      .run()
      .await
    }
