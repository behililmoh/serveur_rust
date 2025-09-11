use actix_multipart::Multipart;
use actix_web::{
    get, post, web, App, HttpResponse, HttpServer, Responder, Result,
    middleware::{Logger, DefaultHeaders},
    http::header::{ContentDisposition, DispositionType, DispositionParam},
};
use futures_util::TryStreamExt as _;
use if_addrs::get_if_addrs;
use qrcode_generator::QrCodeEcc;
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
    name: String,
    size: u64,
    uploaded_at: u64,
    file_type: String,
}

struct Config {
    port: u16,
    refresh_interval: u32,
    upload_dir: String,
    max_file_size: usize,
}

impl Config {
    fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            refresh_interval: env::var("REFRESH_INTERVAL")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000),
            upload_dir: env::var("UPLOAD_DIR")
                .unwrap_or_else(|_| "./uploads".to_string()),
            max_file_size: env::var("MAX_FILE_SIZE")
                .unwrap_or_else(|_| "50".to_string())
                .parse::<usize>()
                .unwrap_or(50) * 1024 * 1024, // MB vers bytes
        }
    }
}

#[get("/")]
async fn index() -> impl Responder {
    let config = Config::from_env();
    let local_ips = get_local_ips();
    let files = get_uploaded_files(&config.upload_dir);
    
    let html = generate_html(&local_ips, config.port, config.refresh_interval, &files, config.max_file_size);
    
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .insert_header(("X-Frame-Options", "SAMEORIGIN"))
        .body(html)
}

#[post("/upload")]
async fn upload_file(mut payload: Multipart) -> Result<HttpResponse> {
    let config = Config::from_env();
    
    // Créer le dossier d'upload s'il n'existe pas
    fs::create_dir_all(&config.upload_dir).map_err(|e| {
        eprintln!("❌ Erreur création dossier upload: {}", e);
        actix_web::error::ErrorInternalServerError("Erreur serveur")
    })?;

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        
        if let Some(filename) = content_disposition.get_filename() {
            // Sécuriser le nom de fichier
            let safe_filename = sanitize_filename(filename);
            let filepath = PathBuf::from(&config.upload_dir).join(&safe_filename);
            
            // Éviter l'écrasement en ajoutant un timestamp
            let final_path = if filepath.exists() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let stem = filepath.file_stem().unwrap().to_string_lossy();
                let extension = filepath.extension().map_or(String::new(), |e| format!(".{}", e.to_string_lossy()));
                PathBuf::from(&config.upload_dir).join(format!("{}_{}{}", stem, timestamp, extension))
            } else {
                filepath
            };

            let mut f = web::block(move || std::fs::File::create(final_path))
                .await??;

            let mut total_size = 0usize;
            while let Some(chunk) = field.try_next().await? {
                total_size += chunk.len();
                if total_size > config.max_file_size {
                    return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "error": format!("Fichier trop volumineux (max: {} MB)", config.max_file_size / (1024 * 1024))
                    })));
                }
                f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
            }

            println!("📁 Fichier uploadé: {} ({} bytes)", safe_filename, total_size);
        }
    }

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/"))
        .finish())
}

#[get("/download/{filename}")]
async fn download_file(path: web::Path<String>) -> Result<HttpResponse> {
    let config = Config::from_env();
    let filename = path.into_inner();
    let safe_filename = sanitize_filename(&filename);
    let filepath = PathBuf::from(&config.upload_dir).join(&safe_filename);

    if !filepath.exists() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Fichier non trouvé"
        })));
    }

    let file_data = web::block(move || std::fs::read(filepath)).await??;
    
    let content_disposition = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(safe_filename)],
    };

    Ok(HttpResponse::Ok()
        .insert_header(content_disposition)
        .body(file_data))
}

#[post("/delete/{filename}")]
async fn delete_file(path: web::Path<String>) -> Result<HttpResponse> {
    let config = Config::from_env();
    let filename = path.into_inner();
    let safe_filename = sanitize_filename(&filename);
    let filepath = PathBuf::from(&config.upload_dir).join(&safe_filename);

    match std::fs::remove_file(&filepath) {
        Ok(_) => {
            println!("🗑️ Fichier supprimé: {}", safe_filename);
            Ok(HttpResponse::Found()
                .insert_header(("Location", "/"))
                .finish())
        }
        Err(e) => {
            eprintln!("❌ Erreur suppression fichier: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Erreur lors de la suppression"
            })))
        }
    }
}

fn generate_html(local_ips: &[String], port: u16, refresh_interval: u32, files: &[FileInfo], max_file_size: usize) -> String {
    let max_size_mb = max_file_size / (1024 * 1024);
    
    let mut html = format!(r#"
    <!DOCTYPE html>
    <html lang="fr">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Serveur de partage - Fichiers</title>
        <style>
            * {{
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }}
            
            body {{
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                min-height: 100vh;
                padding: 20px;
            }}
            
            .container {{
                max-width: 1200px;
                margin: 0 auto;
                background: rgba(255, 255, 255, 0.1);
                backdrop-filter: blur(15px);
                border-radius: 20px;
                padding: 30px;
                box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
            }}
            
            .header {{
                text-align: center;
                margin-bottom: 40px;
            }}
            
            .header h1 {{
                font-size: 3em;
                margin-bottom: 10px;
                text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.5);
                background: linear-gradient(45deg, #ff6b6b, #4ecdc4, #45b7d1);
                -webkit-background-clip: text;
                -webkit-text-fill-color: transparent;
                background-clip: text;
            }}
            
            .tabs {{
                display: flex;
                justify-content: center;
                margin-bottom: 30px;
            }}
            
            .tab {{
                background: rgba(255, 255, 255, 0.2);
                border: none;
                padding: 15px 30px;
                color: white;
                font-weight: bold;
                cursor: pointer;
                border-radius: 25px;
                margin: 0 10px;
                transition: all 0.3s ease;
            }}
            
            .tab.active, .tab:hover {{
                background: rgba(255, 255, 255, 0.3);
                transform: translateY(-2px);
            }}
            
            .tab-content {{
                display: none;
            }}
            
            .tab-content.active {{
                display: block;
            }}
            
            .upload-area {{
                background: rgba(255, 255, 255, 0.15);
                border: 3px dashed rgba(255, 255, 255, 0.5);
                border-radius: 15px;
                padding: 40px;
                text-align: center;
                margin-bottom: 30px;
                transition: all 0.3s ease;
                cursor: pointer;
            }}
            
            .upload-area:hover, .upload-area.drag-over {{
                border-color: #4ecdc4;
                background: rgba(78, 205, 196, 0.1);
            }}
            
            .upload-area input[type="file"] {{
                display: none;
            }}
            
            .upload-btn {{
                background: linear-gradient(45deg, #4ecdc4, #44a08d);
                border: none;
                padding: 15px 30px;
                border-radius: 25px;
                color: white;
                font-weight: bold;
                cursor: pointer;
                font-size: 16px;
                transition: transform 0.3s ease;
            }}
            
            .upload-btn:hover {{
                transform: scale(1.05);
            }}
            
            .files-grid {{
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
                gap: 20px;
                margin-top: 30px;
            }}
            
            .file-card {{
                background: rgba(255, 255, 255, 0.15);
                border-radius: 15px;
                padding: 20px;
                transition: all 0.3s ease;
            }}
            
            .file-card:hover {{
                transform: translateY(-5px);
                box-shadow: 0 15px 35px rgba(0, 0, 0, 0.3);
            }}
            
            .file-icon {{
                font-size: 3em;
                text-align: center;
                margin-bottom: 15px;
            }}
            
            .file-name {{
                font-weight: bold;
                margin-bottom: 10px;
                word-break: break-word;
            }}
            
            .file-info {{
                font-size: 0.9em;
                opacity: 0.8;
                margin-bottom: 15px;
            }}
            
            .file-actions {{
                display: flex;
                gap: 10px;
            }}
            
            .btn {{
                flex: 1;
                padding: 10px;
                border: none;
                border-radius: 8px;
                cursor: pointer;
                font-weight: bold;
                transition: all 0.3s ease;
            }}
            
            .btn-download {{
                background: linear-gradient(45deg, #4ecdc4, #44a08d);
                color: white;
            }}
            
            .btn-delete {{
                background: linear-gradient(45deg, #ff6b6b, #ee5a52);
                color: white;
            }}
            
            .btn:hover {{
                transform: scale(1.05);
            }}
            
            .qr-grid {{
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                gap: 20px;
            }}
            
            .qr-card {{
                background: rgba(255, 255, 255, 0.15);
                padding: 20px;
                border-radius: 15px;
                text-align: center;
                transition: transform 0.3s ease;
            }}
            
            .qr-card:hover {{
                transform: translateY(-5px);
            }}
            
            .qr-code {{
                background: white;
                padding: 15px;
                border-radius: 10px;
                display: inline-block;
                margin: 10px 0;
            }}
            
            .status-bar {{
                position: fixed;
                top: 20px;
                right: 20px;
                background: rgba(76, 175, 80, 0.9);
                padding: 12px 20px;
                border-radius: 25px;
                font-size: 0.9em;
                backdrop-filter: blur(10px);
            }}
            
            .progress-bar {{
                width: 100%;
                height: 6px;
                background: rgba(255, 255, 255, 0.3);
                border-radius: 3px;
                margin-top: 15px;
                overflow: hidden;
            }}
            
            .progress {{
                height: 100%;
                background: linear-gradient(90deg, #4ecdc4, #44a08d);
                border-radius: 3px;
                width: 0%;
                transition: width 0.3s ease;
            }}
            
            .ip-list {{
                background: rgba(255, 255, 255, 0.1);
                padding: 20px;
                border-radius: 15px;
                margin: 20px 0;
            }}
            
            .ip-list ul {{
                list-style: none;
                padding: 0;
            }}
            
            .ip-list li {{
                padding: 8px 0;
                border-bottom: 1px solid rgba(255, 255, 255, 0.2);
            }}
            
            .empty-state {{
                text-align: center;
                padding: 60px 20px;
                opacity: 0.7;
            }}
            
            .empty-state .icon {{
                font-size: 4em;
                margin-bottom: 20px;
            }}
        </style>
    </head>
    <body>
        <div class="status-bar" id="status">🟢 Serveur actif</div>
        
        <div class="container">
            <div class="header">
                <h1>📁 Serveur de Partage</h1>
                <p>Partagez vos fichiers facilement sur le réseau local</p>
            </div>
            
            <div class="tabs">
                <button class="tab active" onclick="showTab('files')">📁 Fichiers</button>
                <button class="tab" onclick="showTab('qr')">📱 QR Codes</button>
            </div>
            
            <div id="files" class="tab-content active">
                <form action="/upload" method="post" enctype="multipart/form-data" id="uploadForm">
                    <div class="upload-area" id="uploadArea">
                        <div style="font-size: 3em; margin-bottom: 20px;">☁️</div>
                        <h3>Glissez vos fichiers ici ou cliquez pour sélectionner</h3>
                        <p>Taille maximale: {} MB par fichier</p>
                        <input type="file" id="fileInput" name="file" multiple accept="*/*">
                        <div class="progress-bar" id="progressBar" style="display: none;">
                            <div class="progress" id="progress"></div>
                        </div>
                    </div>
                </form>
                
                <h2>📋 Fichiers disponibles ({})</h2>
    "#, max_size_mb, files.len());

    if files.is_empty() {
        html.push_str(r#"
            <div class="empty-state">
                <div class="icon">📭</div>
                <h3>Aucun fichier partagé</h3>
                <p>Uploadez des fichiers pour commencer le partage</p>
            </div>
        "#);
    } else {
        html.push_str(r#"<div class="files-grid">"#);
        
        for file in files {
            let file_icon = get_file_icon(&file.file_type);
            let file_size = format_file_size(file.size);
            let upload_date = format_timestamp(file.uploaded_at);
            
            html.push_str(&format!(r#"
                <div class="file-card">
                    <div class="file-icon">{}</div>
                    <div class="file-name">{}</div>
                    <div class="file-info">
                        📏 {} | 🕒 {}
                    </div>
                    <div class="file-actions">
                        <button class="btn btn-download" onclick="downloadFile('{}')">
                            ⬇️ Télécharger
                        </button>
                        <button class="btn btn-delete" onclick="deleteFile('{}')">
                            🗑️ Supprimer
                        </button>
                    </div>
                </div>
            "#, file_icon, file.name, file_size, upload_date, file.name, file.name));
        }
        
        html.push_str(r#"</div>"#);
    }

    html.push_str(r#"
            </div>
            
            <div id="qr" class="tab-content">
                <div class="ip-list">
                    <h3>🌐 Adresses d'accès disponibles:</h3>
                    <ul>
    "#);

    for ip in local_ips {
        html.push_str(&format!("<li>📡 <strong>http://{}:{}</strong></li>", ip, port));
    }
    html.push_str(&format!("<li>🏠 <strong>http://localhost:{}</strong></li>", port));

    html.push_str(r#"
                    </ul>
                </div>
                
                <h2>📱 QR Codes d'accès</h2>
                <div class="qr-grid">
    "#);

    // QR codes
    let urls = [
        ("Localhost", format!("http://localhost:{}", port)),
    ];

    for (name, url) in urls {
        let qr_svg = generate_qr_code_svg(&url);
        html.push_str(&format!(r#"
            <div class="qr-card">
                <h3>{}</h3>
                <div class="qr-code">{}</div>
                <div style="margin-top: 15px; font-weight: bold; word-break: break-all; font-size: 0.9em;">{}</div>
            </div>
        "#, name, qr_svg, url));
    }

    for ip in local_ips {
        let url = format!("http://{}:{}", ip, port);
        let qr_svg = generate_qr_code_svg(&url);
        html.push_str(&format!(r#"
            <div class="qr-card">
                <h3>Réseau Local</h3>
                <div class="qr-code">{}</div>
                <div style="margin-top: 15px; font-weight: bold; word-break: break-all; font-size: 0.9em;">{}</div>
            </div>
        "#, qr_svg, url));
    }

    html.push_str(&format!(r#"
                </div>
            </div>
        </div>
        
        <script>
            // Gestion des onglets
            function showTab(tabName) {{
                document.querySelectorAll('.tab-content').forEach(content => {{
                    content.classList.remove('active');
                }});
                document.querySelectorAll('.tab').forEach(tab => {{
                    tab.classList.remove('active');
                }});
                
                document.getElementById(tabName).classList.add('active');
                event.target.classList.add('active');
            }}
            
            // Gestion du drag & drop
            const uploadArea = document.getElementById('uploadArea');
            const fileInput = document.getElementById('fileInput');
            const uploadForm = document.getElementById('uploadForm');
            const progressBar = document.getElementById('progressBar');
            const progress = document.getElementById('progress');
            
            uploadArea.addEventListener('click', () => fileInput.click());
            
            uploadArea.addEventListener('dragover', (e) => {{
                e.preventDefault();
                uploadArea.classList.add('drag-over');
            }});
            
            uploadArea.addEventListener('dragleave', () => {{
                uploadArea.classList.remove('drag-over');
            }});
            
            uploadArea.addEventListener('drop', (e) => {{
                e.preventDefault();
                uploadArea.classList.remove('drag-over');
                fileInput.files = e.dataTransfer.files;
                uploadFiles();
            }});
            
            fileInput.addEventListener('change', uploadFiles);
            
            function uploadFiles() {{
                if (fileInput.files.length === 0) return;
                
                progressBar.style.display = 'block';
                progress.style.width = '0%';
                
                const formData = new FormData();
                for (let file of fileInput.files) {{
                    formData.append('file', file);
                }}
                
                const xhr = new XMLHttpRequest();
                
                xhr.upload.addEventListener('progress', (e) => {{
                    if (e.lengthComputable) {{
                        const percentComplete = (e.loaded / e.total) * 100;
                        progress.style.width = percentComplete + '%';
                    }}
                }});
                
                xhr.addEventListener('load', () => {{
                    if (xhr.status === 200 || xhr.status === 302) {{
                        setTimeout(() => location.reload(), 1000);
                    }} else {{
                        alert('Erreur lors de l\'upload');
                        progressBar.style.display = 'none';
                    }}
                }});
                
                xhr.addEventListener('error', () => {{
                    alert('Erreur réseau lors de l\'upload');
                    progressBar.style.display = 'none';
                }});
                
                xhr.open('POST', '/upload');
                xhr.send(formData);
            }}
            
            function downloadFile(filename) {{
                window.location.href = '/download/' + encodeURIComponent(filename);
            }}
            
            function deleteFile(filename) {{
                if (confirm('Êtes-vous sûr de vouloir supprimer ce fichier ?')) {{
                    fetch('/delete/' + encodeURIComponent(filename), {{
                        method: 'POST'
                    }}).then(() => location.reload());
                }}
            }}
            
            // Auto-refresh avec countdown
            let countdown = {};
            const statusEl = document.getElementById('status');
            
            const timer = setInterval(() => {{
                countdown--;
                if (countdown > 0) {{
                    statusEl.textContent = `🔄 Refresh dans ${{countdown}}s`;
                }} else {{
                    location.reload();
                }}
            }}, 1000);
        </script>
    </body>
    </html>
    "#, refresh_interval / 1000));

    html
}

fn get_uploaded_files(upload_dir: &str) -> Vec<FileInfo> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(upload_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let size = metadata.len();
                    let uploaded_at = metadata
                        .modified()
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    
                    let file_type = Path::new(&name)
                        .extension()
                        .map_or("unknown".to_string(), |ext| ext.to_string_lossy().to_lowercase());
                    
                    files.push(FileInfo {
                        name,
                        size,
                        uploaded_at,
                        file_type,
                    });
                }
            }
        }
    }
    
    // Trier par date de modification (plus récent en premier)
    files.sort_by(|a, b| b.uploaded_at.cmp(&a.uploaded_at));
    files
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn get_file_icon(file_type: &str) -> &'static str {
    match file_type {
        "pdf" => "📄",
        "doc" | "docx" => "📝",
        "xls" | "xlsx" => "📊",
        "ppt" | "pptx" => "📽️",
        "txt" => "📄",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => "🖼️",
        "mp4" | "avi" | "mkv" | "mov" | "webm" => "🎥",
        "mp3" | "wav" | "flac" | "aac" => "🎵",
        "zip" | "rar" | "7z" | "tar" | "gz" => "📦",
        "exe" | "msi" => "⚙️",
        _ => "📄",
    }
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

fn format_timestamp(timestamp: u64) -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .saturating_sub(timestamp);
    
    match duration {
        0..=59 => "À l'instant".to_string(),
        60..=3599 => format!("Il y a {} min", duration / 60),
        3600..=86399 => format!("Il y a {}h", duration / 3600),
        _ => format!("Il y a {} jours", duration / 86400),
    }
}

fn get_local_ips() -> Vec<String> {
    let mut ips = Vec::new();
    
    match get_if_addrs() {
        Ok(interfaces) => {
            for interface in interfaces {
                if !interface.is_loopback() && interface.ip().is_ipv4() {
                    let ip = interface.ip().to_string();
                    if !ip.starts_with("172.17.") {
                        ips.push(ip);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️ Erreur lecture interfaces réseau: {}", e);
        }
    }
    
    ips
}

fn generate_qr_code_svg(url: &str) -> String {
    match qrcode_generator::to_svg_to_string(url, QrCodeEcc::Medium, 200, None::<&str>) {
        Ok(svg) => svg,
        Err(e) => {
            eprintln!("❌ Erreur génération QR code pour '{}': {}", url, e);
            r#"<div style="color: #ff6b6b;">❌ QR code indisponible</div>"#.to_string()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = Config::from_env();
    let local_ips = get_local_ips();
    
    // Créer le dossier d'upload
    fs::create_dir_all(&config.upload_dir).unwrap_or_else(|e| {
        eprintln!("⚠️ Erreur création dossier upload: {}", e);
    });
    
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║                    📁 SERVEUR DE PARTAGE DE FICHIERS                ║");
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║ 🌐 Adresses d'accès:                                              ║");
    
    for ip in &local_ips {
        println!("║   📡 http://{}:{}                                        ║", ip, config.port);
    }
    
    println!("║   🏠 http://localhost:{}                                    ║", config.port);
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║ ⚙️  Configuration:                                                ║");
    println!("║   🔌 Port: {}                                                  ║", config.port);
    println!("║   📁 Dossier upload: {}                                       ║", config.upload_dir);
    println!("║   📏 Taille max: {} MB                                         ║", config.max_file_size / (1024 * 1024));
    println!("║   🔄 Auto-refresh: {}s                                         ║", config.refresh_interval / 1000);
    println!("║   🖥️  Interface: 0.0.0.0 (toutes)                             ║");
    println!("╚════════════════════════════════════════════════════════════════════╝");
    
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(DefaultHeaders::new()
                .add(("X-Content-Type-Options", "nosniff"))
                .add(("X-Frame-Options", "SAMEORIGIN"))
            )
            .service(index)
            .service(upload_file)
            .service(download_file)
            .service(delete_file)
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}