CREATE TABLE reports
(
 id SERIAL PRIMARY KEY,
 reported TEXT NOT NULL,
 reporter TEXT NOT NULL,
 description TEXT NOT NULL
)
