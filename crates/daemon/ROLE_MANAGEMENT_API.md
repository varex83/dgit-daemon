# Role Management API

This document describes the role management endpoints for the dgit daemon.

## Overview

The role management system supports two types of roles:
- **Pusher Role**: Allows pushing code to repositories
- **Admin Role**: Allows administrative operations on repositories

## Endpoints

### Grant Pusher Role

**POST** `/repo/{repo}/grant-pusher/{address}`

Grants pusher role to the specified address for the repository.

**Parameters:**
- `repo`: Repository name
- `address`: Ethereum address (e.g., `0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88`)

**Response:**
```json
{
  "repo": "my-repo",
  "address": "0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88",
  "role": "pusher",
  "granted": true
}
```

### Revoke Pusher Role

**POST** `/repo/{repo}/revoke-pusher/{address}`

Revokes pusher role from the specified address for the repository.

**Parameters:**
- `repo`: Repository name
- `address`: Ethereum address

**Response:**
```json
{
  "repo": "my-repo",
  "address": "0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88",
  "role": "pusher",
  "granted": false
}
```

### Grant Admin Role

**POST** `/repo/{repo}/grant-admin/{address}`

Grants admin role to the specified address for the repository.

**Parameters:**
- `repo`: Repository name
- `address`: Ethereum address

**Response:**
```json
{
  "repo": "my-repo",
  "address": "0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88",
  "role": "admin",
  "granted": true
}
```

### Revoke Admin Role

**POST** `/repo/{repo}/revoke-admin/{address}`

Revokes admin role from the specified address for the repository.

**Parameters:**
- `repo`: Repository name
- `address`: Ethereum address

**Response:**
```json
{
  "repo": "my-repo",
  "address": "0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88",
  "role": "admin",
  "granted": false
}
```

### Check Pusher Role

**GET** `/repo/{repo}/check-pusher/{address}`

Checks if the specified address has pusher role for the repository.

**Parameters:**
- `repo`: Repository name
- `address`: Ethereum address

**Response:**
```json
{
  "repo": "my-repo",
  "address": "0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88",
  "role": "pusher",
  "has_role": true
}
```

### Check Admin Role

**GET** `/repo/{repo}/check-admin/{address}`

Checks if the specified address has admin role for the repository.

**Parameters:**
- `repo`: Repository name
- `address`: Ethereum address

**Response:**
```json
{
  "repo": "my-repo",
  "address": "0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88",
  "role": "admin",
  "has_role": false
}
```

## Error Responses

All endpoints return HTTP 400 (Bad Request) with error message in case of failure:

```json
"Repository not found"
```

```json
"Invalid address format"
```

## Usage Examples

### Using curl

Grant pusher role:
```bash
curl -X POST http://localhost:3000/repo/my-repo/grant-pusher/0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88
```

Check if address has admin role:
```bash
curl http://localhost:3000/repo/my-repo/check-admin/0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88
```

### Integration with CLI

These endpoints will be integrated with the CLI tool for easier management:

```bash
# Grant permissions
dgit permissions grant my-repo 0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88 pusher

# Revoke permissions  
dgit permissions revoke my-repo 0x742d35Cc6463c48f1f08Db96Bc1B94F0De77fB88

# List permissions
dgit permissions list my-repo
```

## Notes

- Repository must exist before managing roles
- Ethereum addresses must be valid hex format (42 characters starting with 0x)
- Role operations are blockchain transactions and may take time to confirm
- Only authorized addresses can grant/revoke roles (depending on smart contract permissions) 