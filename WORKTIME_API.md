# Work Time Tracker API Documentation

The Work Time API provides a JSON-based RESTful interface for external applications to interface with the Work Time Tracker backend.

## Base URL
The API endpoints are served under the `/api/worktime` path prefix (assuming the controller is mounted at that path). 

## Authentication
All endpoints (except `/login` and `/playground`) require authentication via a Bearer token in the `Authorization` header.

```http
Authorization: Bearer <your-uuid-token>
```

You can obtain this token by calling the `/login` endpoint.

---

## 🔐 Authentication

### POST `/login`
Authenticate a user and receive an API token.

**Request Body:**
```json
{
  "username": "your_username",
  "password": "your_password"
}
```

**Response:**
```json
{
  "status": "success",
  "token": "123e4567-e89b-12d3-a456-426614174000"
}
```

---

## 📊 Statistics

### GET `/stats`
Get summary statistics for the authenticated user's work time.

**Response:**
```json
{
  "total_hours": 42.5,
  "total_earnings": 1050.75,
  "currency": "USD",
  "entries_count": 15,
  "current_shift_earnings": 0.0,
  "pay_period_hours": 42.5
}
```

---

## 👥 Roles Management

### GET `/roles`
List all user roles.

**Response:** Array of role objects.

### POST `/roles`
Create a new user role.

**Request Body:**
```json
{
  "role_name": "Developer",
  "hourly_wage": "45.00",
  "currency": "USD",
  "is_tipped": false
}
```

### POST `/roles/<role_id>/edit`
Update an existing user role.

**Request Body:** Same as create role.

### DELETE `/roles/<role_id>`
Delete a user role.

---

## ⏱️ Time Tracking

### POST `/start`
Start a new active time tracking session.

**Request Body:**
```json
{
  "user_role_id": "123e4567-e89b-12d3-a456-426614174000"
}
```

### POST `/stop`
Stop the currently active time tracking session.

**Response:**
```json
{
  "entry": { ... },
  "is_tipped": false
}
```

---

## 📝 Work Entries

### GET `/entries`
List past work time entries formatted for display.

### GET `/entries/<entry_id>`
Get details for a specific entry.

### POST `/entries`
Manually create a new past work time entry.

**Request Body:**
```json
{
  "user_role_id": "123e4567-e89b-12d3-a456-426614174000",
  "start_time": "2023-10-25T09:00:00Z",
  "end_time": "2023-10-25T17:00:00Z"
}
```

### POST `/entries/<entry_id>/edit`
Update an existing entry.

**Request Body:** Same as manual creation.

### DELETE `/entries/<entry_id>`
Delete a work entry.

### POST `/entries/<entry_id>/tips`
Add tips to a completed entry (for tipped roles).

**Request Body:**
```json
{
  "tip_amount": "25.50"
}
```

---

## 📅 Pay Periods

### GET `/payperiods`
Get a list of pay periods along with summary data.

### POST `/payperiods`
Create a custom pay period.

**Request Body:**
```json
{
  "period_name": "October First Half",
  "start_date": "2023-10-01",
  "end_date": "2023-10-15"
}
```

### DELETE `/payperiods/<period_id>`
Delete a pay period.

### POST `/payperiods/assign`
Auto-assign any unassigned work entries to their appropriate pay periods based on dates.

---

## ⚙️ Settings & Notifications

### GET `/notifications`
Get notification settings for the user.

### POST `/notifications`
Update notification settings.

**Request Body:**
```json
{
  "time_based_enabled": true,
  "time_threshold_minutes": "480",
  "earnings_based_enabled": false,
  "earnings_threshold": "0",
  "currency": "USD",
  "daily_goal_enabled": true,
  "daily_hours_goal": "8"
}
```

### GET `/settings/timezone`
Get current timezone preference.

### POST `/settings/timezone`
Update timezone preference.

**Request Body:**
```json
{
  "timezone": "America/New_York"
}
```

### GET `/settings/payperiod`
Get default auto-pay period settings.

### POST `/settings/payperiod`
Update default pay period length and start day.

**Request Body:**
```json
{
  "start_day": "monday",
  "period_length": 2
}
```