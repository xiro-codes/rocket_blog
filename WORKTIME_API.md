# Work Time Tracker JSON API

The Work Time Tracker includes a pure JSON API for interacting with the application programmatically. The API is mounted under the `/api/worklog` prefix.

## Authentication

All API endpoints (except public ones, if any) require authentication via a session cookie named `token`. You can obtain this token by sending a POST request to the JSON login endpoint (`/api/worklog/login`), which will return the token in the response and also set it as an `HttpOnly` cookie.

### Login
- **URL**: `/api/worklog/login`
- **Method**: `POST`
- **Body**: `AccountFormDTO`
```json
{
  "username": "your_username",
  "password": "your_password"
}
```
- **Response**:
```json
{
  "status": "success",
  "token": "..."
}
```

## Endpoints

### 1. Get Summary Statistics
- **URL**: `/api/worklog/stats`
- **Method**: `GET`
- **Response**: `WorkTimeSummaryDTO`
```json
{
  "total_hours": 32.5,
  "total_earnings": 450.0,
  "currency": "USD",
  "entries_count": 5,
  "current_shift_earnings": 12.5,
  "pay_period_hours": 32.5
}
```

### 2. Get User Roles
- **URL**: `/api/worklog/roles`
- **Method**: `GET`
- **Response**: Array of `user_role` models
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "account_id": "...",
    "role_name": "Developer",
    "hourly_wage": 45.0,
    "currency": "USD",
    "is_tipped": false,
    "is_active": true,
    "created_at": "...",
    "updated_at": "..."
  }
]
```

### 3. Create a Role
- **URL**: `/api/worklog/roles`
- **Method**: `POST`
- **Body**: `UserRoleFormDTO`
```json
{
  "role_name": "Designer",
  "hourly_wage": "50.0",
  "currency": "USD",
  "is_tipped": false
}
```
- **Response**: The newly created `user_role` object.

### 4. Start Time Tracking
- **URL**: `/api/worklog/start`
- **Method**: `POST`
- **Body**: `TimeTrackingControlDTO`
```json
{
  "user_role_id": "550e8400-e29b-41d4-a716-446655440000"
}
```
- **Response**: The newly created `work_time_entry` object representing the active shift.

### 5. Stop Time Tracking
- **URL**: `/api/worklog/stop`
- **Method**: `POST`
- **Response**: Contains the completed entry and whether the role allows tips.
```json
{
  "entry": {
    "id": "...",
    "start_time": "...",
    "end_time": "...",
    "duration": 120,
    // ...
  },
  "is_tipped": false
}
```

### 6. Get Recent Work Entries
- **URL**: `/api/worklog/entries`
- **Method**: `GET`
- **Response**: Array of `WorkTimeEntryDisplayDTO`
```json
[
  {
    "id": "...",
    "start_time": "...",
    "end_time": "...",
    "start_time_display": "10/24 09:00",
    "end_time_display": "10/24 17:00",
    "duration": 480,
    "role_name": "Developer",
    "hourly_wage": 45.0,
    "currency": "USD",
    "earnings": 360.0
  }
]
```

### 7. Create Manual Entry
- **URL**: `/api/worklog/entries`
- **Method**: `POST`
- **Body**: `WorkTimeEntryFormDTO`
```json
{
  "user_role_id": "...",
  "start_time": "2024-10-24T09:00:00Z",
  "end_time": "2024-10-24T17:00:00Z"
}
```
- **Response**: The manually created `work_time_entry` object.

### 8. Add Tips to an Entry
- **URL**: `/api/worklog/entries/<entry_id>/tips`
- **Method**: `POST`
- **Body**: `TipEntryFormDTO`
```json
{
  "tip_amount": "25.50"
}
```
- **Response**: The updated `work_time_entry` object.

## Error Handling

If an error occurs, endpoints will respond with a JSON object containing an `error` field:
```json
{
  "error": "Error description here"
}
```
