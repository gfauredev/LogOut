---
lang: en
---

# User Stories

<!--toc:start-->

- [Clean State Home](#clean-state-home)
  - [Acceptance Criteria](#acceptance-criteria)
- [Navigate to Exercise List](#navigate-to-exercise-list)
  - [Acceptance Criteria](#acceptance-criteria)
- [Clean State Exercise List](#clean-state-exercise-list)
  - [Acceptance Criteria](#acceptance-criteria)
- [Navigate to Analytics](#navigate-to-analytics)
  - [Acceptance Criteria](#acceptance-criteria)
- [Clean State Analytics](#clean-state-analytics)
  - [Acceptance Criteria](#acceptance-criteria)
- [Navigate to Credits](#navigate-to-credits)
  - [Acceptance Criteria](#acceptance-criteria)
- [Clean State Credits](#clean-state-credits)
  - [Acceptance Criteria](#acceptance-criteria)
- [Navigate Back to Home](#navigate-back-to-home)
  - [Acceptance Criteria](#acceptance-criteria)
- [Start Workout Session](#start-workout-session)
  - [Acceptance Criteria](#acceptance-criteria)
- [Cancel Empty Session](#cancel-empty-session)
  - [Acceptance Criteria](#acceptance-criteria)
- [Search Exercises in Browser](#search-exercises-in-browser)
  - [Acceptance Criteria](#acceptance-criteria)
- [Search Exercises in Active Session](#search-exercises-in-active-session)
  - [Acceptance Criteria](#acceptance-criteria)
- [Full Workout Session](#full-workout-session)
  - [Acceptance Criteria](#acceptance-criteria)
- [Remove Exercise from Session](#remove-exercise-from-session)
  - [Acceptance Criteria](#acceptance-criteria)
- [Delete a Past Session](#delete-a-past-session)
  - [Acceptance Criteria](#acceptance-criteria)
- [Repeat Session from History](#repeat-session-from-history)
  - [Acceptance Criteria](#acceptance-criteria)
- [Add Custom Exercise](#add-custom-exercise)
  - [Acceptance Criteria](#acceptance-criteria)

<!--toc:end-->

Comprehensive set of user stories for LogOut, a cross-platform workout logging
application. Each story is validated by Maestro end-to-end tests for each
supported platforms, currently :

- Web browser PWA : `maestro/web/`
- Android native app : `maestro/android/`

## Clean State Home

**As a** _user_ (and a _tester_), **I want to** see the Home in a clean, initial
state when I first launch the app, **so that** I’m not disturbed by existing
training Session(s) even though I didn’t record one yet.

### Acceptance Criteria

- Header contains "LogOut", "Log your workOut"
- Main body contains "No past sessions yet", "Tap + to start your first workout"

## Navigate to Exercise List

**As a** _user_, **I want to** navigate to the Exercise List, **so that** I can
browse available exercises.

### Acceptance Criteria

- The bottom navigation bar contains a "📚" button
  - **Click it**, the Exercise List page opens
- Header displays "📚 Exercise Database"
- "📚" button is slightly emphasized compared to others in bottom navigation bar

## Clean State Exercise List

**As a** _user_ (and a _tester_), **I want to** see the _Exercise List_ in a
clean, initial state when I first launch the app, **so that** I’m not disturbed
by existing custom Exercise(s) even though I didn’t created one yet.

### Acceptance Criteria

- There’s only default exercises, not custom ones created by a user
- The first exercise has the button "✏️ Clone & Edit", not just "✏️ Edit"

## Navigate to Analytics

**As a** _user_, **I want to** navigate to the analytics page, **so that** I can
view my workout progress over time.

### Acceptance Criteria

- The bottom navigation bar contains a "📊" button
  - **Click it**, the Analytics page opens
- Header displays "📊 Analytics"
- "📊" button is slightly emphasized compared to others in bottom navigation bar

## Clean State Analytics

**As a** _user_ (and a _tester_), **I want to** see the _Analytics_ page in a
clean, initial state when I first launch the app, **so that** I’m not disturbed
by existing training data even though I didn’t record any yet.

### Acceptance Criteria

- Select any metric in "Weight", "Repetitions", "Distance", "Duration"
- Then, no exercise can be selected, as none has been done yet

## Navigate to Credits

**As a** _user_, **I want to** navigate to the credits page, **so that** I can
see information about the app and configure settings.

### Acceptance Criteria

- The bottom navigation bar contains a "ℹ️" button
  - **Click it**, the Credits page opens
- Header displays "ℹ️ Credits"
- "ℹ️" button is slightly emphasized compared to others in bottom navigation bar

## Clean State Credits

**As a** _user_ (and a _tester_), **I want to** see the _Credits_ page in a
clean, initial state when I first launch the app, **so that** I’m not disturbed
by existing customization that shouldn’t be.

### Acceptance Criteria

- The database URL is the default one

## Navigate Back to Home

**As a** _user_, **I want to** navigate back to the home page, **so that** I can
return to my workout session history.

### Acceptance Criteria

- The bottom navigation bar contains a "💪" button
  - **Click it**, the Home page opens
- Header displays "💪 LogOut"
- "💪" button is slightly emphasized compared to others in bottom navigation bar

## Start Workout Session

**As a** _user_, **I want to** start a new workout session, **so that** I can
begin logging my exercises.

### Acceptance Criteria

- Tapping the "+" button opens the Active Session _view_
- Active Session
  - Header displays "Active Session", "Cancel Session" button and a timer
  - After 1 second, the timer has incremented by 1 second

## Cancel Empty Session

**As a** _user_, **I want to** cancel an empty workout session, **so that** I
can return to the home screen without saving a useless, empty session.

### Acceptance Criteria

- Active Session
  - "Cancel Session" button is visible if the Session is empty
  - Tapping "Cancel Session" returns to the Home _view_
- Home
  - Heading "LogOut" is visible again
  - Main body still shows "No past sessions yet"

## Search Exercises in Browser

**As a** _user_, **I want to** search for exercises in the exercise database,
**so that** I can quickly find a specific exercise.

### Acceptance Criteria

- A search input is visible on the exercise browser page
- Typing a search term filters the exercise list
- The page remains functional after searching

## Search Exercises in Active Session

**As a** _user_, **I want to** search for exercises while in an active workout
session, **so that** I can find and add exercises to my current workout.

### Acceptance Criteria

- An exercise search input is visible in the active session view
- Typing a search term does not crash the app
- The active session remains functional after searching

## Full Workout Session

**As a** _user_, **I want to** complete a full workout session from start to
finish, **so that** my workout is saved and visible in my session history.

### Acceptance Criteria

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

### Acceptance Criteria

- Complete an exercise in an active session
- Delete the completed exercise using the 🗑️ button
- The session reverts to an empty state showing "Cancel Session"
- Cancelling the empty session returns to the home screen

## Delete a Past Session

**As a** _user_, **I want to** delete a completed session from my history, **so
that** I can remove unwanted or accidental entries.

### Acceptance Criteria

- A completed session is visible on the home screen
- Tapping the 🗑️ button on a session card opens a confirmation dialog
- Confirming the deletion removes the session from the home screen
- The home screen returns to its empty state

## Repeat Session from History

**As a** _user_, **I want to** start a new session based on a past workout, **so
that** I can quickly repeat the same exercises.

### Acceptance Criteria

- A completed session with at least one exercise is visible on the home screen
- Tapping the 🔄 button on a session card opens a new active session
- The new session shows the exercises from the original session in the
  "Pre-added Exercises" section

## Add Custom Exercise

**As a** _user_, **I want to** add a custom exercise to the exercise database,
**so that** I can log exercises that are not in the built-in list.

### Acceptance Criteria

- Tapping the "+" button in the exercise browser opens the "Add Exercise" form
- Filling in the exercise name and saving creates a new exercise
- The new exercise is visible when searching in the exercise browser
