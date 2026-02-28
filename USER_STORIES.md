# User Stories

Comprehensive set of user stories for LogOut, a cross-platform workout logging
application. Each story is validated by two Maestro end-to-end tests: one
running against the PWA in a browser (`maestro/web/`) and one running on a
native Android build (`maestro/android/`).

## 1. View Home Screen

**As a** user,
**I want to** see the home screen when I launch the app,
**so that** I know the app has loaded and I can start using it.

### Acceptance Criteria

- The header displays "LogOut"
- The tagline "Log your workOut" is visible
- A bottom navigation bar with 4 tabs is present

## 2. Start Workout Session

**As a** user,
**I want to** start a new workout session,
**so that** I can begin logging my exercises.

### Acceptance Criteria

- Tapping the "+" button opens the active session view
- The active session header displays "Active Session"

## 3. Cancel Empty Session

**As a** user,
**I want to** cancel an empty workout session,
**so that** I can return to the home screen without saving anything.

### Acceptance Criteria

- A "Cancel Session" button is visible in an empty active session
- Tapping "Cancel Session" returns to the home screen
- The home screen title "LogOut" is visible again

## 4. Navigate to Exercise Browser

**As a** user,
**I want to** navigate to the exercise database,
**so that** I can browse available exercises.

### Acceptance Criteria

- Tapping the "📚" tab in the bottom navigation opens the exercise browser
- The exercise browser heading "Exercise Database" is visible
- Navigating back to the home tab shows the home screen again

## 5. Search Exercises in Browser

**As a** user,
**I want to** search for exercises in the exercise database,
**so that** I can quickly find a specific exercise.

### Acceptance Criteria

- A search input is visible on the exercise browser page
- Typing a search term filters the exercise list
- The page remains functional after searching

## 6. Navigate to Analytics

**As a** user,
**I want to** navigate to the analytics page,
**so that** I can view my workout progress over time.

### Acceptance Criteria

- Tapping the "📊" tab in the bottom navigation opens the analytics page
- The analytics heading "Analytics" is visible

## 7. Navigate to Credits

**As a** user,
**I want to** navigate to the credits page,
**so that** I can see information about the app and configure settings.

### Acceptance Criteria

- Tapping the "ℹ️" tab in the bottom navigation opens the credits page
- The credits heading "Credits" is visible

## 8. Search Exercises in Active Session

**As a** user,
**I want to** search for exercises while in an active workout session,
**so that** I can find and add exercises to my current workout.

### Acceptance Criteria

- An exercise search input is visible in the active session view
- Typing a search term does not crash the app
- The active session remains functional after searching
