# Dates and times

The system for storing timestamps is complicated within the journal.

* The "written date" should always be presented as the day the *author* wrote it in their local time. This is important because of the potential relevance to the date of an entry to the contents described within (i.e. an entry might say "yesterday as of writing," and the reader should be able to identify what day that was from the author's perspective). Section timestamps are stored server side as a YMD string and stringified using the NaiveDate API.
* All other timestamps, such as timestamps for when a user has read a section or when a comment was written, should be stored as the number of milliseconds since the epoch. These are transmitted rawly to the client, and the client uses their timezone to parse it.
