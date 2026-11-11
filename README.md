# load_balancer

A load balancer in Rust, built step by step, from a simple proxy up to a layer 7 proxy with caching and rate limiting.

## Current state

Minimal layer 4 proxy using only the standard library:

- accepts connections on `0.0.0.0:8080`
- forwards each request to **a single hardcoded backend** (`127.0.0.1:9001`)
- relays the backend's response back to the client
- handles **one client at a time**

```
client ──▶ load_balancer :8080 ──▶ backend :9001
client ◀── load_balancer :8080 ◀── backend :9001
```

## Running

In three terminals:

```sh
# 1. test backend
python3 -m http.server 9001

# 2. load balancer
cargo run

# 3. client
curl -v localhost:8080
```

`curl` should print the directory listing served by Python, and the Python log should show the request coming from the load balancer.

## Known limitations

- One client at a time: others wait in the queue.
- The request is read with a single 4096-byte `read`, so larger requests get truncated.
- Relaying the response only finishes when the backend closes the connection. This works with Python's `http.server`, but blocks with backends that keep connections alive.
- The backend address is hardcoded.

## Roadmap

Each item is meant to be one small commit.

### Phase 1: simple proxy
- [x] Accept connections and forward to a hardcoded backend
- [x] One thread per client
- [x] Copy both directions concurrently (`try_clone` + two threads)
- [x] Handle backend connection errors with clear messages

### Phase 2: multiple backends
- [ ] Backend list (`Vec<SocketAddr>`) instead of a single address
- [ ] Load backends from a config file
- [ ] Round-robin policy
- [ ] Least-connections policy

### Phase 3: failure handling
- [ ] Timeout on `connect`
- [ ] Retry on another backend if the connection fails
- [ ] Periodic background health checks
- [ ] Remove and restore backends from rotation based on health

### Phase 4: consistent hashing
- [ ] Hash ring with virtual nodes
- [ ] Pick the backend by client address
- [ ] Test: adding a backend only remaps ~1/N of clients

### Phase 5: layer 7
- [ ] Move to Tokio
- [ ] Parse requests with `hyper`
- [ ] Add a header with the original client address
- [ ] Rate limiter (token bucket per client)
- [ ] Response cache with expiration

### Phase 6: benchmarking
- [ ] Load tests with `oha` or `wrk`
- [ ] Results (requests/s and latency) in this README

### Phase 7 (optional): direct server return
- [ ] Test environment with network namespaces
- [ ] Stepping stone: MAC rewriting on the same local network (validates packet capture, flow hashing and backend setup before adding tunnels)
- [ ] Layer 3 balancer with IP-in-IP encapsulation
- [ ] Backends decapsulate via kernel tunnel interface
- [ ] Run in front of multiple proxy instances