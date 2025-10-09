# 🚀 Dev Proxy

A high-performance development proxy built with Rust and Pingora, featuring traffic recording, mock responses, response modification, rate limiting, latency injection, and real-time analytics dashboard.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![SvelteKit](https://img.shields.io/badge/sveltekit-%23f1413d.svg?style=for-the-badge&logo=svelte&logoColor=white)
![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)

## ✨ Features

### 🎯 Core Features
- **High-Performance Proxy** - Built on Cloudflare's Pingora framework
- **Traffic Recording** - Capture all HTTP requests and responses with full body content
- **Request Mocking** - Intercept requests and return custom responses without hitting backend
- **Response Modification** - Modify responses on-the-fly with powerful transformation rules
- **Rate Limiting** - Control request rates with flexible rate limiting strategies
- **Latency Injection** - Simulate network delays and test application resilience
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

### 🔧 Response Modification Features
- **Body Replacement** - Replace text patterns in response bodies with regex support
- **Header Manipulation** - Add or remove response headers dynamically
- **Status Code Changes** - Override backend status codes
- **JSON Modification** - Modify specific JSON fields in responses
- **Delay Injection** - Add artificial delays to specific endpoints
- **Priority-Based Rules** - Control modification order with priority system

### ⏱️ Latency Injection Features
- **Fixed Delays** - Add consistent delays to requests/responses
- **Random Delays** - Simulate variable network conditions (min-max range)
- **Normal Distribution** - Realistic latency patterns with mean and standard deviation
- **Spike Simulation** - Occasional high latency with configurable probability
- **Separate Request/Response** - Apply delays to requests, responses, or both
- **Pattern Matching** - Target specific endpoints with flexible URL matching

### 🚦 Rate Limiting Features
- **Per-IP Rate Limiting** - Limit requests by client IP address
- **Per-Header Rate Limiting** - Limit by custom header values (e.g., API keys)
- **Global Rate Limiting** - Shared rate limits across all clients
- **Custom Key Patterns** - Define custom rate limiting keys
- **Burst Capacity** - Allow temporary bursts above the normal limit
- **Configurable Responses** - Custom status codes, headers, and error messages
- **Bucket Reset** - Manual reset of rate limit counters per rule

## 🏗️ Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│   HTTP Layer    │ ◄── Handles rate limits, mocks, latency (Port 8080)
│   (Port 8080)   │
└────────┬────────┘
         │
         ▼
    ┌────────┐
    │ Rate   │──Limited──► Return 429 Response
    │ Limit? │
    └───┬────┘
        │Pass
        ▼
    ┌────────┐
    │ Latency│──Delay──► Apply Request Delay
    │ Inject?│
    └───┬────┘
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
└────────┬────────┘
         │
         ▼
    ┌────────┐
    │Modifier│──Apply──► Modify Response
    └───┬────┘
        │
        ▼
    ┌────────┐
    │Latency │──Delay──► Apply Response Delay
    │ Inject?│
    └───┬────┘
        │
        ▼
┌─────────────────┐
│   Response to   │
│     Client      │
└─────────────────┘
```

## 📦 Installation

### Prerequisites

- **Rust** 1.70+ ([Install Rust](https://rustup.rs/))
- **Node.js** 18+ ([Install Node.js](https://nodejs.org/))
- **npm** or **yarn**

### Build from Source

```bash
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

```bash
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

```bash
# Proxy to custom backend
./target/release/dev-proxy --upstream http://api.example.com --record

# Use custom ports
./target/release/dev-proxy --port 3000 --ui-port 8080 --record

# Proxy without recording (lightweight mode)
./target/release/dev-proxy --upstream http://localhost:5000
```

## 🎯 Using the Proxy

### Send Requests Through Proxy

```bash
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

### Create Response Modification Rules

1. Open UI at `http://localhost:3000/modifiers`
2. Click **"+ New Modifier"**
3. Configure your modifier rule:
   - **Name**: Descriptive name
   - **Priority**: Execution order (higher first)
   - **Match Request**: URL pattern and optional method
   - **Match Type**: exact, contains, regex, starts with, ends with
   - **Status Codes**: Apply only to specific status codes (optional)
   - **Modifications**: Add multiple modifications:
     - Replace Body: Find and replace text/regex patterns
     - Add Header: Insert custom headers
     - Remove Header: Remove existing headers
     - Change Status: Override status code
     - Inject Delay: Add artificial delay
     - Modify JSON: Change specific JSON fields
4. Click **"Create Modifier Rule"**

### Create Rate Limit Rules

1. Open UI at `http://localhost:3000/rate-limits`
2. Click **"+ New Rate Limit"**
3. Configure your rate limit rule:
   - **Name**: Descriptive name
   - **Priority**: Rule matching order
   - **URL Pattern**: Target endpoints
   - **Method**: Optional HTTP method filter
   - **Rate Limit Key**: Choose from:
     - Global: Shared limit across all clients
     - Per IP Address: Individual limit per client IP
     - Per Header: Limit by header value (e.g., API key)
     - Custom: Define custom key pattern
   - **Max Requests**: Number of allowed requests
   - **Window (seconds)**: Time window for the limit
   - **Burst Size**: Optional extra capacity for bursts
   - **Response**: Custom 429 response configuration
4. Click **"Create Rule"**

### Create Latency Injection Rules

1. Open UI at `http://localhost:3000/latency`
2. Click **"+ New Latency Rule"**
3. Configure your latency rule:
   - **Name**: Descriptive name
   - **Priority**: Rule matching order
   - **URL Pattern**: Target endpoints
   - **Method**: Optional HTTP method filter
   - **Apply To**: Request, Response, or Both
   - **Delay Type**: Choose from:
     - Fixed: Constant delay
     - Random: Variable delay (min-max)
     - Normal: Bell curve distribution
     - Spike: Occasional high latency
   - **Delay Parameters**: Configure based on delay type
4. Click **"Create Rule"**

## 📊 Dashboard Pages

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

### Modifiers Page (`/modifiers`)
- Create, edit, delete response modification rules
- Enable/disable rules individually
- View all modification rules with details
- Test modifications in real-time

### Rate Limits Page (`/rate-limits`)
- Create, edit, delete rate limiting rules
- Enable/disable limits with one click
- View active bucket statistics
- Reset rate limit counters per rule
- Monitor rate limit hits and rejections

### Latency Page (`/latency`)
- Create, edit, delete latency injection rules
- Enable/disable latency simulation
- View latency statistics (min, max, avg)
- Monitor latency injections per rule
- Test different delay patterns

## 🔧 Configuration

### Environment Variables

```bash
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
│   │   ├── modifier.rs    # Response modification
│   │   ├── ratelimiter.rs # Rate limiting logic
│   │   ├── latencyinjector.rs # Latency injection
│   │   ├── storage.rs     # Recording storage
│   │   ├── recorder.rs    # Traffic recorder
│   │   ├── ui.rs          # UI server
│   │   └── config.rs      # Configuration
│   └── Cargo.toml
├── ui/                     # SvelteKit frontend
│   ├── src/
│   │   ├── routes/        # Pages
│   │   │   ├── /          # Recordings
│   │   │   ├── dashboard/ # Analytics
│   │   │   ├── mocks/     # Mock management
│   │   │   ├── modifiers/ # Response modification
│   │   │   ├── rate-limits/ # Rate limiting
│   │   │   └── latency/   # Latency injection
│   │   ├── lib/           # Components
│   │   └── app.css        # Styles
│   ├── package.json
│   └── svelte.config.js
├── example-backend/        # Example backend for testing
└── README.md
```

### Running in Development Mode

**Backend:**
```bash
cd proxy
cargo watch -x run
```

**Frontend:**
```bash
cd ui
npm run dev
```

### Running Tests

```bash
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
- **Reqwest** - HTTP client
- **Serde** - Serialization framework
- **Regex** - Pattern matching
- **Parkinglot** - Efficient synchronization primitives

### Frontend
- **SvelteKit** - Web framework
- **TypeScript** - Type-safe JavaScript
- **TailwindCSS** - Utility-first CSS

## 🔥 Key Features Explained

### Response Modification
Transform responses on-the-fly without changing backend code:
- Replace sensitive data in responses
- Add CORS headers dynamically
- Modify JSON responses for testing
- Override error responses
- Inject delays for specific endpoints

### Rate Limiting
Protect your APIs and test rate limiting behavior:
- Multiple key types (IP, header, global, custom)
- Configurable window and burst capacity
- Custom error responses
- Real-time statistics and monitoring
- Manual bucket reset capability

### Latency Injection
Simulate real-world network conditions:
- Test timeout handling
- Verify loading states
- Simulate degraded performance
- Model realistic latency patterns
- Separate request/response delays

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
