# Solution


## Contents
1. [Test Results Report](#test-results-report)
2. [Logical Bugs](#logical-bugs)
   1. [Error Handling when client Disconnected](#error-handling-when-client-disconnected)
   2. [Error Handling when Decoding Failed](#error-handling-when-decoding-failed)
   3. [Error Handling when unsupported messages received](#error-handling-when-unsupported-messages-received)
3. [Enhancements](#enhancements)
   1. [Multi Tasking](#multi-tasking)
   2. [Send function for the Client](#send-function-for-the-client)

---

---

## Test Results Report

### Report Running from README.md

> **Important Note**: The README.md provided command ```cargo test``` which cause problem for using parallel threads (it gives error: address already in use). Replaced with ```cargo test -- --test-threads=1```

```
/home/user/.cargo/bin/cargo test --color=always --profile test --no-fail-fast --config env.RUSTC_BOOTSTRAP=\"1\" -- --format=json --test-threads=1 -Z unstable-options --show-output
Testing started at 6:36 AM ...
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running unittests src/lib.rs (target/debug/deps/embedded_recruitment_task-a071409288bf5f32)
     Running tests/client.rs (target/debug/deps/client-32aa1960be0ef227)
     Running tests/client_test.rs (target/debug/deps/client_test-2fd245594b688c15)
Connecting to localhost:8080
Connected to the server!
Sent message: AddRequest(AddRequest { a: 10, b: 20 })
Sent message: AddResponse(AddResponse { result: 30 })
Disconnected from the server!
Connecting to localhost:8080
Connected to the server!
Disconnected from the server!
Connecting to localhost:8080
Connected to the server!
Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
Disconnected from the server!
Connecting to localhost:8080
Connected to the server!
Connecting to localhost:8080
Connected to the server!
Connecting to localhost:8080
Connected to the server!
Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
Sent message: EchoMessage(EchoMessage { content: "How are you?" })
Sent message: EchoMessage(EchoMessage { content: "How are you?" })
Sent message: EchoMessage(EchoMessage { content: "How are you?" })
Sent message: EchoMessage(EchoMessage { content: "How are you?" })
Sent message: EchoMessage(EchoMessage { content: "How are you?" })
Sent message: EchoMessage(EchoMessage { content: "How are you?" })
Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
Disconnected from the server!
Disconnected from the server!
Disconnected from the server!
Connecting to localhost:8080
Connected to the server!
Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
Sent message: EchoMessage(EchoMessage { content: "How are you?" })
Sent message: EchoMessage(EchoMessage { content: "How are you?" })
Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
Disconnected from the server!
   Doc-tests embedded_recruitment_task

Process finished with exit code 0

```

### Report Running from Terminal

```
[user@fedora embedded-recruitment_task_solution]$ cargo test -- --test-threads=1
   Compiling embedded-recruitment-task v0.1.0 (/home/user/Downloads/embedded-recruitment_task_solution)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.48s
     Running unittests src/lib.rs (target/debug/deps/embedded_recruitment_task-a071409288bf5f32)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/client.rs (target/debug/deps/client-32aa1960be0ef227)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/client_test.rs (target/debug/deps/client_test-2fd245594b688c15)

running 5 tests
test test_client_add_request ... ok
test test_client_connection ... ok
test test_client_echo_message ... ok
test test_multiple_clients ... ok
test test_multiple_echo_messages ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s

   Doc-tests embedded_recruitment_task

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```
---

---

## Logical Bugs
### Error Handling when client Disconnected

#### Problem
```rust
if bytes_read == 0 {
    info!("Client disconnected.");
    return Ok(()); // <- Error -------------
}
```
description: when the client disconnected it should return `Err` to tell the Thread to exit the loop
#### Solution
```rust
if bytes_read == 0 {
    info!("Client disconnected.");
    return Err(io::Error::new(ErrorKind::ConnectionReset, "Client disconnected."));
}
```
---

### Error Handling when Decoding Failed
#### problem
```rust
} else {
    error!("Failed to decode message"); // <--- Only Logging but no error handling
}
```
description: when decoding error happen there is no handling for it except logging it.

#### Solution
```rust
Err(e) => {
    error!("Failed to decode message: {}", e);
    
    // note the return Err here will cause the thread to exit
    // I considered getting an unsupported message means issue in the client side
    return Err(io::Error::new(ErrorKind::InvalidData, e));
}

```

---
### Error Handling when unsupported messages received

description: if a server received an unsupported message it should either request for it to be resent again or drop the connection since it's now could be considered corrupted (depend on the case)
<br>
I implemented to return an error to drop the thread (also the stream will be closed when the thread exits).
```rust
 // Handle any unrecognized message types (though unlikely)
_ => {

    error!("Received an unknown or unsupported message");
   
    // note the return Err here will cause the thread to exit
    // I considered getting an unsupported message means issue in the client side
    return Err(io::Error::new(ErrorKind::Unsupported, "Unknown message type"));
}
```


---

---


## Enhancements
### Multi Tasking
while coding for multi-threading I knew two options
1. **Spawn new thread for every connection** (Chosen Solution)
   <br>
   description: the server waits for new connection and when it has one it creates a new thread with the stream in it to process
   - Advantages
     <br>
      - only use number of threads that we need. So, no reserved resources is not used.
   - Disadvantages
     <br>
      - opening a lot of connections in the same time and exhaust the server resources and may cause a `denial-of-service` if the server crashed.
        <br>
2. **Using thread pool**
   <br>
   description: the server create a pool of threads (also called workers) and assign a thread with a client. the number of threads is determined before creating the pool.
   - Advantages
     <br>
      - good for preventing resources exhausting.
      - good for limiting number of open connections in the same time.
   - Disadvantages
     <br>
      - the number of threads of the pool actually is reserved for the threads only. (ex: thread pool with number of threads 500 -> 500 running threads even if there is no connection active)
      - choosing number of the threads in the pool opens an optimization problem to determine the best value for it (which  may be a headache if the load is changing).

---
### Send function for the Client
since handling different messages, but they are all from `server_message.Message` Enum, I unified the send function. to support all send classes.

```rust
pub fn send(&mut self, message: server_message::Message) -> io::Result<()> {

   // create buffer to hold the encoding
   let mut buffer = Vec::new();
   message.encode(&mut buffer);

   // Send the buffer to the server
   self.stream.write_all(&buffer)?;
   self.stream.flush()?;

   println!("Sent message: {:?}", message);
   Ok(())
}
```

