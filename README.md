# Microservices Architecture for Archive and Notification Services

This project demonstrates a microservices architecture using Rust and gRPC for communication between services. The architecture consists of two primary microservices:

1. **Task Service**: Responsible for managing tasks such as creating archives, tracking progress, and stopping tasks.
2. **REST API Service**: Provides a RESTful API for clients to interact with the Task Service and a Notification Service via gRPC.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Services](#services)
  - [Task Service](#task-service)
  - [REST API Service](#rest-api-service)
- [Configuration](#configuration)
- [Build and Run](#build-and-run)
- [Usage](#usage)
- [Dependencies](#dependencies)

## Architecture Overview

The architecture is designed to separate concerns and improve scalability and maintainability. Each service is responsible for a specific domain of functionality and communicates with other services via gRPC.

## Services

### Task Service

**Description**: The Task Service handles tasks such as creating zip archives with passwords, tracking the progress of these tasks, and stopping tasks.

**gRPC API**:
- `EnqueueTask`: Accepts a request to create a new task for archiving files.
- `GetTaskProgress`: Returns the progress of a specified task.
- `StopTask`: Stops a specified task.

**Proto File**: `proto/task_service.proto`

### REST API Service

**Description**: The REST API Service provides a RESTful interface for clients to interact with the Task Service and the Notification Service. It forwards requests to the appropriate gRPC services and returns responses to the clients.

**REST Endpoints**:
- `POST /enqueue`: Enqueues a new task to create an archive.
- `GET /archive`: Retrieves a completed archive by task ID.
- `GET /progress`: Gets the progress of a specified task or all tasks.
- `GET /stop`: Stops a specified task.
- `POST /send_notification`: Sends a notification to a specified recipient.

## Configuration

Configuration for the services is stored in a `config.toml` file at the root of the project:

```toml
[task_service]
address = "http://[::1]:50051"

[notification_service]
address = "http://[::1]:50052"
```

## Build and Run

### Prerequisites

- Rust and Cargo
- Protocol Buffers compiler (`protoc`)

### Build

1. Generate the gRPC code from the proto files:

```sh
cargo build --release
```

### Run the Services

1. **Task Service**:

```sh
cd task_service
cargo run --release
```

2. **REST API Service**:

```sh
cd ../rest_api
cargo run --release
```

## Usage

### Enqueue Task

```sh
curl -X POST "http://localhost:9188/enqueue" \
     -H "Content-Type: multipart/form-data" \
     -F "archive_name=my_archive.zip" \
     -F "files=@/path/to/your/file1.txt" \
     -F "files=@/path/to/your/file2.txt"
```

### Get Archive

```sh
curl -X GET "http://localhost:9188/archive?taskId=your_task_id"
```

### Get Progress

```sh
curl -X GET "http://localhost:9188/progress?taskId=your_task_id"
```

### Stop Task

```sh
curl -X GET "http://localhost:9188/stop?taskId=your_task_id"
```

### Send Notification

```sh
curl -X POST "http://localhost:9188/send_notification" \
     -H "Content-Type: application/json" \
     -d '{"message": "Hello, World!", "recipient": "user@example.com"}'
```

## Dependencies

### Task Service

- `tokio`: Asynchronous runtime
- `tonic`: gRPC implementation
- `prost`: Protocol Buffers implementation
- `uuid`: Universally unique identifier
- `serde`: Serialization framework
- `chrono`: Date and time library
- `rand`: Random number generator
- `zip`: ZIP archive library

### REST API Service

- `actix-web`: Web framework
- `actix-rt`: Actix runtime
- `actix-multipart`: Multipart form data support
- `futures`: Asynchronous programming
- `serde`: Serialization framework
- `tokio`: Asynchronous runtime
- `tonic`: gRPC implementation
- `prost`: Protocol Buffers implementation
- `config`: Configuration management

This README provides a detailed overview of the microservices architecture, configuration, and usage of the services. It also includes instructions for building and running the services, as well as examples of how to interact with the REST API endpoints.

## How to Contribute 🤝

Contributions are welcome! If you have ideas for new features or improvements, feel free to fork the repository, make your changes, and submit a pull request.

## License 📄

kasl is open-source software licensed under the MIT license. See the [LICENSE](LICENSE) file for more details.