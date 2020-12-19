CREATE TABLE reports
(
 "id"       bigint NOT NULL GENERATED ALWAYS AS IDENTITY (
 start 1
 ),
 reported varchar(36) NOT NULL,
 reporter varchar(36) NOT NULL,
 "desc"     varchar(255) NOT NULL,
 CONSTRAINT PK_reports PRIMARY KEY ( "id" )
)
