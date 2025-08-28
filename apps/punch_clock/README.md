# Punch Clock App

A work time tracking application built with Rust and Rocket framework that allows users to clock in/out for different roles and track their earnings.

## Features

### For Workers
- **Clock In/Out**: Start and stop work sessions for different roles
- **Dashboard**: View current status, today's work, and total statistics
- **Work History**: Complete history of all work sessions
- **Automatic Calculations**: Time and earnings are calculated automatically

### For Administrators
- **Role Management**: Create, edit, and delete work roles
- **Rate Management**: Set different hourly rates for different roles
- **Role Status**: Enable/disable roles for availability

## Database Schema

### WorkRole Table
- `id` (UUID): Primary key
- `name` (String): Role name (e.g., "Developer", "Designer")
- `hourly_rate` (Decimal): Rate per hour for this role
- `is_active` (Boolean): Whether role is available for clock-in
- `created_at`, `updated_at` (DateTime): Timestamps

### WorkSession Table
- `id` (UUID): Primary key
- `account_id` (UUID): Foreign key to user account
- `work_role_id` (UUID): Foreign key to work role
- `clock_in_time` (DateTime): When work session started
- `clock_out_time` (DateTime, nullable): When work session ended
- `duration_minutes` (Integer, nullable): Total minutes worked
- `earnings` (Decimal, nullable): Amount earned for this session
- `created_at`, `updated_at` (DateTime): Timestamps

## API Endpoints

### User Endpoints
- `GET /punch-clock/` - Dashboard view
- `GET /punch-clock/clock-in` - Clock in form
- `POST /punch-clock/clock-in` - Process clock in
- `POST /punch-clock/clock-out` - Process clock out
- `GET /punch-clock/history` - Work history view

### Admin Endpoints
- `GET /punch-clock/roles` - Role management view
- `GET /punch-clock/roles/create` - Create role form
- `POST /punch-clock/roles/create` - Process role creation
- `GET /punch-clock/roles/{id}/edit` - Edit role form
- `POST /punch-clock/roles/{id}/edit` - Process role update
- `POST /punch-clock/roles/{id}/delete` - Delete role

## Business Logic

### Clock In Process
1. Check if user already has an active session (prevent double clock-in)
2. Validate selected role exists and is active
3. Create new work session with current timestamp
4. Return success confirmation

### Clock Out Process
1. Find user's active work session
2. Calculate duration in minutes
3. Calculate earnings: `(minutes / 60) * hourly_rate`
4. Update session with clock out time, duration, and earnings
5. Return success with work summary

### Earnings Calculation
```
hours_worked = duration_minutes / 60
earnings = hours_worked * role.hourly_rate
```

## Authentication

The app uses request guards to protect endpoints:
- `User` guard: Requires authenticated user (for all work tracking features)
- `Admin` guard: Requires admin privileges (for role management)

Authentication is handled through secure HTTP-only cookies containing user tokens.

## Usage Example

1. **Admin creates roles**:
   - "Software Developer" at $50/hour
   - "Code Reviewer" at $40/hour
   - "Meeting Facilitator" at $35/hour

2. **Worker clocks in**:
   - Selects "Software Developer" role
   - Clicks "Clock In" at 9:00 AM

3. **Worker clocks out**:
   - Clicks "Clock Out" at 12:30 PM
   - System calculates: 3.5 hours × $50 = $175 earned

4. **Dashboard shows**:
   - Today: 3h 30m worked, $175 earned
   - Total: Updated with all-time statistics

## Running the Application

```bash
# Build the application
cargo build -p punch_clock

# Run tests
cargo test -p punch_clock

# Run the application
cd apps/punch_clock
cargo run
```

The app will be available at `http://localhost:8001/punch-clock`

## Dependencies

- **Rocket**: Web framework
- **SeaORM**: Database ORM
- **Tera**: Template engine
- **rust_decimal**: Precise decimal arithmetic for money calculations
- **chrono**: Date and time handling
- **uuid**: Unique identifiers

## Database Migration

The punch clock tables are automatically created through SeaORM migrations when the application starts. The migration file is located at:
`shared/migrations/src/m20241213_000001_add_punch_clock.rs`