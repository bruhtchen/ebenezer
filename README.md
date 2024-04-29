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

## TODO

[ ] **Rollover** : copy all the fixed and estimated expenses for the new period.

[ ] **Code** : use Clap for a better CLI experience. 

[ ] **List** : list subitems of expenses to track individual payments.