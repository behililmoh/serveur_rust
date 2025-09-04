
# Rust  Server with QR Codes
A Rust web server that automatically generates QR codes for all network interfaces, making it easy to access your server from any device on the local network.

🚀 Features
🌐 Automatic IP Detection - Discovers all available network interfaces

📱 QR Code Generation - Creates scannable QR codes for each IP address

🎨 Beautiful Web Interface - Responsive design with modern styling

⚡ Built with Actix-web - High-performance Rust web framework

📊 Real-time Network Info - Displays all available network interfaces

# 📦 Installation
Prerequisites
Rust and Cargo installed (rustup.rs)

Git
```bash
# Clone the repository
git clone https://github.com/your-username/your-repo-name.
git clone https://github.com/behililmoh/serveur_rust.git
cd  serveur_rust

# Build and run
cargo run
```
The server will start on http://0.0.0.0:8080
# 🌐 Network Access
The server is accessible through:

http://localhost:8080 (local machine)

http://127.0.0.1:8080 (loopback)

http://[YOUR-LOCAL-IP]:8080 (network devices)


# 📁 Project Structure
text

src/
├── main.rs          # Main server logic and web routes
Cargo.toml          # Dependencies and project config
.gitignore          # Git ignore rules
README.md           # This file