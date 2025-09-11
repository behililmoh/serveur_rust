# File Sharing Server in Rust

A modern Rust web server with a file sharing interface, automatic QR code generation, and smart network detection. It's perfect for quickly sharing files on your local network.

-----

## **Key Features**

### **Comprehensive File Sharing**

  * **Drag & drop upload** - Directly drag your files to upload them.
  * **One-click download** - Get instant access to all files.
  * **File management** - Securely delete files with a confirmation.
  * **Progress bar** - Monitor uploads in real-time.
  * **Integrated security** - Includes secure file naming and overwrite protection.

### **Automatic Network Detection**

  * **Smart discovery** - The server automatically detects all available network interfaces.
  * **Automatic QR codes** - It generates QR codes for each IP address.
  * **Multi-interface** - Provides full IPv4 support across all interfaces.

### **Modern Interface**

  * **Glassmorphism design** - Features modern transparency and blur effects.
  * **Responsive** - The design works perfectly on both mobile and desktop devices.
  * **Fluid animations** - Includes attractive visual effects and transitions.
  * **Auto-refresh** - The file list updates automatically.
  * **Tabbed navigation** - The interface is organized and intuitive.

### **Performance & Reliability**

  * **Rust + Actix-web** - Built on a high-performance web framework.
  * **Robust error handling** - Features clear error messages and automatic recovery.
  * **Real-time information** - Displays server status and network information.

-----

## **Installation**

### **Prerequisites**

  * [Rust and Cargo](https://rustup.rs/) must be installed.
  * Git must be installed.

### **Quick Installation**

```bash
# Clone the repository
git clone https://github.com/behililmoh/serveur_rust.git
cd serveur_rust

# Compile and run
cargo run
```

The server will start on `http://0.0.0.0:8080`.

-----

## **Configuration**

### **Environment Variables**

```bash
# Optional configuration
export PORT=8080                    # Server port (default: 8080)
export UPLOAD_DIR=./uploads          # Storage folder (default: ./uploads)
export MAX_FILE_SIZE=50              # Max file size in MB (default: 50MB)
export REFRESH_INTERVAL=30000        # Auto-refresh in ms (default: 30s)
```

### **Example of a Run with Configuration**

```bash
# Server on port 3000 with a max upload size of 100MB
PORT=3000 MAX_FILE_SIZE=100 cargo run
```

-----

## **Network Access**

The server can be accessed via:

  * **Local** : `http://localhost:8080`
  * **Loopback** : `http://127.0.0.1:8080`
  * **Local network** : `http://[YOUR-LOCAL-IP]:8080`

### **Easy Mobile Access**

1.  Launch the server on your PC.
2.  Scan the QR code with your phone.
3.  Instantly access the sharing interface.

-----

## **Usage**

### **File Upload**

1.  **Drag & Drop** : Drag your files into the upload area.
2.  **Manual Selection** : Click the area to open the file explorer.
3.  **Multiple files** : Select several files at once.
4.  **Real-time tracking** : A progress bar displays during the upload.

### **Download**

  * Click **⬇️ Download** on any file to start the download immediately.

### **File Management**

  * **🗑️ Delete** : Delete files with a confirmation.
  * **📊 Information** : File size, type, and upload date are displayed.
  * **🔍 Smart icons** : The server automatically recognizes file types.

-----

## **API Endpoints**

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/` | Main interface |
| `POST` | `/upload` | File upload (multipart/form-data) |
| `GET` | `/download/{filename}` | File download |
| `POST` | `/delete/{filename}` | File deletion |

-----

## **Project Structure**

```
serveur_rust/
├── src/
│   └── main.rs          # Server logic and web routes
├── uploads/             # Storage folder (created automatically)
├── Cargo.toml          # Dependencies and configuration
├── Cargo.lock          # Exact versions of dependencies
├── .gitignore          # Git ignore rules
└── README.md           # This documentation
```

-----

## **Dependencies**

  * **actix-web** - A high-performance web framework.
  * **actix-multipart** - Manages multipart uploads.
  * **if-addrs** - Detects network interfaces.
  * **qrcode-generator** - Generates QR codes.
  * **futures-util** - Utilities for asynchronous programming.
  * **serde** - JSON serialization/deserialization.

-----

## **Security**

### **Integrated Security Measures**

  * **✅ Filename sanitization** - Dangerous characters are removed.
  * **✅ Size limitation** - Protection against oversized files.
  * **✅ Overwrite protection** - A timestamp is automatically added to filenames to prevent overwrites.
  * **✅ Security headers** - Includes XSS and clickjacking protection.
  * **✅ Path validation** - Prevents path traversal attacks.

### **Recommendations**

  * Only use this server on trusted networks.
  * Monitor available disk space.
  * Regularly clean the uploads folder.

-----

## **Advanced Features**

### **Supported File Types**

  * **📄 Documents** : PDF, DOC, XLS, TXT, etc.
  * **🖼️ Images** : JPG, PNG, GIF, WebP, etc.
  * **🎥 Videos** : MP4, AVI, MKV, WebM, etc.
  * **🎵 Audio** : MP3, WAV, FLAC, AAC, etc.
  * **📦 Archives** : ZIP, RAR, 7Z, TAR, etc.

### **Responsive Interface**

  * **📱 Mobile-first** - Optimized for touchscreens.
  * **💻 Desktop** - A rich interface for large screens.
  * **🎨 Modern theme** - Contemporary design with visual effects.

-----

## **Contribution**

Contributions are welcome\! Feel free to:

  * **🐛 Report bugs**.
  * **💡 Suggest features**.
  * **🔧 Submit pull requests**.
  * **📖 Improve the documentation**.

-----

## **License**

This project is under the MIT license. See the `LICENSE` file for more details.

-----

## **Acknowledgments**

  * [Actix-web](https://actix.rs/) for the web framework.
  * [QR Code Generator](https://crates.io/crates/qrcode-generator) for QR code generation.
  * The Rust community for the fantastic ecosystem.

-----

**⭐ Don't forget to give it a star if this project was useful to you\!**