# yesterday_weather

A small program that connects to weather underground and uses their api to gather information about the previous day.
That info is then used to calculate the [heat units](http://sandhillpreservation.com/pages/sweetpotato_catalog.html) 
and record them in a sqlite database.  The goal of this program is to help determine when sweet potatoes can be harvested.
According to sand hill preservation it takes about 1200 heat units to ripen an early sweet potato variety.  I also record
rainfall amounts which could prove useful.

Once your database is up and running you can connect to it with:
```
sqlite3 heat_units.sqlite3 
SQLite version 3.11.0 2016-02-15 17:29:24
Enter ".help" for usage hints.
sqlite> select * from heatunits;
63|48|2017-04-06 12:00|0|0.38

```
The database schema is: `CREATE TABLE heatunits (temp_min int, temp_max int,date datetime,heat_units int, rainfall int, primary key (date));`
