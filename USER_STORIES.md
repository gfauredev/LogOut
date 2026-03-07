---
lang: en
---

# User Stories

<!--toc:start-->

- [Clean State Home](#clean-state-home)
- [Allow Notifications](#allow-notifications)
- [Navigate to Credits](#navigate-to-credits)
- [Clean State Credits](#clean-state-credits)
- [Change Exercises Database](#change-exercises-database)
- [Navigate to Exercise List](#navigate-to-exercise-list)
- [Clean State Exercise List](#clean-state-exercise-list)
- [Search Exercises in Browser](#search-exercises-in-browser)
- [Learn About an Exercise](#learn-about-an-exercise)
- [Add Custom Exercise](#add-custom-exercise)
- [Edit a Cloned Exercise](#edit-a-cloned-exercise)
- [Navigate to Analytics](#navigate-to-analytics)
- [Clean State Analytics](#clean-state-analytics)
- [Navigate Back to Home](#navigate-back-to-home)
- [Full Workout Session](#full-workout-session)
  - [Start Session](#start-session)
  - [Search Exercise](#search-exercise)
  - [Record Exercise](#record-exercise)
  - [Replay Exercise](#replay-exercise)
  - [Repeat For Another Exercise](#repeat-for-another-exercise)
  - [Remove Exercise](#remove-exercise)
  - [Finish Session](#finish-session)
- [Cancel Empty Session](#cancel-empty-session)
- [Repeat Session from History](#repeat-session-from-history)
- [Delete Past Sessions](#delete-past-sessions)
- [Lookup a Previously Done Exercise](#lookup-a-previously-done-exercise)
- [Be Astonished by Your Incredible Progress](#be-astonished-by-your-incredible-progress)

<!--toc:end-->

Comprehensive user stories for [LogOut] that serve as a basis for end-to-end
tests for all supported platforms.

- Web browser PWA test via [Maestro] in `maestro/web/`
- Android native app via [Maestro] in `maestro/android/`

Each user story corresponds to an independent, isolated test, but some might
need preconditions like pre-existing state (e.g. a completed session).
Preconditions may be satisfied by executing other user stories or common sub
flows before. No global ordering between tests is assumed nor required, and
tests don’t need to care about the state left.

## Clean State Home

**As a** _user_ and a _tester_, **I want to** see the Home in an initial state
when I first launch the app, **so that** I’m not disturbed by existing training
Session(s) that shouldn’t.

### Preconditions

- No sessions present: either the app has never been used, or all previous
  sessions have been deleted (see _Delete Past Sessions_)

### Acceptance Criteria

- Header contains strings
  - "💪 LogOut"
  - "Turn off your computer, Log your workOut"
- Main body contains strings
  - "No past sessions yet"
  - "Tap + to start your first workout"

## Allow Notifications

**As a** _user_, **I want to** allow notifications when prompted, **so that** I
can receive exercise and rest duration reminders.

### Preconditions

- Notification permission has not yet been granted or blocked in the browser

### Acceptance Criteria

- A toast is present containing "⚠️ Tap here to enable notifications"
- When clicked, the notifications’ permission dialog opens
- When notifications are denied, a toast with "⚠️ Notifications blocked" appears
- When notifications are allowed in browser settings, the toast disappears

## Navigate to Credits

**As a** _user_, **I want to** navigate to the credits page, **so that** I can
see information about the app and configure settings.

### Preconditions

- Be on a page different than credits

### Acceptance Criteria

- The bottom navigation bar contains a "ℹ️" button
  - When **clicked**, it opens the Credits page
- Header displays "ℹ️ Credits"
- "ℹ️" button is slightly emphasized compared to others in bottom navigation bar

## Clean State Credits

**As a** _user_ and a _tester_, **I want to** see the _Credits_ page in a clean,
initial state when I first launch the app, **so that** I’m not disturbed by
existing customization that shouldn’t be.

### Preconditions

- The exercise database URL has not been changed from the default value

### Acceptance Criteria

- The database URL input shows the default URL
  (`https://raw.githubusercontent.com/gfauredev/free-exercise-db/main/`)

## Change Exercises Database

**As a** _user_ and a _tester_, **I want to** change the exercises database URL,
**so that** I can use a custom database instead of the default one.

### Acceptance Criteria

- Input the test exercise database address (`http://localhost:8080`)
- Click the save button, database URL is saved without errors

## Navigate to Exercise List

**As a** _user_, **I want to** navigate to the Exercise List, **so that** I can
browse available exercises.

### Preconditions

- Be on a page other than Exercise List

### Acceptance Criteria

- The bottom navigation bar contains a "📚" button
  - When **clicked**, it opens the Exercise List page
- Header displays "📚 Exercises"
- "📚" button is slightly emphasized compared to others in bottom navigation bar

## Clean State Exercise List

**As a** _user_ and a _tester_, **I want to** see the Exercise List in an
initial state when I first launch the app, **so that** I’m not disturbed by
existing custom Exercise(s) that shouldn’t.

### Preconditions

- No custom exercises added yet (custom exercises cannot be deleted)

### Acceptance Criteria

- There are only built-in exercises, no custom ones created by a user
- Built-in exercises show a "+" button (title: "Clone then edit"), not a "✏️"
  button
- A "✏️" button (Edit) is only shown on custom exercises

## Search Exercises in Browser

**As a** _user_, **I want to** search for exercises of the database via the
Exercise List, **so that** I can quickly find a specific one.

### Acceptance Criteria

- Exercise List page header displays a search input
- Typing a search term filters the exercise list
- Exercises matching the search filter are visible, others are hidden
- Multi-word queries are error-tolerant: each word is matched independently so
  that e.g. "wide grip bench" finds "Wide-Grip Barbell Bench Press"
- The page remains functional after searching
- Removing search term(s) shows the full exercise list again

## Learn About an Exercise

**As a** _user_, **I want to** learn about an exercise details, **so that** I
can perform it properly and confidently.

### Acceptance Criteria

- Clicking on an Exercise heading displays the exercise’s instructions
- Clicking on an Exercise image cycles to the next image
- A row of tags displays Category, Force, Equipment, and Level of the exercise
- A row of tags displays Primary Muscle(s) targeted by the exercise
- A row of tags displays Secondary Muscle(s) targeted by the exercise

> Not every exercise has every detail available, so try other exercise instead
> of failing the test in case something is missing

## Add Custom Exercise

**As a** _user_, **I want to** add a custom exercise to the database, **so
that** I can log exercises that are not in the built-in list.

### Acceptance Criteria

- A "+" button is located near the search bar
  - When **clicked**, it opens the Add Exercise form
- Filling in the exercise name and clicking save button creates a new exercise
- The new exercise is visible in the Exercise List

## Edit a Cloned Exercise

**As a** _user_, **I want to** edit a built-in exercise after cloning it, **so
that** I can create a custom clone that fits my training better.

### Acceptance Criteria

- A "+" button (titled "Clone then edit") is located on each built-in exercise
  card
  - When **clicked**, it clones the exercise and opens the Edit Exercise form
- Changing the exercise name and clicking save button creates a new exercise
- The modified clone is visible in the Exercise List with the new name

## Navigate to Analytics

**As a** _user_, **I want to** navigate to the analytics page, **so that** I can
view my workout progress over time.

### Preconditions

- Be on a page different than analytics

### Acceptance Criteria

- The bottom navigation bar contains a "📊" button
  - When **clicked**, it opens the Analytics page
- Header displays "📊 Analytics"
- "📊" button is slightly emphasized compared to others in bottom navigation bar

## Clean State Analytics

**As a** _user_ and a _tester_, **I want to** see the _Analytics_ page in an
initial state when I first launch the app, **so that** I’m not disturbed by
existing training data that shouldn’t.

### Preconditions

- No completed sessions present: either the app has never been used, or all
  sessions have been deleted (see _Delete Past Sessions_)

### Acceptance Criteria

- Select any metric in "Weight", "Repetitions", "Distance", "Duration"
- Then, no exercise can be selected, as none has been done yet

## Navigate Back to Home

**As a** _user_, **I want to** navigate back to the home page, **so that** I can
return to my workout session (history).

### Preconditions

- Be on a page different than home

### Acceptance Criteria

- The bottom navigation bar contains a "💪" button
  - When **clicked**, it opens the Home page
- Header displays "💪 LogOut"
- "💪" button is slightly emphasized compared to others in bottom navigation bar

## Full Workout Session

### Start Session

**As a** _user_, **I want to** start a new workout session, **so that** I can
begin logging my exercises.

#### Acceptance Criteria

- The Home page contains a "+" button
  - When **clicked**, opens the Active Session _view_
- Active Session
  - Header displays "⏱️ Active Session", an elapsed timer, and a session button
  - When the session is empty, the button shows "❌" (Cancel Session)
  - After 1 second, the timer has incremented by 1 second

### Search Exercise

**As a** _user_, **I want to** search for an exercise in my active session, **so
that** I can log it.

#### Acceptance Criteria

- Input an exercise name, category, or muscle in the search bar (multi-word
  search is supported, e.g. "wide grip bench" finds "Wide-Grip Barbell Bench
  Press")
- Select an exercise from the search results to open the exercise form

### Record Exercise

**As a** _user_, **I want to** record a completed exercise in my active session,
**so that** I can track my progress.

#### Acceptance Criteria

- Input a random number between 0 and 99 inclusive in the first metric
- If there’s a second metric, tap a dozen times its "+" button
- **Click** on the "✓ Complete Exercise" button
- Completed exercise appears in the "Completed Exercises" section, with metrics

### Replay Exercise

**As a** _user_, **I want to** replay a completed exercise in my active session,
**so that** I can quickly perform another set.

#### Acceptance Criteria

- **Click** on the 🔁 button to replay the exercise (do another set)
- Slightly increment some non-null metric(s)
- **Click** on the "✓ Complete Exercise" button
- Second set appears in the "Completed Exercises" section, with higher metrics

### Repeat For Another Exercise

**As a** _user_, **I want to** repeat the previous steps for another exercise,
**so that** I’m able to record a varied session.

- Repeat the 3 previous steps in order to do sets of another different exercise
- Same acceptance criteria applies at each step

### Remove Exercise

**As a** _user_, **I want to** remove a completed exercise from my active
session, **so that** I can correct mistakes or remove unwanted entries.

#### Acceptance Criteria

- **Click** the 🗑️ button on the latest exercise to remove it
- There’s only 3 completed exercises left in the Active Session

### Finish Session

**As a** _user_, **I want to** finish my active session, **so that** I can save
it in my history and see my progress over time.

#### Acceptance Criteria

- Tap the "✅" (Finish Session) button in the active session header
- The home screen shows the completed session with the current date
- A congratulation toast appears

## Cancel Empty Session

**As a** _user_, **I want to** cancel an empty workout session, **so that** I
can return to the home screen without saving a useless, empty session.

### Preconditions

- An empty active session has been started (no exercises recorded yet)
- The E2E test starts a fresh session itself, so no external setup is needed

### Acceptance Criteria

- Active Session
  - "Cancel Session" button is visible if the Session is empty
  - Tapping "Cancel Session" returns to the Home _view_
- Home
  - Heading "💪 LogOut" is visible again
  - Main body still shows "No past sessions yet"

## Repeat Session from History

**As a** _user_, **I want to** start a new session based on a past workout, **so
that** I can quickly repeat the same exercises.

### Preconditions

- At least one completed workout session with at least one exercise exists; the
  E2E test creates this session itself using a setup subflow

### Acceptance Criteria

- A completed session with at least one exercise is visible on the home screen
- **Click** 🔄 button on a previous Session card, a new Active Session opens
- The "Pre-added Exercises" section contains the previous session exercises

## Delete Past Sessions

**As a** _user_, **I want to** delete a completed session from my history, **so
that** I can remove unwanted or accidental entries.

### Preconditions

- At least one completed workout session with at least one exercise exists; the
  E2E test creates this session itself using a setup subflow

### Acceptance Criteria

- **Click** 🗑️ button on a Session card, a confirmation dialog opens
- Clicking "🗑️ Delete" in the dialog removes the session from the home screen
- After all sessions are deleted, the main body shows "No past sessions yet"

## Lookup a Previously Done Exercise

**As a** _user_, **I want to** look up exercises I have previously done, **so
that** I can find similar exercises.

### Preconditions

- At least one completed workout session with at least one exercise exists; the
  E2E test creates this session itself using a setup subflow

### Acceptance Criteria

- The Session card contains done exercise(s) tags
  - **Click one** of the tags, the Exercise List opens
- Exercise List
  - The search bar is pre-filled with the name of the selected exercise

## Be Astonished by Your Incredible Progress

**As a** _user_, **I want to** see my workout progress over time in the
analytics page, **so that** I can be astonished by my incredible progress and
stay motivated to continue improving further.

### Preconditions

- At least two sets of the same exercise completed with an improved metric; the
  E2E test creates this data itself (bench press set 1: 10 reps, set 2: 12 reps)

### Acceptance Criteria

- Select a metric worked in the previous full session
  - Among "Weight", "Repetitions", "Distance", "Duration"
- Select two exercises for which this metric has improved between previous sets
- Be astonished by the raising curves in the analytics charts

[LogOut]: https://gfauredev.github.io/LogOut
[Maestro]: https://maestro.dev
