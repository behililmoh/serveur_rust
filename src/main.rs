use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use if_addrs::get_if_addrs;
use qrcode_generator::QrCodeEcc;

#[get("/")]
async fn hello() -> impl Responder {
    let local_ips = get_local_ips();
    let port = 8080;
    
    let mut html = String::from(r#"
    <!DOCTYPE html>
    <html lang="fr">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Serveur  shering</title>
        <style>
            body {
                font-family: 'Arial', sans-serif;
                max-width: 800px;
                margin: 0 auto;
                padding: 20px;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
            }
            .container {
                background: rgba(255, 255, 255, 0.1);
                padding: 30px;
                border-radius: 15px;
                backdrop-filter: blur(10px);
                box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
            }
            h1 {
                text-align: center;
                margin-bottom: 30px;
                font-size: 2.5em;
                text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.5);
            }
            .qr-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                gap: 20px;
                margin-top: 30px;
            }
            .qr-card {
                background: rgba(255, 255, 255, 0.15);
                padding: 20px;
                border-radius: 10px;
                text-align: center;
                transition: transform 0.3s ease;
            }
            .qr-card:hover {
                transform: translateY(-5px);
            }
            .qr-code {
                font-family: monospace;
                line-height: 1;
                font-size: 6px;
                background: white;
                padding: 15px;
                border-radius: 8px;
                display: inline-block;
                color: black;
            }
            .url {
                margin-top: 15px;
                font-weight: bold;
                word-break: break-all;
            }
            .ip-list {
                background: rgba(255, 255, 255, 0.1);
                padding: 15px;
                border-radius: 8px;
                margin: 20px 0;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🚀 Serveur     shering Web</h1>
            <div class="ip-list">
                <h3>🌐 Adresses IP disponibles:</h3>
                <ul>
    "#);

    // Liste des IPs
    for ip in &local_ips {
        html.push_str(&format!("<li><strong>{}</strong></li>", ip));
    }
    html.push_str(&format!("<li><strong>localhost</strong></li>"));
    html.push_str(&format!("<li><strong>127.0.0.1</strong></li>"));

    html.push_str(r#"
                </ul>
            </div>
            <h2>📱 QR Codes d'accès</h2>
            <div class="qr-grid">
    "#);

    // QR code pour localhost
    let localhost_url = format!("http://localhost:{}", port);
    let localhost_qr = generate_qr_code_svg(&localhost_url);
    html.push_str(&format!(r#"
        <div class="qr-card">
            <h3>Localhost</h3>
            <div class="qr-code">{}</div>
            <div class="url">{}</div>
        </div>
    "#, localhost_qr, localhost_url));

    // QR codes pour les IPs réseau
    for ip in local_ips {
        let url = format!("http://{}:{}", ip, port);
        let qr_svg = generate_qr_code_svg(&url);
        html.push_str(&format!(r#"
            <div class="qr-card">
                <h3>Réseau</h3>
                <div class="qr-code">{}</div>
                <div class="url">{}</div>
            </div>
        "#, qr_svg, url));
    }

    html.push_str(r#"
            </div>
            <div style="text-align: center; margin-top: 30px;">
                <p>📸 Scannez le QR code avec votre téléphone pour accéder au serveur</p>
                <p>🔄 La page se rafraîchit automatiquement toutes les 30 secondes</p>
            </div>
        </div>
        <script>
            // Rafraîchissement automatique
            setTimeout(() => location.reload(), 30000);
        </script>
    </body>
    </html>
    "#);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn get_local_ips() -> Vec<String> {
    let mut ips = Vec::new();
    
    match get_if_addrs() {
        Ok(interfaces) => {
            for interface in interfaces {
                if !interface.is_loopback() && interface.ip().is_ipv4() {
                    ips.push(interface.ip().to_string());
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Erreur lecture interfaces: {}", e);
        }
    }
    
    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }
    
    ips
}

fn generate_qr_code_svg(url: &str) -> String {
    match qrcode_generator::to_svg_to_string(url, QrCodeEcc::Low, 128, None::<&str>) {
        Ok(svg) => svg,
        Err(_) => format!("<div style='color: red;'>Erreur QR code</div>")
    }
}

#[actix_web::main]
//async fn main() -> std::io::Result<()> {
   // let port = 8080;
    
  //  println!("🚀 Serveur démarré sur http://0.0.0.0:{}", port);
 //   println!("📱 Accédez à http://localhost:{} pour voir les QR codes", port);
    
//for ip in local_ips {
   //     println!("   • Réseau: http://{}:{}", ip, port);
  //  }

    
//HttpServer::new(|| App::new().service(hello))
   ////     .bind(("0.0.0.0", port))?
   //     .run()
//.await
//}


///
 async fn main() -> std::io::Result<()> {
    let port = 8080;
    let local_ips = get_local_ips();
    
    println!("🌐 Adresses IP de votre machine:");
    println!("══════════════════════════════════════════════════");
    
    for ip in &local_ips {
        println!("📡 {}", ip);
    }
    
    println!("══════════════════════════════════════════════════");
    println!("🚀 Serveur démarré sur toutes les interfaces (0.0.0.0)");
    println!("📋 URLs d'accès:");
    println!("   • Local: http://localhost:{}", port);
    println!("   • Loopback: http://127.0.0.1:{}", port);
    
    for ip in local_ips {
        println!("   • Réseau: http://{}:{}", ip, port);
    }
    
    println!("══════════════════════════════════════════════════");

    HttpServer::new(|| App::new().service(hello))
        .bind(("0.0.0.0", port))?
        .run()
        .await
       
}
