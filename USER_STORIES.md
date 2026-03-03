---
lang: en
---

# User Stories

<!--toc:start-->

- [View Home Screen](#view-home-screen)
  - [1. Acceptance Criteria](#1-acceptance-criteria)
- [Start Workout Session](#start-workout-session)
  - [2. Acceptance Criteria](#2-acceptance-criteria)
- [Cancel Empty Session](#cancel-empty-session)
  - [3. Acceptance Criteria](#3-acceptance-criteria)
- [Navigate to Exercise Browser](#navigate-to-exercise-browser)
  - [4. Acceptance Criteria](#4-acceptance-criteria)
- [Search Exercises in Browser](#search-exercises-in-browser)
  - [5. Acceptance Criteria](#5-acceptance-criteria)
- [Navigate to Analytics](#navigate-to-analytics)
  - [6. Acceptance Criteria](#6-acceptance-criteria)
- [Navigate to Credits](#navigate-to-credits)
  - [7. Acceptance Criteria](#7-acceptance-criteria)
- [Search Exercises in Active Session](#search-exercises-in-active-session)
  - [8. Acceptance Criteria](#8-acceptance-criteria)
- [Full Workout Session](#full-workout-session)
  - [9. Acceptance Criteria](#9-acceptance-criteria)
- [Remove Exercise from Session](#remove-exercise-from-session)
  - [10. Acceptance Criteria](#10-acceptance-criteria)
- [Delete a Past Session](#delete-a-past-session)
  - [11. Acceptance Criteria](#11-acceptance-criteria)
- [Repeat Session from History](#repeat-session-from-history)
  - [12. Acceptance Criteria](#12-acceptance-criteria)
- [Add Custom Exercise](#add-custom-exercise)
  - [13. Acceptance Criteria](#13-acceptance-criteria)

<!--toc:end-->

Comprehensive set of user stories for LogOut, a cross-platform workout logging
application. Each story is validated by Maestro end-to-end tests for each
supported platforms, currently :

- Web browser PWA : `maestro/web/`
- Android native app : `maestro/android/`

## View Home Screen

**As a** _user_, **I want to** see the home screen when I launch the app, **so
that** I know the app has loaded and I can start using it.

### 1. Acceptance Criteria

- The header displays "LogOut"
- The tagline "Log your workOut" is visible
- A bottom navigation bar with 4 tabs is present

## Start Workout Session

**As a** _user_, **I want to** start a new workout session, **so that** I can
begin logging my exercises.

### 2. Acceptance Criteria

- From Home: Tapping the "+" button opens the Active Session _view_
- The Active Session header displays "Active Session"

## Cancel Empty Session

**As a** _user_, **I want to** cancel an empty workout session, **so that** I
can return to the home screen without saving anything.

### 3. Acceptance Criteria

- From Active Session: "Cancel Session" button is visible if session is empty
- Tapping "Cancel Session" returns to the Home _view_
- The Home _view_ title "LogOut" is visible again

## Navigate to Exercise Browser

**As a** _user_, **I want to** navigate to the exercise database, **so that** I
can browse available exercises.

### 4. Acceptance Criteria

- Tapping the "📚" tab in the bottom navigation opens the exercise browser
- The exercise browser heading "Exercise Database" is visible
- Navigating back to the home tab shows the home screen again

## Search Exercises in Browser

**As a** _user_, **I want to** search for exercises in the exercise database,
**so that** I can quickly find a specific exercise.

### 5. Acceptance Criteria

- A search input is visible on the exercise browser page
- Typing a search term filters the exercise list
- The page remains functional after searching

## Navigate to Analytics

**As a** _user_, **I want to** navigate to the analytics page, **so that** I can
view my workout progress over time.

### 6. Acceptance Criteria

- Tapping the "📊" tab in the bottom navigation opens the analytics page
- The analytics heading "Analytics" is visible

## Navigate to Credits

**As a** _user_, **I want to** navigate to the credits page, **so that** I can
see information about the app and configure settings.

### 7. Acceptance Criteria

- Tapping the "ℹ️" tab in the bottom navigation opens the credits page
- The credits heading "Credits" is visible

## Search Exercises in Active Session

**As a** _user_, **I want to** search for exercises while in an active workout
session, **so that** I can find and add exercises to my current workout.

### 8. Acceptance Criteria

- An exercise search input is visible in the active session view
- Typing a search term does not crash the app
- The active session remains functional after searching

## Full Workout Session

**As a** _user_, **I want to** complete a full workout session from start to
finish, **so that** my workout is saved and visible in my session history.

### 9. Acceptance Criteria

- Start a new session and search for an exercise (e.g. "bench press")
- Select the exercise from the search results to open the exercise form
- Input weight and repetitions, then complete the exercise
- The completed exercise appears in the "Completed Exercises" section
- Replay the exercise (another set) using the 🔁 button
- Complete the second set successfully
- Finish the session using "Finish Session"
- The home screen shows the completed session with the exercise name

## Remove Exercise from Session

**As a** _user_, **I want to** remove a completed exercise from my active
session, **so that** I can correct mistakes or remove unwanted entries.

### 10. Acceptance Criteria

- Complete an exercise in an active session
- Delete the completed exercise using the 🗑️ button
- The session reverts to an empty state showing "Cancel Session"
- Cancelling the empty session returns to the home screen

## Delete a Past Session

**As a** _user_, **I want to** delete a completed session from my history, **so
that** I can remove unwanted or accidental entries.

### 11. Acceptance Criteria

- A completed session is visible on the home screen
- Tapping the 🗑️ button on a session card opens a confirmation dialog
- Confirming the deletion removes the session from the home screen
- The home screen returns to its empty state

## Repeat Session from History

**As a** _user_, **I want to** start a new session based on a past workout, **so
that** I can quickly repeat the same exercises.

### 12. Acceptance Criteria

- A completed session with at least one exercise is visible on the home screen
- Tapping the 🔄 button on a session card opens a new active session
- The new session shows the exercises from the original session in the
  "Pre-added Exercises" section

## Add Custom Exercise

**As a** _user_, **I want to** add a custom exercise to the exercise database,
**so that** I can log exercises that are not in the built-in list.

### 13. Acceptance Criteria

- Tapping the "+" button in the exercise browser opens the "Add Exercise" form
- Filling in the exercise name and saving creates a new exercise
- The new exercise is visible when searching in the exercise browser
