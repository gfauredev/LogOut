---
lang: en
---

# User Stories

<!--toc:start-->

- [Clean State Home](#clean-state-home)
- [Navigate to Exercise List](#navigate-to-exercise-list)
- [Clean State Exercise List](#clean-state-exercise-list)
- [Search Exercises in Browser](#search-exercises-in-browser)
- [Learn About an Exercise](#learn-about-an-exercise)
- [Add Custom Exercise](#add-custom-exercise)
- [Edit a Cloned Exercise](#edit-a-cloned-exercise)
- [Navigate to Analytics](#navigate-to-analytics)
- [Clean State Analytics](#clean-state-analytics)
- [Navigate to Credits](#navigate-to-credits)
- [Clean State Credits](#clean-state-credits)
- [Navigate Back to Home](#navigate-back-to-home)
- [Start Workout Session](#start-workout-session)
- [Cancel Empty Session](#cancel-empty-session)
- [Full Workout Session](#full-workout-session)
  - [Start Session](#start-session)
    - [Search Exercise](#search-exercise)
    - [Record Exercise](#record-exercise)
    - [Replay Exercise](#replay-exercise)
    - [Repeat For Another Exercise](#repeat-for-another-exercise)
  - [Remove Exercise](#remove-exercise)
    - [Finish Session](#finish-session)
  - [Repeat Session from History](#repeat-session-from-history)
- [Delete a Past Session](#delete-a-past-session)
- [Lookup a Previously Done Exercise](#lookup-a-previously-done-exercise)
- [Navigate Again to Analytics](#navigate-again-to-analytics)
- [Be Astonished by Your Incredible Progress](#be-astonished-by-your-incredible-progress)

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

## Search Exercises in Browser

**As a** _user_, **I want to** search for exercises of the exercise database via
the Exercise List, **so that** I can quickly find a specific exercise.

### Acceptance Criteria

- Exercise List page header displays a search input
- Typing a search term filters the exercise list
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

**As a** _user_, **I want to** add a custom exercise to the exercise database,
**so that** I can log exercises that are not in the built-in list.

### Acceptance Criteria

- A "+" button is located near the search bar
  - **Click it**, the Add Exercise form opens
- Filling in the exercise name and clicking save button creates a new exercise
- The new exercise is visible in the Exercise List

## Edit a Cloned Exercise

**As a** _user_, **I want to** edit a built-in exercise after cloning it, **so
that** I can create a custom clone that fits my training better.

### Acceptance Criteria

- A "✏️ Clone & Edit" button is located on each built-in exercise card
  - **Click it**, the Edit Exercise form opens
- Changin the exercise name and clicking save button creates a new exercise
- The modified clone is visible in the Exercise List with the new name

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

- The Home page contains a "+" button
  - **Click it**, the Active Session _view_ opens
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

## Full Workout Session

I want to improve my health and track my progress to continue improving further.

### Start Session

**As a** _user_, **I want to** start a new workout session, **so that** I can
begin logging my exercises.

#### Acceptance Criteria

- The Home page contains a "+" button
  - **Click it**, the Active Session _view_ opens
- Active Session
  - Header displays "Active Session", "Cancel Session" button and a timer
  - After 1 second, the timer has incremented by 1 second

### Search Exercise

**As a** _user_, **I want to** search for an exercise in my active session, **so
that** I can log it.

#### Acceptance Criteria

- Input an exercise name, category, or muscle in the search bar
- Select an exercise from the search results to open the exercise form

### Record Exercise

**As a** _user_, **I want to** record a completed exercise in my active session,
**so that** I can track my progress.

#### Acceptance Criteria

- Input some metrics
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

- Finish the session using "Finish Session"
- The home screen shows the completed session with the current date
- A congratulation toast appears

## Repeat Session from History

**As a** _user_, **I want to** start a new session based on a past workout, **so
that** I can quickly repeat the same exercises.

### Acceptance Criteria

- A completed session with at least one exercise is visible on the home screen
- **Click** 🔄 button on a previous Session card, a new Active Session opens
- The "Pre-added Exercises" section contains the previous session exercises

## Delete a Past Session

**As a** _user_, **I want to** delete a completed session from my history, **so
that** I can remove unwanted or accidental entries.

### Acceptance Criteria

- **Click** 🗑️ button on the latest Session card, a confirmation dialog opens
- Confirming the deletion removes the session from the home screen

## Lookup a Previously Done Exercise

**As a** _user_, **I want to** look up exercises I have previously done, **so
that** I can find similar exercises.

### Acceptance Criteria

- The Session card contains done exercise(s) tags
  - **Click one** of the tags, the Exercise List opens
- Exercise List
  - The search bar is pre-filled with the name of the selected exercise

## Navigate Again to Analytics

**As a** _user_, **I want to** navigate to the analytics page, **so that** I can
view my workout progress over time, now that I have recorded some exercises.

### Acceptance Criteria

- The bottom navigation bar contains a "📊" button
  - **Click it**, the Analytics page opens
- Header displays "📊 Analytics"
- "📊" button is slightly emphasized compared to others in bottom navigation bar

## Be Astonished by Your Incredible Progress

**As a** _user_, **I want to** see my workout progress over time in the
analytics page, **so that** I can be astonished by my incredible progress and
stay motivated to continue improving further.

### Acceptance Criteria

- Select a metric worked in the previous full session
  - Among "Weight", "Repetitions", "Distance", "Duration"
- Select two exercises for which this metric has improved between previous sets
- Be astonished by the raising curves in the analytics charts
