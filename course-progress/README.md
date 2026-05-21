# Course Progress

This example shows a simple course-progress tracker. A contract owner creates
courses made of ordered lessons, and learners mark lessons as complete as they
work through the course.

The contract demonstrates:

- owner-created course records
- lesson-list validation and configurable lesson limits
- learner-owned progress records
- duplicate lesson-completion protection
- progress reset by the learner
- course archiving by the owner
- ownership transfer and configurable validation limits
- single-course, progress, count, config, and paginated course-list queries

## Messages

Instantiate the contract with optional limits:

```json
{
  "max_text_length": 120,
  "max_lessons_per_course": 20
}
```

Create a course:

```json
{
  "create_course": {
    "title": "CosmWasm Basics",
    "description": "A short introduction to contracts",
    "lessons": ["Instantiate", "Execute", "Query"]
  }
}
```

Complete a lesson:

```json
{
  "complete_lesson": {
    "course_id": 1,
    "lesson_index": 0
  }
}
```

Reset your own progress:

```json
{
  "reset_progress": {
    "course_id": 1
  }
}
```

Archive a course:

```json
{
  "archive_course": {
    "course_id": 1
  }
}
```

Update config or transfer ownership:

```json
{
  "update_config": {
    "owner": "cosmos1newowner...",
    "max_text_length": 160,
    "max_lessons_per_course": 30
  }
}
```

## Queries

Get contract configuration:

```json
{
  "config": {}
}
```

Get a course:

```json
{
  "course": {
    "course_id": 1
  }
}
```

Get learner progress:

```json
{
  "progress": {
    "course_id": 1,
    "learner": "cosmos1..."
  }
}
```

List courses:

```json
{
  "list_courses": {
    "start_after": 1,
    "limit": 20
  }
}
```
