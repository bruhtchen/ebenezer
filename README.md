# Ebenezer - a lightweight CLI personal budget app 

## Purpose

This application is intended to be a drop-in replacement for the three-columns Excel worksheet I use for tracking my monthly expenses. As such, it is a **personal** project, emphasis on "personal" - this would not, in any way, be suited to any kind of corporate environment.

As an aside, it was also a way for me to learn a bit of Rust.

## Concepts

The application tracks **incomes** and **expenses** over some **period** of time. The corresponding data is stored in a SQLite database.

* An **income** is just that : some amount of money moving towards you.
* **Expenses** can be of three kinds : 
    **fixed** expenses occur once every period, and the amount of money associated is known beforehand. ex: mortgage payments, recurring utility bills.
    **estimated** expenses occur every period, but their amount cannot be known in advance. ex: groceries.
    **unplanned** expenses are the unwelcome surprises that happen from time to time.
* **Periods** are lapses of time, with a start date and an optional end date.

Once you put in some incomes and expenses, you can get your current balance, your expected balance at end of period, and an overview of all expense categories.

## Commands

Usage: ebenezer [COMMAND]

Commands:
  [empty]   Print the current balance
  list      List incomes and expenses
  logs      List every transaction of the current period for auditing purposes
  logs-all  List every transaction for auditing purposes
  roll      Switch to a new period
  period    Display the current period
  remove    Remove an expense line
  spend     Spend some money on an expense line. If amount is omitted, the whole expense is spent
  fixed     Create a new constant expense line
  estimate  Create a new estimated expense line
  income    Create a new constant expense line
  rename    Change the label of an expense line
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

