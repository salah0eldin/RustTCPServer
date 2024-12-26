# Solution
---

## Key Modifications

### 1. Transition to Multithreading
The original server implementation was single-threaded and could not handle concurrent client requests. The following improvements were made:
- **Multithreading**: Each incoming client connection now spawns a new thread for independent processing.
- **Synchronization**: Shared control of the server state is managed using an `Arc<AtomicBool>` to enable safe and consistent shutdown across threads.

### 2. Enhanced Client Handling
The client-handling logic was updated to:
- Decode and process client messages using Protocol Buffers (`prost::Message`).
- Support multiple message types, including `AddRequest` and `EchoMessage`.
- Implement a non-blocking I/O mechanism to handle idle clients efficiently.
- Gracefully manage edge cases such as connection resets, disconnections, and malformed requests.

### 3. Optimized Test Case Execution
Initially, test cases running in parallel caused conflicts due to shared port usage. Two solutions were implemented:
1. **Serial Execution**: Tests were temporarily run sequentially, eliminating port conflicts but slightly reducing execution speed.
2. **Port Isolation**: Each test case was assigned a unique port. This required substantial refactoring of the test suite, resulting in significantly improved efficiency and reliability.

---

## Code Changes

### Server Code
1. **Client Handling**:
   - Added decoding logic to process various message types (`AddRequest` and `EchoMessage`).
   - Introduced non-blocking I/O for smoother handling of idle or slow clients.
   - Enhanced error handling to better address disconnections and unexpected messages.

2. **Multithreading**:
   - Client handling was moved to separate threads to allow concurrent processing.
   - Thread-safe server control was ensured using `Arc<AtomicBool>`.

### Test Suite
1. **Sequential Execution**:
   - Adjusted the test suite to execute tests one at a time, avoiding port conflicts. (neglected due to long time)

2. **Port Isolation**:
   - Each test case now dynamically binds to a unique port, enabling parallel execution without conflicts.
   - Test logic was refactored to support dynamic port assignment, improving efficiency and scalability. (implemented)

---

## Challenges Encountered

1. **Race Conditions**:
   - Synchronization mechanisms were introduced to prevent data races and ensure thread safety.

2. **Test Refactoring**:
   - Significant restructuring of the test suite was required to implement port isolation, but the effort was necessary to achieve reliable and efficient execution.

---

## Future Recommendations

1. **Dynamic Port Allocation**:
   - Develop a centralized system for dynamic port assignment during testing.

2. **Logging Improvements**:
   - Enhance logging for better debugging and monitoring of client-server interactions. (couldn't finish due to time)

3. **Scalability**:
   - Conduct performance evaluations under high-load scenarios and implement additional optimizations as needed.

---

## Conclusion
The enhancements to the server and test suite resolved critical issues, improved efficiency, and ensured reliable operation under concurrent conditions. The updated implementation adheres to the functional and technical requirements outlined in the task specifications, making it robust and scalable for future needs.

