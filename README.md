# Trial License System - Local Demo Setup

## 📁 Project Structure

```
.
├── Cargo.lock
├── Cargo.toml
├── license-server
│   ├── Cargo.lock
│   ├── Cargo.toml
│   └── src
│       └── main.rs
├── README.md
└── trial-binary
    ├── Cargo.lock
    ├── Cargo.toml
    ├── README.md
    └── src
        ├── bin
        └── main.rs

6 directories, 10 files

```

# 1 - Run the server

# 2 - From trial-binary/ directory
```cargo run --bin get-license -- my-user-id```                     # Get license
```cargo run```                                                     # Run app
