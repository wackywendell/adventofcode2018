--- Day 21: Chronal Conversion ---

You should have been watching where you were going, because as you wander the new North Pole base, you trip and fall into a very deep hole!

Just kidding. You're falling through time again.

If you keep up your current pace, you should have resolved all of the temporal anomalies by the next time the device activates. Since you have very little interest in browsing history in 500-year increments for the rest of your life, you need to find a way to get back to your present time.

After a little research, you discover two important facts about the behavior of the device:

First, you discover that the device is hard-wired to always send you back in time in 500-year increments. Changing this is probably not feasible.

Second, you discover the activation system (your puzzle input) for the time travel module. Currently, it appears to run forever without halting.

If you can cause the activation system to halt at a specific moment, maybe you can make the device send you so far back in time that you cause an integer underflow in time itself and wrap around back to your current time!

The device executes the program as specified in manual section one and manual section two.

Your goal is to figure out how the program works and cause it to halt. You can only control register 0; every other register begins at 0 as usual.

Because time travel is a dangerous activity, the activation system begins with a few instructions which verify that bitwise AND (via bani) does a numeric operation and not an operation as if the inputs were interpreted as strings. If the test fails, it enters an infinite loop re-running the test instead of allowing the program to execute normally. If the test passes, the program continues, and assumes that all other bitwise operations (banr, bori, and borr) also interpret their inputs as numbers. (Clearly, the Elves who wrote this system were worried that someone might introduce a bug while trying to emulate this system with a scripting language.)

What is the lowest non-negative integer value for register 0 that causes the program to halt after executing the fewest instructions? (Executing the same instruction multiple times counts as multiple instructions executed.)

--- Part Two ---

In order to determine the timing window for your underflow exploit, you also need an upper bound:

What is the lowest non-negative integer value for register 0 that causes the program to halt after executing the most instructions? (The program must actually halt; running forever does not count as halting.)

### My Notes

// 5675749 is too high.

```
#ip 5
0. seti 123 0 3         // x3 = 123
1. bani 3 456 3         // x3 = x3 & 456 = 72
2. eqri 3 72 3          // x3 = x3 == 72 = 1
3. addr 3 5 5           // ip = x5 = (x3 + x5) = 4
                        // i.e. "if x3 == 72, skip next"
4. seti 0 0 5           // ip = x5 = 0
                        // i.e. GOTO 1
5. seti 0 9 3           // x3 = 0
////////////////////////////////////////////////////////////
6. bori 3 65536 1       // x1 = x3 | 65536
7. seti 14906355 8 3    // x3 = 14906355
////////////////////////////////////////////////////////////
x4 = x1 & 255
x3 = (((x3 + (x1 & 255)) & 16777215) * 65899) & 16777215

8. bani 1 255 4         // x4 = x1 & 255
9. addr 3 4 3           // x3 = x3 + x4
10. bani 3 16777215 3   // x3 = x3 & 16777215
11. muli 3 65899 3      // x3 = x3 * 65899
12. bani 3 16777215 3   // x3 = x3 & 16777215
////////////////////////////////////////////////////////////
if 256 > x1 {
    GOTO 28
} else {
    x4 = 0
}
13. gtir 256 1 4        // x4 = 256 > x1
14. addr 4 5 5          // x5 = x4 + x5
15. addi 5 1 5          // x5 = x5 + 1
16. seti 27 8 5         // x5 = 27    i.e. GOTO 28
17. seti 0 4 4          // x4 = 0
////////////////////////////////////////////////////////////
loop {
    x2 = (x4 + 1) * 256 > x1
    if x2 {
        x1 = x4
        GOTO 8
    }
    x4 = x4 + 1
}
18. addi 4 1 2          // x2 = x4 + 1
19. muli 2 256 2        // x2 = x2 * 256
20. gtrr 2 1 2          // x2 = x2 > x1
21. addr 2 5 5          // x5 = x2 + x5
22. addi 5 1 5          // x5 = x5 + 1
23. seti 25 1 5         // x5 = 25    i.e. GOTO 26
24. addi 4 1 4          // x4 = x4 + 1
25. seti 17 2 5         // ip = x5 = 17    i.e. GOTO 18
26. setr 4 9 1          // x1 = x4
27. seti 7 0 5          // ip = x5 = 7    i.e. GOTO 8
////////////////////////////////////////////////////////////
if x3 == x0 {
    return
}
GOTO 6

28. eqrr 3 0 4          // x4 = (x3 == x0)
29. addr 4 5 5          // x5 = x4 + x5
30. seti 5 3 5          // x5 = 5    i.e. GOTO 6
```

```
6:
    x1 = x3 | 65536
    x3 = 14906355
8:
    x4 = x1 & 255
    x3 = (((x3 + (x1 & 255)) & 16777215) * 65899) & 16777215
13:
    if 256 > x1 {
        GOTO 28
    } else {
        x4 = 0
    }
18:
    loop {
        x2 = (x4 + 1) * 256 > x1
        if x2 {
            x1 = x4
            GOTO 8
        }
        x4 = x4 + 1
    }
28:
    if x3 == x0 {
        return
    }
    GOTO 6
```

```
'a: loop {
    x4 = x1 & 255
    x3 = (((x3 + (x1 & 255)) & 16777215) * 65899) & 16777215

    if 256 > x1 {
        if x3 == x0 {
            return
        }
        x1 = x3 | 65536
        x3 = 14906355
        continue
    } else {
        x4 = 0
    }

    loop {
        x2 = (x4 + 1) * 256 > x1
        if x2 {
            x1 = x4
            continue 'a
        }
        x4 = x4 + 1
    }
}
```

```
'a: loop {
    x4 = x1 & 255
    x3 = (((x3 + (x1 & 255)) & 16777215) * 65899) & 16777215

    if 256 > x1 {
        if x3 == x0 {
            return
        }
        x1 = x3 | 65536
        x3 = 14906355
        continue
    } else {
        x4 = 0
    }

    loop {
        x2 = (x4 + 1) * 256 > x1
        if x2 {
            x1 = x4
            continue 'a
        }
        x4 = x4 + 1
    }
}
```

### Part 2

13202558 is too high.

12464363 is the right answer.
