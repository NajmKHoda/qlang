datasource journal_db;

table User from journal_db {
  name: str,
  age: int,
  is_male: bool
}

table JournalEntry from journal_db {
  title: str,
  body: str
}

failable function main() -> int {
  let users = query { select all from User };
  let user = User { name: "", age: 0, is_male: true };
  if users.has_next() {
    user = users.next();
    prints("Welcome back, " + user.name + "! Let's get started.");
  } else {
    user = doIntroduction();
  }

  while true {
    prints("Would you like to LOG or VIEW a journal entry, or STOP?");
    let command = inputs();
    if command == "LOG" {
      writeJournalEntry();
    } else if command == "VIEW" {
      viewJournalEntry();
    } else if command == "STOP" {
      break;
    } else {
      prints("I don't recognize that command.");
    }
  }
}

failable function doIntroduction() -> User {
  let valid = false;
  let name = "";
  prints("Hey there, new user! What's your name?");
  while not valid {
    name = inputs();
    if name == "" {
      prints("You have to have a name! Try entering it again.");
    } else {
      valid = true;
    }
  }
  
  valid = false;
  let age = 0;
  prints("Nice to meet you, " + name + "! How old are you?");
  while not valid {
    age = inputi();
    if age < 0 {
      prints("I don't think that's possible... try again!");
    } else {
      valid = true;
    }
  }

  valid = false;
  let is_male = true;
  prints("Are you male or female (M/F)?");
  while not valid {
    let gender_input = inputs();
    if gender_input == "M" {
      valid = true;
    } else if gender_input == "F" {
      is_male = false;
      valid = true;
    } else {
      prints("Whoops! Please enter M or F.");
    }
  }

  let gender_string = "male";
  if not is_male {
    gender_string = "female";
  }

  prints("Great! So your name is " + name + ", you're " + str(age)
    + " years old, and you're " + gender_string + "! Let's get started.");

  let user = User {
    name: name,
    age: age,
    is_male: is_male
  };
  query { insert user into User };

  return user;
}

failable function viewJournalEntry() -> void {
  prints("What's the title of the journal entry you want to see?");
  let entry_title = inputs();
  let entries = query {
    select all from JournalEntry
    where title == entry_title
  };

  if not entries.has_next() {
    prints("No journal log with that title exists.");
    return;
  }

  let entry = entries.next();
  prints("Here's your journal entry!");
  prints("");
  prints(entry.title);
  prints("------------------");
  prints(entry.body);
  prints("");
}

failable function writeJournalEntry() -> void {
  prints("Give a title to your journal entry.");
  let entry_title = inputs();
  let entries = query {
    select all from JournalEntry
    where title == entry_title
  };

  let overwrite = false;
  if entries.has_next() {
    prints("A journal entry with that title already exists. Overwrite it? (y/N)");
    overwrite = (inputs() == "Y");
  }

  prints("Write to your heart's content!");
  let entry_body = inputs();

  while true {
    transaction {
      if overwrite {
        query {
          update JournalEntry
          set body = entry_body
          where title == entry_title
        };
      } else {
        query {
          insert { title: entry_title, body: entry_body }
          into JournalEntry
        };
      }

      prints("Saved successfully!");
      return;
    } on rollback {
      prints("Saving failed. Try again? (Y/n)");
      let response = inputs();
      if response == "N" {
        return;
      }
    }
  }
}
