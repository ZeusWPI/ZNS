-- Your SQL goes here
CREATE TABLE records (
  name TEXT NOT NULL,
  type INT NOT NULL,
  class INT NOT NULL,
  ttl INT NOT NULL,
  rdlength INT NOT NULL,
  rdata BLOB NOT NULL,

  PRIMARY KEY (name,type,class)
)
