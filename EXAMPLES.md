# Example: Spawn a Code Helper Agent

## Request
```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "code_helper",
    "prompt": "Write a Rust function that calculates fibonacci numbers",
    "tools_allowed": ["write_file", "read_file"],
    "system_append": "You are an expert Rust developer. Write clean, idiomatic code with proper error handling.",
    "create_session": true
  }'
```

## Expected Response (SSE Stream)

```
data: {"type":"session_created","session_id":"550e8400-e29b-41d4-a716-446655440000"}

data: {"type":"output","data":"{\"event\":\"text\",\"text\":\"I'll create a Rust function for calculating Fibonacci numbers.\"}"}

data: {"type":"output","data":"{\"event\":\"tool_use\",\"tool\":\"write_file\",\"input\":{\"path\":\"fibonacci.rs\",\"content\":\"pub fn fibonacci(n: u64) -> u64 {\\n    match n {\\n        0 => 0,\\n        1 => 1,\\n        _ => fibonacci(n - 1) + fibonacci(n - 2)\\n    }\\n}\"}}"}

data: {"type":"completed"}
```

---

# Example: Multi-turn Conversation

## First Turn - Create Session
```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "assistant",
    "prompt": "Create a simple web server in Rust using axum",
    "tools_allowed": ["write_file"],
    "create_session": true
  }'
```

**Save the session_id from the response**

## Second Turn - Resume Session
```bash
curl -N -X POST http://localhost:8080/message/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Add a health check endpoint",
    "tools_allowed": ["write_file", "read_file"]
  }'
```

---

# Example: Strict Tool Control

## Read-Only Agent
```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "analyzer",
    "prompt": "Analyze the code in src/main.rs and suggest improvements",
    "tools_allowed": ["read_file"],
    "create_session": false
  }'
```

## Write-Only Agent
```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "generator",
    "prompt": "Generate a new Rust project structure",
    "tools_allowed": ["write_file"],
    "create_session": false
  }'
```

---

# Example: Custom CLI Flags

```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "assistant",
    "prompt": "Help me debug this code",
    "flags": ["--max-tokens", "2000", "--temperature", "0.7"],
    "tools_allowed": ["read_file"],
    "create_session": false
  }'
```

---

# Example: Python Client

```python
import requests
import json

def create_agent_session(agent_type, initial_prompt, tools):
    """Create a new agent session and return the session ID."""
    url = "http://localhost:8080/spawn"
    payload = {
        "agent_type": agent_type,
        "prompt": initial_prompt,
        "tools_allowed": tools,
        "create_session": True
    }
    
    session_id = None
    response = requests.post(url, json=payload, stream=True)
    
    for line in response.iter_lines():
        if line and line.startswith(b'data: '):
            data = json.loads(line[6:])
            if data['type'] == 'session_created':
                session_id = data['session_id']
                print(f"✓ Session created: {session_id}")
            elif data['type'] == 'output':
                # Parse Claude's JSONL output
                claude_data = json.loads(data['data'])
                print(f"  {claude_data}")
            elif data['type'] == 'completed':
                print("✓ Turn completed")
                break
    
    return session_id

def send_message(session_id, prompt, tools):
    """Send a message to an existing session."""
    url = f"http://localhost:8080/message/{session_id}"
    payload = {
        "prompt": prompt,
        "tools_allowed": tools
    }
    
    response = requests.post(url, json=payload, stream=True)
    
    for line in response.iter_lines():
        if line and line.startswith(b'data: '):
            data = json.loads(line[6:])
            if data['type'] == 'output':
                claude_data = json.loads(data['data'])
                print(f"  {claude_data}")
            elif data['type'] == 'completed':
                print("✓ Turn completed")
                break

# Usage
session = create_agent_session(
    agent_type="code_assistant",
    initial_prompt="Create a simple REST API in Rust",
    tools=["write_file"]
)

if session:
    send_message(
        session_id=session,
        prompt="Add authentication to the API",
        tools=["write_file", "read_file"]
    )
```

---

# Example: JavaScript/Node.js Client

```javascript
const EventSource = require('eventsource');

async function spawnAgent(agentType, prompt, tools, createSession = false) {
  const response = await fetch('http://localhost:8080/spawn', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      agent_type: agentType,
      prompt: prompt,
      tools_allowed: tools,
      create_session: createSession
    })
  });

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let sessionId = null;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n');

    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = JSON.parse(line.slice(6));
        
        switch (data.type) {
          case 'session_created':
            sessionId = data.session_id;
            console.log(`✓ Session: ${sessionId}`);
            break;
          case 'output':
            const claudeData = JSON.parse(data.data);
            console.log('  Output:', claudeData);
            break;
          case 'error':
            console.error('  Error:', data.error);
            break;
          case 'completed':
            console.log('✓ Completed');
            return sessionId;
        }
      }
    }
  }

  return sessionId;
}

// Usage
(async () => {
  const session = await spawnAgent(
    'assistant',
    'Write a TypeScript interface for a User',
    ['write_file'],
    true
  );
  
  console.log('Session ID:', session);
})();
```

---

# Example: Terminating a Running Agent

```bash
# Start a long-running agent
SESSION_ID=$(curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "assistant",
    "prompt": "Generate documentation for all files in the project",
    "tools_allowed": ["read_file", "write_file"],
    "create_session": true
  }' | grep session_id | cut -d'"' -f4)

echo "Session ID: $SESSION_ID"

# Terminate it
curl -X POST http://localhost:8080/terminate/$SESSION_ID
```

---

# Example: List All Sessions

```bash
curl http://localhost:8080/sessions | jq
```

**Output:**
```json
{
  "sessions": [
    {
      "session_id": "550e8400-e29b-41d4-a716-446655440000",
      "agent_type": "code_helper",
      "created_at": 1698624000,
      "last_used": 1698624120
    },
    {
      "session_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
      "agent_type": "assistant",
      "created_at": 1698625000,
      "last_used": 1698625050
    }
  ]
}
```
