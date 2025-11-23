-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
           'ddf8994f-d522-4659-8d02-c1d479057be6',
           'admin',
           '$argon2id$v=19$m=15000,t=2,p=1$OEx/rcq+3ts//WUDzGNl2g$Am8UFBA4w5NJEmAtquGvBmAlu92q/VQcaoL5AyJPfc8'
       );

INSERT INTO users
VALUES (
        'ddf8994f-d522-4659-8d02-c1d479057be7',
        'hellsent',
        '$argon2id$v=19$m=15000,t=2,p=1$820o2oDiTGqxBIoACfctoQ$JUlmCGx+tyeHT5nXuoBisUQBTsNSeLtcXk+ShLfi694'
       );