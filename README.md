# 🚀 Dev Proxy

A high-performance development proxy built with Rust and Pingora, featuring traffic recording, mock responses, filtering, and real-time analytics dashboard.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![SvelteKit](https://img.shields.io/badge/sveltekit-%23f1413d.svg?style=for-the-badge&logo=svelte&logoColor=white)
![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)

## ✨ Features

### 🎯 Core Features
- **High-Performance Proxy** - Built on Cloudflare's Pingora framework
- **Traffic Recording** - Capture all HTTP requests and responses with full body content
- **Request Mocking** - Intercept requests and return custom responses without hitting backend
- **Advanced Filtering** - Search and filter by method, status code, URL pattern, duration
- **Real-time Analytics** - Live dashboard with charts and statistics

### 📊 Dashboard Features
- **Traffic Statistics** - Total requests, success rate, error breakdown
- **Method Distribution Chart** - Visual breakdown of HTTP methods (GET, POST, etc.)
- **Status Code Distribution** - Response status patterns (2xx, 3xx, 4xx, 5xx)
- **Request Timeline** - Last hour of activity with color-coded status
- **Top 10 Endpoints** - Most requested endpoints with average duration and error rates

### 🎭 Mock Features
- **Flexible Matching** - Match by exact URL, contains, starts with, ends with, or regex
- **Method Filtering** - Mock specific HTTP methods or any method
- **Priority System** - Control which mock rules take precedence
- **Custom Responses** - Set status code, headers, and body for mocked responses
- **Artificial Delays** - Simulate slow API responses for testing
- **Live Management** - Create, edit, delete, and toggle mocks without restarting

## 🏗️ Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│   HTTP Layer    │ ◄── Handles mocks (Port 8080)
│   (Port 8080)   │
└────────┬────────┘
         │
         ▼
    ┌────────┐
    │ Mock?  │──Yes──► Return Mock Response
    └───┬────┘
        │No
        ▼
┌─────────────────┐
│ Pingora Proxy   │ ◄── Records traffic (Port 9090 internal)
│  (Port 9090)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Upstream     │
│     Backend     │
└─────────────────┘
```

## 📦 Installation

### Prerequisites

- **Rust** 1.70+ ([Install Rust](https://rustup.rs/))
- **Node.js** 18+ ([Install Node.js](https://nodejs.org/))
- **npm** or **yarn**

### Build from Source

```
# Clone the repository
git clone https://github.com/aarambhdevhub/dev-proxy.git
cd dev-proxy

# Build UI
cd ui
npm install
npm run build
cd ..

# Build Rust proxy
cargo build --release

# Binary will be at ./target/release/dev-proxy
```

## 🚀 Usage

### Basic Usage

```
# Start proxy with recording enabled
./target/release/dev-proxy --record

# Access UI
open http://localhost:3000
```

### Command Line Options

```
dev-proxy [OPTIONS]

OPTIONS:
    -p, --port <PORT>              Proxy port [default: 8080]
    -u, --ui-port <UI_PORT>        UI port [default: 3000]
    --internal-port <PORT>         Internal Pingora port [default: 9090]
    -u, --upstream <UPSTREAM>      Upstream URL [default: http://localhost:8000]
    -r, --record                   Enable traffic recording
    -h, --help                     Print help information
```

### Examples

```
# Proxy to custom backend
./target/release/dev-proxy --upstream http://api.example.com --record

# Use custom ports
./target/release/dev-proxy --port 3000 --ui-port 8080 --record

# Proxy without recording (lightweight mode)
./target/release/dev-proxy --upstream http://localhost:5000
```

## 🎯 Using the Proxy

### Send Requests Through Proxy

```
# Make requests to the proxy (it forwards to upstream)
curl http://localhost:8080/api/users
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"John","email":"john@example.com"}'
```

### Create Mock Rules

1. Open UI at `http://localhost:3000/mocks`
2. Click **"+ New Mock"**
3. Configure your mock rule:
   - **Name**: Descriptive name for the mock
   - **Priority**: Higher priority rules match first
   - **Method**: HTTP method (or leave empty for any)
   - **URL Pattern**: `/api/users` (with match type)
   - **Match Type**: Choose from exact, contains, starts with, ends with, regex
   - **Status Code**: 200, 404, 500, etc.
   - **Delay**: Artificial delay in milliseconds
   - **Headers**: Response headers (one per line)
   - **Body**: Response body content

4. Click **"Create Mock Rule"**

### Example Mock Rules

**Mock a successful response:**
```
{
  "name": "Mock User List",
  "method": "GET",
  "url_pattern": "/api/users",
  "url_match_type": "contains",
  "response": {
    "status": 200,
    "headers": {
      "content-type": "application/json"
    },
    "body": "{\"users\": [{\"id\": 1, \"name\": \"John Doe\"}]}"
  }
}
```

**Mock an error response:**
```
{
  "name": "Simulate 500 Error",
  "method": "POST",
  "url_pattern": "/api/orders",
  "url_match_type": "exact",
  "response": {
    "status": 500,
    "headers": {
      "content-type": "application/json"
    },
    "body": "{\"error\": \"Internal Server Error\"}"
  },
  "delay_ms": 1000
}
```

**Mock with regex:**
```
{
  "name": "Mock User by ID",
  "url_pattern": "^/api/users/\\d+$",
  "url_match_type": "regex",
  "response": {
    "status": 200,
    "headers": {
      "content-type": "application/json"
    },
    "body": "{\"id\": 1, \"name\": \"Mocked User\"}"
  }
}
```

## 📊 Dashboard Features

### Recordings Page (`/`)
- View all captured HTTP requests/responses
- Filter by method, status code, duration
- Search in URLs and request/response bodies
- Click any request to see full details (headers, body, timing)

### Dashboard Page (`/dashboard`)
- **Traffic Statistics**: Total requests, success rate, error breakdown
- **HTTP Method Distribution**: Bar chart showing GET, POST, PUT, DELETE usage
- **Status Code Distribution**: Breakdown of 2xx, 3xx, 4xx, 5xx responses
- **Request Timeline**: Last hour of traffic with color-coded status bars
- **Top 10 Endpoints**: Most hit endpoints with avg duration and error rates

### Mocks Page (`/mocks`)
- Create, edit, delete mock rules
- Enable/disable rules with one click
- View all configured mocks with status indicators
- Priority-based rule matching

## 🔧 Configuration

### Environment Variables

```
# Set log level
export RUST_LOG=info

# Or for detailed logging
export RUST_LOG=debug
```

### Proxy Configuration

Edit `proxy/src/config.rs` for advanced configuration options.

## 🛠️ Development

### Project Structure

```
dev-proxy/
├── proxy/                  # Rust backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── proxy.rs       # Pingora proxy implementation
│   │   ├── http_layer.rs  # HTTP layer for mocking
│   │   ├── mock.rs        # Mock rule management
│   │   ├── storage.rs     # Recording storage
│   │   ├── recorder.rs    # Traffic recorder
│   │   ├── ui.rs          # UI server
│   │   └── config.rs      # Configuration
│   └── Cargo.toml
├── ui/                     # SvelteKit frontend
│   ├── src/
│   │   ├── routes/        # Pages
│   │   ├── lib/           # Components
│   │   └── app.css        # Styles
│   ├── package.json
│   └── svelte.config.js
└── README.md
```

### Running in Development Mode

**Backend:**
```
cd proxy
cargo watch -x run
```

**Frontend:**
```
cd ui
npm run dev
```

### Running Tests

```
# Rust tests
cd proxy
cargo test

# UI tests
cd ui
npm test
```


## 🎨 Tech Stack

### Backend
- **Rust** - Systems programming language
- **Pingora** - Cloudflare's HTTP proxy framework
- **Tokio** - Async runtime
- **Hyper** - HTTP library
- **Serde** - Serialization framework
- **Regex** - Pattern matching

### Frontend
- **SvelteKit** - Web framework
- **TypeScript** - Type-safe JavaScript
- **TailwindCSS** - Utility-first CSS
- **Chart.js** - Data visualization (optional)

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Pingora](https://github.com/cloudflare/pingora) - HTTP proxy framework by Cloudflare
- [SvelteKit](https://kit.svelte.dev/) - Web development framework
- [TailwindCSS](https://tailwindcss.com/) - CSS framework

## 📧 Contact

**Aaramb Dev** - [@aarambhdevhub](https://github.com/aarambhdevhub)

Project Link: [https://github.com/aarambhdevhub/dev-proxy](https://github.com/aarambhdevhub/dev-proxy)

---

**⭐ If you find this project useful, please consider giving it a star!**
