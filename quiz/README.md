# Quiz

This example shows a simple multiple-choice quiz contract.

The owner creates quizzes with a question, answer options, and the correct option
index. Players submit one answer per quiz, and the contract records whether the
answer was correct. Player stats track total answers and correct answers across
all quizzes.

## Messages

- `CreateQuiz`: owner-only creation of a multiple-choice quiz.
- `SubmitAnswer`: records one answer per player for a quiz.
- `CloseQuiz`: owner-only manual quiz close.
- `UpdateConfig`: owner-only ownership transfer and validation-limit updates.

## Queries

- `Config`: returns owner and validation limits.
- `Quiz`: returns one quiz.
- `Answer`: returns one player's answer for one quiz.
- `PlayerStats`: returns total and correct answer counts for one player.
- `ListQuizzes`: returns quizzes by ascending quiz ID.

## Example flow

1. Instantiate the contract.
2. The owner creates a quiz with several answer options.
3. Players submit answers before the optional closing height.
4. Query `PlayerStats` or `Answer` to see results.
